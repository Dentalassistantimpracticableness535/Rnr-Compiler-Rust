//! Minimal codegen skeleton.
use crate::ast::*;
use crate::error::Error;
use mips::{asm::*, instr::Instr, instrs::Instrs, rf::Reg};
use std::collections::HashMap;

pub struct CodeGen {
    instrs: Vec<String>,
    label_counter: usize,
    params: HashMap<String, i32>,
    locals: HashMap<String, i32>,
    pending_locals: HashMap<String, Vec<i32>>,
    next_local: i32,
    strings: HashMap<String, String>,
    data_labels: Vec<(String, String)>,
}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            instrs: Vec::new(),
            label_counter: 0,
            params: HashMap::new(),
            locals: HashMap::new(),
            pending_locals: HashMap::new(),
            next_local: -4,
            strings: HashMap::new(),
            data_labels: Vec::new(),
        }
    }

    // For building instruction list
    fn emit(&mut self, s: impl Into<String>) {
        self.instrs.push(s.into());
    }

    // For generating labels
    fn fresh_label(&mut self, base: &str) -> String {
        let l = format!("{}_{}", base, self.label_counter);
        self.label_counter += 1;
        l
    }

    fn push_reg(&mut self, reg: &str) {
        self.emit("    addi sp, sp, -4");
        self.emit(format!("    sw   {}, 0(sp)", reg));
    }

    fn pop_reg(&mut self, reg: &str) {
        self.emit(format!("    lw   {}, 0(sp)", reg));
        self.emit("    addi sp, sp, 4");
    }

    pub fn generate(&mut self, prog: &Prog) -> Result<String, Error> {
        for f in &prog.0 {
            self.generate_function(f)?;
        }

        let mut out: Vec<String> = Vec::new();
        if !self.data_labels.is_empty() {
            out.push(".data".to_string());
            for (label, content) in &self.data_labels {
                let esc = content
                    .replace("\\", "\\\\")
                    .replace("\"", "\\\"")
                    .replace("\n", "\\n");
                out.push(format!("{}: .asciiz \"{}\"", label, esc));
            }
            out.push(".text".to_string());
        }
        out.extend(self.instrs.clone());
        // If code contains calls to the runtime `println` label, emit a small stub.
        if self.instrs.iter().any(|s| {
            let parts: Vec<&str> = s.split_whitespace().collect();
            parts.windows(2).any(|w| w[0] == "bal" && w[1] == "println")
        }) {
            out.push("\n# runtime stubs".to_string());
            out.push("println:".to_string());
            out.push("    jr   ra".to_string());
        }
        Ok(out.join("\n"))
    }

    fn intern_string(&mut self, s: &str) -> String {
        if let Some(l) = self.strings.get(s) {
            return l.clone();
        }
        let lbl = format!("str_{}", self.label_counter);
        self.label_counter += 1;
        self.strings.insert(s.to_string(), lbl.clone());
        self.data_labels.push((lbl.clone(), s.to_string()));
        lbl
    }

    fn generate_function(&mut self, f: &FnDeclaration) -> Result<(), Error> {
        self.params.clear();
        self.locals.clear();
        self.next_local = -4;

        let fn_start = self.instrs.len();
        self.emit(format!("{}:", f.id));
        self.emit("    addi sp, sp, -4");
        self.emit("    sw   ra, 0(sp)");
        self.emit("    addi sp, sp, -4");
        self.emit("    sw   fp, 0(sp)");
        self.emit("    addi fp, sp, 0");

        // params offsets
        let n_params = f.parameters.0.len() as i32;
        for (i, p) in f.parameters.0.iter().enumerate() {
            let offset = 8 + 4 * (n_params - 1 - i as i32);
            self.params.insert(p.id.clone(), offset);
        }

        //  collect locals and reserve frame space once
        let locals = self.collect_locals_block(&f.body);
        let n_locals = locals.len() as i32;
        if n_locals > 0 {
            self.emit(format!("    addi sp, sp, -{}", n_locals * 4));
            // assign offsets into pending map (do not make them visible yet)
            for (i, id) in locals.into_iter().enumerate() {
                let off = -4 - 4 * (i as i32);
                self.pending_locals.entry(id).or_default().push(off);
            }
            self.next_local = -4 - 4 * n_locals;
        }

        // Generate body; if function with not Unit return type,
        // we need to keep the last expression value on the stack to return it
        let is_tail = match &f.ty {
            Some(t) => !matches!(t, crate::ast::Type::Unit),
            None => false,
        };
        self.gen_block(&f.body, is_tail)?;

        if is_tail {
            // if returning value -> pop return into t0 and return in register
            self.pop_reg("t0");
            if n_locals > 0 {
                let size = n_locals * 4;
                self.emit(format!("    addi sp, sp, {}", size));
            }
            self.emit("    lw   fp, 0(sp)");
            self.emit("    addi sp, sp, 4");
            self.emit("    lw   ra, 0(sp)");
            self.emit("    addi sp, sp, 4");
            // leave return value in t0
            self.emit("    jr   ra");
        } else {
            if n_locals > 0 {
                let size = n_locals * 4;
                self.emit(format!("    addi sp, sp, {}", size));
            }
            self.emit("    lw   fp, 0(sp)");
            self.emit("    addi sp, sp, 4");
            self.emit("    lw   ra, 0(sp)");
            self.emit("    addi sp, sp, 4");
            self.emit("    jr   ra");
        }
        self.emit("");

        Ok(())
    } // collect local variables id in blocks
    fn collect_locals_block(&self, b: &Block) -> Vec<String> {
        let mut names: Vec<String> = Vec::new();
        self.collect_locals_block_inner(b, &mut names);
        names
    }

    fn collect_locals_block_inner(&self, b: &Block, names: &mut Vec<String>) {
        for stmt in &b.statements {
            match stmt {
                Statement::Let(_m, id, _ty, _init) => {
                    // Collect every local declaration occurrence
                    names.push(id.clone());
                }
                Statement::While(_cond, body) => self.collect_locals_block_inner(body, names),
                Statement::Expr(e) => self.collect_locals_expr_inner(e, names),
                Statement::Fn(_) => {}
                Statement::Assign(_, _) => {}
            }
        }
    }

    fn collect_locals_expr_inner(&self, e: &Expr, names: &mut Vec<String>) {
        match e {
            Expr::IfThenElse(_c, then_b, else_b) => {
                self.collect_locals_block_inner(then_b, names);
                if let Some(eb) = else_b {
                    self.collect_locals_block_inner(eb, names);
                }
            }
            Expr::Block(b) => self.collect_locals_block_inner(b, names),
            Expr::BinOp(_op, l, r) => {
                self.collect_locals_expr_inner(l, names);
                self.collect_locals_expr_inner(r, names);
            }
            Expr::Par(i) | Expr::UnOp(_, i) | Expr::Ref(i, _) => {
                self.collect_locals_expr_inner(i, names)
            }
            Expr::Call(_, args) => {
                for a in &args.0 {
                    self.collect_locals_expr_inner(a, names);
                }
            }
            Expr::Ident(_) | Expr::Lit(_) => {}
        }
    }

    fn gen_block(&mut self, b: &Block, is_tail: bool) -> Result<(), Error> {
        // Snapshot current locals so that bindings introduced inside the
        // block do not leak to outer scopes
        let saved_locals = self.locals.clone();
        let len = b.statements.len();
        for (i, stmt) in b.statements.iter().enumerate() {
            let is_last = i + 1 == len;
            self.gen_stmt(stmt, is_tail && is_last)?; // boolean to know if we let the value on stack
        }
        // restore outer scope locals
        self.locals = saved_locals;
        Ok(())
    }

    fn gen_stmt(&mut self, s: &Statement, is_tail: bool) -> Result<(), Error> {
        match s {
            Statement::Let(_m, id, _ty, init_opt) => {
                if let Some(init) = init_opt {
                    // Evaluate initializer before making the new binding visible
                    self.gen_expr(init)?;
                    self.pop_reg("t0");
                    // If an explicit offset was precomputed, make it visible now;
                    // otherwise allocate fresh slot

                    // Always allocate a fresh binding
                    let off = if let Some(vec) = self.pending_locals.get_mut(id) {
                        if !vec.is_empty() {
                            let off_val = vec.remove(0);
                            if vec.is_empty() {
                                self.pending_locals.remove(id);
                            }
                            self.locals.insert(id.clone(), off_val);
                            off_val
                        } else {
                            let off = self.next_local;
                            self.locals.insert(id.clone(), off);
                            self.next_local -= 4;
                            off
                        }
                    } else {
                        let off = self.next_local;
                        self.locals.insert(id.clone(), off);
                        self.next_local -= 4;
                        off
                    };
                    self.emit(format!("    sw   t0, {}(fp)", off));
                    Ok(())
                } else {
                    // uninitialized let (initialize to 0, unit/zero)
                    // allocate a fresh binding
                    let off = if let Some(vec) = self.pending_locals.get_mut(id) {
                        if !vec.is_empty() {
                            let off_val = vec.remove(0);
                            if vec.is_empty() {
                                self.pending_locals.remove(id);
                            }
                            self.locals.insert(id.clone(), off_val);
                            off_val
                        } else {
                            let off = self.next_local;
                            self.locals.insert(id.clone(), off);
                            self.next_local -= 4;
                            off
                        }
                    } else {
                        let off = self.next_local;
                        self.locals.insert(id.clone(), off);
                        self.next_local -= 4;
                        off
                    };
                    self.emit("    addi t0, zero, 0".to_string());
                    self.emit(format!("    sw   t0, {}(fp)", off));
                    Ok(())
                }
            }
            Statement::Assign(lhs, rhs) => {
                self.gen_expr(rhs)?;
                self.pop_reg("t0");

                match lhs {
                    Expr::Ident(name) => {
                        if let Some(off) = self.locals.get(name) {
                            self.emit(format!("    sw   t0, {}(fp)", off));
                            Ok(())
                        } else if let Some(off) = self.params.get(name) {
                            self.emit(format!("    sw   t0, {}(fp)", off));
                            Ok(())
                        } else {
                            Err(format!("unknown variable '{}'", name))
                        }
                    }
                    Expr::UnOp(op, inner) => {
                        // assignment to dereference: *inner = rhs
                        if let crate::ast::UnOp::Deref = op {
                            match &**inner {
                                Expr::Ident(name) => {
                                    if let Some(off) = self.locals.get(name) {
                                        // r contains an address; load it into t1 then store
                                        self.emit(format!("    lw   t1, {}(fp)", off));
                                        self.emit("    sw   t0, 0(t1)");
                                        Ok(())
                                    } else if let Some(off) = self.params.get(name) {
                                        self.emit(format!("    lw   t1, {}(fp)", off));
                                        self.emit("    sw   t0, 0(t1)");
                                        Ok(())
                                    } else {
                                        Err(format!(
                                            "unknown identifier '{}' for deref assign",
                                            name
                                        ))
                                    }
                                }
                                _ => {
                                    // evaluate inner to produce an address on the stack, pop into t1 and store
                                    self.gen_expr(inner)?; // should leave address on stack
                                    self.pop_reg("t1");
                                    self.emit("    sw   t0, 0(t1)");
                                    Ok(())
                                }
                            }
                        } else {
                            Err("assignment to non-identifier not supported".to_string())
                        }
                    }
                    Expr::Ref(inner, _) => {
                        // assign to a ref : compute address then store
                        match &**inner {
                            Expr::Ident(name) => {
                                if let Some(off) = self.locals.get(name) {
                                    self.emit(format!("    addi t1, fp, {}", off));
                                    self.emit("    sw   t0, 0(t1)");
                                    Ok(())
                                } else if let Some(off) = self.params.get(name) {
                                    self.emit(format!("    addi t1, fp, {}", off));
                                    self.emit("    sw   t0, 0(t1)");
                                    Ok(())
                                } else {
                                    Err(format!("unknown identifier '{}' for ref assign", name))
                                }
                            }
                            _ => {
                                // For other reference targets, try to evaluate the inner expression to obtain
                                // an address on the stack, pop it into t1 and store the RHS into 0(t1).
                                self.gen_expr(inner)?; // attempt to produce an address/value on the stack
                                self.pop_reg("t1");
                                self.emit("    sw   t0, 0(t1)");
                                Ok(())
                            }
                        }
                    }
                    _ => Err("assignment to non-ident/ref not supported".to_string()),
                }
            }
            Statement::While(cond, body) => {
                let start = self.fresh_label("while_start");
                let end = self.fresh_label("while_end");
                self.emit(format!("{}:", start));
                self.gen_expr(cond)?;
                self.pop_reg("t0");
                self.emit(format!("    beq  t0, zero, {}", end));
                self.gen_block(body, false)?;
                self.emit(format!("    b    {}", start));
                self.emit(format!("{}:", end));
                Ok(())
            }
            Statement::Expr(e) => {
                self.gen_expr(e)?;
                if is_tail {
                    // let value on stack for caller (function return)
                    Ok(())
                } else {
                    self.pop_reg("t0");
                    Ok(())
                }
            }
            Statement::Fn(_) => Ok(()),
        }
    }

    fn gen_expr(&mut self, e: &Expr) -> Result<(), Error> {
        match e {
            Expr::Ident(name) => {
                if let Some(off) = self.locals.get(name) {
                    self.emit(format!("    lw   t0, {}(fp)", off));
                    self.push_reg("t0");
                    Ok(())
                } else if let Some(off) = self.params.get(name) {
                    self.emit(format!("    lw   t0, {}(fp)", off));
                    self.push_reg("t0");
                    Ok(())
                } else {
                    Err(format!("unknown identifier '{}'", name))
                }
            }

            Expr::Lit(l) => {
                match l {
                    Literal::Int(n) => {
                        self.emit(format!("    addi t0, zero, {}", n));
                        self.push_reg("t0");
                    }
                    Literal::Bool(b) => {
                        let v = if *b { 1 } else { 0 };
                        self.emit(format!("    addi t0, zero, {}", v));
                        self.push_reg("t0");
                    }
                    Literal::String(s) => {
                        let lbl = self.intern_string(s);
                        self.emit(format!("    la   t0, {}", lbl));
                        self.push_reg("t0");
                    }
                    Literal::Unit => {}
                }
                Ok(())
            }
            Expr::Par(inner) => self.gen_expr(inner),
            Expr::BinOp(op, l, r) => {
                self.gen_expr(l)?;
                self.gen_expr(r)?;
                self.pop_reg("t1");
                self.pop_reg("t0");
                match op {
                    crate::ast::BinOp::Add => self.emit("    add  t0, t0, t1"),
                    crate::ast::BinOp::Sub => self.emit("    sub  t0, t0, t1"),
                    crate::ast::BinOp::Mul => self.emit("    mul  t0, t0, t1"),
                    crate::ast::BinOp::Div => self.emit("    div  t0, t0, t1"),
                    crate::ast::BinOp::And => self.emit("    and  t0, t0, t1"),
                    crate::ast::BinOp::Or => self.emit("    or   t0, t0, t1"),
                    crate::ast::BinOp::Eq => {
                        let l_true = self.fresh_label("eq_true");
                        let l_end = self.fresh_label("eq_end");
                        self.emit("    sub  t2, t0, t1");
                        self.emit(format!("    beq  t2, zero, {}", l_true));
                        self.emit("    addi t0, zero, 0");
                        self.emit(format!("    b    {}", l_end));
                        self.emit(format!("{}:", l_true));
                        self.emit("    addi t0, zero, 1");
                        self.emit(format!("{}:", l_end));
                    }
                    crate::ast::BinOp::Lt => self.emit("    slt  t0, t0, t1"),
                    crate::ast::BinOp::Gt => self.emit("    slt  t0, t1, t0"),
                }
                self.push_reg("t0");
                Ok(())
            }
            Expr::UnOp(op, inner) => {
                self.gen_expr(inner)?;
                self.pop_reg("t0");
                match op {
                    crate::ast::UnOp::Neg => self.emit("    sub  t0, zero, t0"),
                    crate::ast::UnOp::Bang => {
                        let l_true = self.fresh_label("not_true");
                        let l_end = self.fresh_label("not_end");
                        self.emit(format!("    beq  t0, zero, {}", l_true));
                        self.emit("    addi t0, zero, 0");
                        self.emit(format!("    b    {}", l_end));
                        self.emit(format!("{}:", l_true));
                        self.emit("    addi t0, zero, 1");
                        self.emit(format!("{}:", l_end));
                    }
                    crate::ast::UnOp::Deref => self.emit("    lw   t0, 0(t0)"),
                }
                self.push_reg("t0");
                Ok(())
            }
            Expr::Call(name, args) => {
                if name == "println!" {
                    // Push all arguments onto the stack, but ensure
                    // the (format string is pushed last so that it appears at 0(sp)
                    let mut argc: i32 = 0;
                    let mut fmt_lbl: Option<String> = None;
                    for a in &args.0 {
                        match a {
                            Expr::Lit(Literal::String(s)) => {
                                // defer pushing format string until last
                                fmt_lbl = Some(self.intern_string(s));
                            }
                            _ => {
                                self.gen_expr(a)?; // leaves value on stack
                                argc += 1;
                            }
                        }
                    }
                    //  push the format string (it ends up at 0(sp))
                    if let Some(lbl) = fmt_lbl {
                        self.emit(format!("    la   t0, {}", lbl));
                        self.push_reg("t0");
                        argc += 1;
                    }
                    // set t2 = argc for the runtime to know how many args
                    if argc > 0 {
                        self.emit(format!("    addi t2, zero, {}", argc));
                    } else {
                        self.emit("    addi t2, zero, 0".to_string());
                    }
                    self.emit("    bal  println");
                    if argc > 0 {
                        self.emit(format!("    addi sp, sp, {}", argc * 4));
                    }
                    // println returns unit
                    self.emit("    addi t0, zero, 0");
                    self.push_reg("t0");
                    Ok(())
                } else {
                    let mut argc: i32 = 0;
                    for a in &args.0 {
                        // we generate args so they are ready on the stack
                        self.gen_expr(a)?;
                        argc += 1;
                    }
                    self.emit(format!("    bal {}", name));
                    if argc > 0 {
                        self.emit(format!("    addi sp, sp, {}", argc * 4));
                    }
                    self.push_reg("t0");
                    Ok(())
                }
            }
            Expr::IfThenElse(cond, then_blk, else_blk) => {
                self.gen_expr(cond)?;
                self.pop_reg("t0");
                let l_else = self.fresh_label("if_else");
                let l_end = self.fresh_label("if_end");
                self.emit(format!("    beq  t0, zero, {}", l_else));
                // gnerate then-branch with is_tail=true so it leaves its value on stack
                self.gen_block(then_blk, true)?;
                self.emit(format!("    b    {}", l_end));
                self.emit(format!("{}:", l_else));
                if let Some(eb) = else_blk {
                    // generate else-branch with is_tail=true so it leaves its value on stack
                    self.gen_block(eb, true)?;
                } else {
                    // If no else-branch, push unit/0 as a placeholder value
                    self.emit("    addi t0, zero, 0");
                    self.push_reg("t0");
                }
                self.emit(format!("{}:", l_end));
                Ok(())
            }
            Expr::Block(b) => self.gen_block(b, false),
            Expr::Ref(inner, _m) => {
                // If referencing an identifier, generate its address (fp + offset)
                match &**inner {
                    Expr::Ident(name) => {
                        if let Some(off) = self.locals.get(name) {
                            // address = fp + off
                            self.emit(format!("    addi t0, fp, {}", off));
                            self.push_reg("t0");
                            Ok(())
                        } else if let Some(off) = self.params.get(name) {
                            self.emit(format!("    addi t0, fp, {}", off));
                            self.push_reg("t0");
                            Ok(())
                        } else {
                            Err(format!("unknown identifier '{}' for ref", name))
                        }
                    }
                    _ => {
                        // For non-ident, evaluate expression (value on stack),
                        // then take the address of the top-of-stack (sp) and push it.
                        self.gen_expr(inner)?; // leaves value on stack at 0(sp)
                                               // copy sp into t0
                        self.emit("    addi t0, sp, 0");
                        self.push_reg("t0");
                        Ok(())
                    }
                }
            }
        }
    }
}

impl Default for CodeGen {
    fn default() -> Self {
        Self::new()
    }
}

pub fn generate_prog_to_string(prog: &Prog) -> Result<String, Error> {
    let mut cg = CodeGen::new();
    cg.generate(prog)
}

/// Convert generated textual ASM into `mips::instrs::Instrs` for execution
pub fn generate_prog_to_instrs(prog: &Prog) -> Result<Instrs, Error> {
    let asm = generate_prog_to_string(prog)?;

    let mut data: HashMap<String, String> = HashMap::new();
    let mut data_order: Vec<String> = Vec::new();
    let mut instr_lines: Vec<String> = Vec::new();
    let mut in_data = false;
    for line in asm.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == ".data" {
            in_data = true;
            continue;
        }
        if line == ".text" {
            in_data = false;
            continue;
        }
        if in_data {
            if let Some((lbl, rest)) = line.split_once(":") {
                if let Some((_, s)) = rest.trim().split_once(".asciiz") {
                    let s = s.trim();
                    let s = s.trim_matches('"');
                    let key = lbl.trim().to_string();
                    data.insert(key.clone(), s.replace("\\n", "\n"));
                    data_order.push(key);
                }
            }
        } else {
            instr_lines.push(line.to_string());
        }
    }

    fn reg_from_str(s: &str) -> Result<Reg, Error> {
        use Reg::*;
        match s {
            "sp" => Ok(sp),
            "fp" => Ok(fp),
            "ra" => Ok(ra),
            "t0" => Ok(t0),
            "t1" => Ok(t1),
            "t2" => Ok(t2),
            "t3" => Ok(t3),
            "t4" => Ok(t4),
            "t5" => Ok(t5),
            "t6" => Ok(t6),
            "t7" => Ok(t7),
            "zero" => Ok(zero),
            _ => Err(format!("unknown register {}", s)),
        }
    }

    // Build Instr vector
    let mut instrs_vec: Vec<Instr> = Vec::new();

    // When main returns with jr ra (ra==0), it jumps to address 0
    // Put halt() at address 0 to terminate program properly
    instrs_vec.push(halt());

    // Initialize stack pointer
    instrs_vec.push(addi(Reg::sp, Reg::zero, 10000));

    // Jump to start program execution
    instrs_vec.push(b_label("main"));

    for line in instr_lines.iter() {
        if line.ends_with(":") {
            let lbl = line.trim_end_matches(":");
            instrs_vec.push(addu(Reg::zero, Reg::zero, Reg::zero).label(lbl));
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        let op = parts[0];
        match op {
            "addi" => {
                let rd = parts[1].trim_end_matches(',');
                let rs = parts[2].trim_end_matches(',');
                let imm: i16 = parts[3].parse().unwrap_or(0);
                if imm < 0 {
                    instrs_vec.push(addiu(reg_from_str(rd)?, reg_from_str(rs)?, imm));
                } else {
                    instrs_vec.push(addi(reg_from_str(rd)?, reg_from_str(rs)?, imm));
                }
            }
            "add" => {
                let rd = parts[1].trim_end_matches(',');
                let rs = parts[2].trim_end_matches(',');
                let rt = parts[3];
                instrs_vec.push(add(reg_from_str(rd)?, reg_from_str(rs)?, reg_from_str(rt)?));
            }
            "sub" => {
                let rd = parts[1].trim_end_matches(',');
                let r1 = parts[2].trim_end_matches(',');
                let r2 = parts[3];
                instrs_vec.push(sub(reg_from_str(rd)?, reg_from_str(r1)?, reg_from_str(r2)?));
            }
            "and" => {
                let rd = parts[1].trim_end_matches(',');
                let r1 = parts[2].trim_end_matches(',');
                let r2 = parts[3];
                instrs_vec.push(and(reg_from_str(rd)?, reg_from_str(r1)?, reg_from_str(r2)?));
            }
            "or" => {
                let rd = parts[1].trim_end_matches(',');
                let r1 = parts[2].trim_end_matches(',');
                let r2 = parts[3];
                instrs_vec.push(or(reg_from_str(rd)?, reg_from_str(r1)?, reg_from_str(r2)?));
            }
            "slt" => {
                let rd = parts[1].trim_end_matches(',');
                let r1 = parts[2].trim_end_matches(',');
                let r2 = parts[3];
                instrs_vec.push(slt(reg_from_str(rd)?, reg_from_str(r1)?, reg_from_str(r2)?));
            }
            "la" => {
                let rd = parts[1].trim_end_matches(',');
                let lbl = parts[2];
                let labid: i16 = if let Some(pos) = data_order.iter().position(|x| x == lbl) {
                    -(((pos as i32) + 1) as i16)
                } else {
                    -1
                };
                instrs_vec.push(addi(reg_from_str(rd)?, Reg::zero, labid));
            }
            "sw" => {
                let reg = parts[1].trim_end_matches(',');
                let memspec = parts[2];
                if let Some((off_s, base_s)) = memspec.split_once('(') {
                    let off: i16 = off_s.parse().unwrap_or(0);
                    let base = base_s.trim_end_matches(')');
                    instrs_vec.push(sw(reg_from_str(reg)?, off, reg_from_str(base)?));
                }
            }
            "lw" => {
                let rd = parts[1].trim_end_matches(',');
                let memspec = parts[2];
                if let Some((off_s, base_s)) = memspec.split_once('(') {
                    let off: i16 = off_s.parse().unwrap_or(0);
                    let base = base_s.trim_end_matches(')');
                    instrs_vec.push(lw(reg_from_str(rd)?, off, reg_from_str(base)?));
                }
            }
            "beq" => {
                let r1 = parts[1].trim_end_matches(',');
                let r2 = parts[2].trim_end_matches(',');
                let lbl = parts[3];
                instrs_vec.push(beq_label(reg_from_str(r1)?, reg_from_str(r2)?, lbl));
            }
            "b" => {
                let lbl = parts[1];
                instrs_vec.push(b_label(lbl));
            }
            "bal" => {
                let lbl = parts[1];
                instrs_vec.push(bal_label(lbl));
            }
            "jr" => {
                let r = parts[1];
                instrs_vec.push(jr(reg_from_str(r)?));
            }
            _ => {
                // Unknown
            }
        }
    }

    Ok(Instrs::new_from_slice(&instrs_vec))
}
