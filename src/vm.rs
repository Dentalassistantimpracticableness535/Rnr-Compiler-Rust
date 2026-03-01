use crate::ast::{BinOp, Block, Expr, FnDeclaration, Literal, Prog, Statement, UnOp};
use crate::common::Eval;
use crate::env::Env;
use crate::error::Error;

#[derive(Debug, Clone)]
pub enum Val {
    Lit(Literal),
    UnInit,
    Mut(Box<Val>),
    Fun(FnDeclaration),
    Overloads(Vec<FnDeclaration>),
    RefVal(Box<Val>),
    RefName(String, bool),
    Intrinsic(crate::intrinsics::Intrinsic),
}

// manual PartialEq impl to avoid comparing function pointer addresses
impl PartialEq for Val {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Val::Lit(a), Val::Lit(b)) => a == b,
            (Val::UnInit, Val::UnInit) => true,
            (Val::Mut(a), Val::Mut(b)) => a == b,
            (Val::Fun(a), Val::Fun(b)) => a == b,
            (Val::Overloads(a), Val::Overloads(b)) => a == b,
            (Val::RefVal(a), Val::RefVal(b)) => a == b,
            (Val::RefName(n1, m1), Val::RefName(n2, m2)) => n1 == n2 && m1 == m2,
            // For intrinsics, avoid pointer comparison; consider them equal if both are Intrinsic
            (Val::Intrinsic(_), Val::Intrinsic(_)) => true,
            _ => false,
        }
    }
}

// Helpers for Val
// Alternatively implement the TryFrom trait
impl Val {
    pub fn get_bool(&self) -> Result<bool, Error> {
        match self {
            Val::Lit(Literal::Bool(b)) => Ok(*b),
            _ => Err(format!("cannot get Bool from {:?}", self)),
        }
    }

    pub fn get_int(&self) -> Result<i32, Error> {
        match self {
            Val::Lit(Literal::Int(i)) => Ok(*i),
            _ => Err(format!("cannot get integer from {:?}", self)),
        }
    }
}

// Helper for Op
impl BinOp {
    // Evaluate operator to literal
    pub fn eval(&self, left: Val, right: Val) -> Result<Val, Error> {
        match self {
            BinOp::Add => {
                let l = left.get_int()?;
                let r = right.get_int()?;
                Ok(Val::Lit(Literal::Int(l + r)))
            }
            BinOp::Sub => {
                let l = left.get_int()?;
                let r = right.get_int()?;
                Ok(Val::Lit(Literal::Int(l - r)))
            }
            BinOp::Mul => {
                let l = left.get_int()?;
                let r = right.get_int()?;
                Ok(Val::Lit(Literal::Int(l * r)))
            }
            BinOp::Div => {
                let l = left.get_int()?;
                let r = right.get_int()?;
                if r == 0 {
                    return Err("division by zero".to_string());
                }
                Ok(Val::Lit(Literal::Int(l / r)))
            }
            // And/Or are handled via short-circuit evaluation in eval_expr;
            // they should never reach BinOp::eval with both operands evaluated.
            BinOp::And | BinOp::Or => {
                unreachable!("And/Or should be short-circuited in eval_expr")
            }
            BinOp::Eq => match (left, right) {
                (Val::Lit(Literal::Int(a)), Val::Lit(Literal::Int(b))) => {
                    Ok(Val::Lit(Literal::Bool(a == b)))
                }
                (Val::Lit(Literal::Bool(a)), Val::Lit(Literal::Bool(b))) => {
                    Ok(Val::Lit(Literal::Bool(a == b)))
                }
                (Val::Lit(Literal::Unit), Val::Lit(Literal::Unit)) => {
                    Ok(Val::Lit(Literal::Bool(true)))
                }
                (Val::Lit(Literal::String(a)), Val::Lit(Literal::String(b))) => {
                    Ok(Val::Lit(Literal::Bool(a == b)))
                }
                (l, r) => Err(format!("cannot compare values {:?} and {:?}", l, r)),
            },
            BinOp::Lt => {
                let l = left.get_int()?;
                let r = right.get_int()?;
                Ok(Val::Lit(Literal::Bool(l < r)))
            }
            BinOp::Gt => {
                let l = left.get_int()?;
                let r = right.get_int()?;
                Ok(Val::Lit(Literal::Bool(l > r)))
            }
        }
    }
}

impl Eval<Val> for Expr {
    fn eval(&self) -> Result<Val, Error> {
        let mut vm = VM::new();
        vm.eval_expr(self)
    }
}

impl Eval<Val> for Block {
    fn eval(&self) -> Result<Val, Error> {
        let mut vm = VM::new();
        vm.eval_block(self)
    }
}

impl Eval<Val> for Prog {
    fn eval(&self) -> Result<Val, Error> {
        let mut vm = VM::new();

        // register all toplevel functions (so they can call each other)
        for f in &self.0 {
            vm.env.insert_overload(f.id.clone(), f.clone());
        }

        // find main + evaluate its body
        let main_fn = self
            .0
            .iter()
            .find(|f| f.id == "main")
            .ok_or("main function not found")?;
        vm.eval_block(&main_fn.body)
    }
}

impl From<Literal> for Val {
    fn from(lit: Literal) -> Self {
        Val::Lit(lit)
    }
}

impl From<Val> for Literal {
    fn from(val: Val) -> Self {
        match val {
            Val::Lit(lit) => lit,
            _ => panic!("cannot convert to Literal from {:?}", val),
        }
    }
}

impl From<i32> for Val {
    fn from(val: i32) -> Self {
        Val::Lit(val.into())
    }
}

impl From<bool> for Val {
    fn from(val: bool) -> Self {
        Val::Lit(val.into())
    }
}

impl From<()> for Val {
    fn from(val: ()) -> Self {
        Val::Lit(val.into())
    }
}

impl From<Val> for i32 {
    fn from(val: Val) -> Self {
        match val {
            Val::Lit(Literal::Int(i)) => i,
            _ => panic!("cannot get int from {:?}", val),
        }
    }
}

impl From<Literal> for bool {
    fn from(lit: Literal) -> Self {
        match lit {
            Literal::Bool(b) => b,
            _ => panic!("cannot get bool from {:?}", lit),
        }
    }
}
// VM implementation
pub struct VM {
    env: Env,
}

impl VM {
    #![allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut vm = VM { env: Env::new() };

        // register builtin intrinsics so they are visible for overload resolution
        let (decl, intrinsic) = crate::intrinsics::vm_println();
        vm.env.insert_overload(decl.id.clone(), decl.clone());
        vm.env.insert(decl.id.clone(), Val::Intrinsic(intrinsic));

        vm
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Val, Error> {
        match expr {
            Expr::Lit(lit) => Ok(Val::from(lit.clone())),
            Expr::Par(expr) => self.eval_expr(expr),
            Expr::Ref(inner, mutable) => {
                // If ref of an ident, create alias
                if let Expr::Ident(name) = &**inner {
                    // ensure the variable exists
                    let stored = self.env.lookup(name)?;
                    // If asking for a mutable ref, ensure target binding is mutable
                    if mutable.0 {
                        match stored {
                            Val::Mut(_) => Ok(Val::RefName(name.clone(), true)),
                            _ => Err(format!(
                                "cannot take mutable reference to non-mutable '{}'",
                                name
                            )),
                        }
                    } else {
                        Ok(Val::RefName(name.clone(), false))
                    }
                } else {
                    // For non-ident, create a boxed reference value (by-value reference)
                    let inner_val = self.eval_expr(inner)?;
                    let boxed = if mutable.0 {
                        Val::Mut(Box::new(inner_val))
                    } else {
                        inner_val
                    };
                    Ok(Val::RefVal(Box::new(boxed)))
                }
            }
            Expr::Ident(id) => {
                let val = self.env.lookup(id)?;
                // check if mutable
                match val {
                    Val::Mut(inner) => Ok(*inner),
                    _ => Ok(val),
                }
            }
            Expr::BinOp(op, left, right) => match op {
                BinOp::And => {
                    let l = self.eval_expr(left)?;
                    if !l.get_bool()? {
                        Ok(Val::Lit(Literal::Bool(false)))
                    } else {
                        let r = self.eval_expr(right)?;
                        Ok(Val::Lit(Literal::Bool(r.get_bool()?)))
                    }
                }
                BinOp::Or => {
                    let l = self.eval_expr(left)?;
                    if l.get_bool()? {
                        Ok(Val::Lit(Literal::Bool(true)))
                    } else {
                        let r = self.eval_expr(right)?;
                        Ok(Val::Lit(Literal::Bool(r.get_bool()?)))
                    }
                }
                _ => {
                    let left: Val = self.eval_expr(left)?;
                    let right: Val = self.eval_expr(right)?;
                    op.eval(left, right)
                }
            },
            Expr::UnOp(op, expr) => {
                let val = self.eval_expr(expr)?;
                match op {
                    UnOp::Neg => {
                        let i = val.get_int()?;
                        Ok(Val::Lit(Literal::Int(-i)))
                    }
                    UnOp::Bang => {
                        let b = val.get_bool()?;
                        Ok(Val::Lit(Literal::Bool(!b)))
                    }
                    UnOp::Deref => {
                        // dereference
                        match val {
                            Val::RefVal(boxed) => match *boxed {
                                Val::Mut(inner) => Ok(*inner),
                                other => Ok(other),
                            },
                            Val::RefName(name, _ref_mut) => {
                                // lookup the target variable and return its value
                                let stored = self.env.lookup(&name)?;
                                match stored {
                                    Val::Mut(inner) => Ok(*inner),
                                    other => Ok(other),
                                }
                            }

                            // manage error if it's not a reference
                            _ => Err("cannot dereference non-reference".to_string()),
                        }
                    }
                }
            }
            Expr::IfThenElse(cond, then_block, else_block) => {
                let cond_val = self.eval_expr(cond)?;
                if cond_val.get_bool()? {
                    self.eval_block(then_block)
                // if else block is present
                } else if let Some(else_blk) = else_block {
                    self.eval_block(else_blk)
                } else {
                    Ok(Val::Lit(Literal::Unit))
                }
            }
            Expr::Block(block) => self.eval_block(block),
            Expr::Call(name, args) => {
                let stored = self.env.lookup(name)?;
                match stored {
                    Val::Intrinsic(func) => {
                        // evaluate args to runtime Vals
                        let mut evaluated_args: Vec<Val> = Vec::new();
                        for a in &args.0 {
                            evaluated_args.push(self.eval_expr(a)?);
                        }
                        // resolve any RefName aliases to their stored values
                        for v in evaluated_args.iter_mut() {
                            if let Val::RefName(name, _m) = v.clone() {
                                let stored = self.env.lookup(&name)?;
                                *v = stored;
                            }
                        }
                        // call intrinsic
                        let res = func(evaluated_args)?;
                        Ok(res)
                    }
                    Val::Fun(func_decl) => {
                        // args nb check
                        let expected = func_decl.parameters.0.len();
                        let got = args.0.len();
                        if expected != got {
                            return Err(format!(
                                "error, function '{}' expected {} args but got {}",
                                name, expected, got
                            ));
                        }

                        // Evaluate args and enter new scope
                        let mut evaluated_args: Vec<Val> = Vec::new();
                        for a in &args.0 {
                            evaluated_args.push(self.eval_expr(a)?);
                        }
                        self.env.push_scope();

                        // Bind param in this new scope
                        for (param, arg_val) in func_decl
                            .parameters
                            .0
                            .iter()
                            .zip(evaluated_args.into_iter())
                        {
                            let stored_val = if param.mutable.0 {
                                Val::Mut(Box::new(arg_val))
                            } else {
                                arg_val
                            };
                            self.env.insert(param.id.clone(), stored_val);
                        }

                        // evaluate function body
                        let result = self.eval_block(&func_decl.body);

                        self.env.pop_scope()?;

                        result
                    }
                    Val::Overloads(vec) => {
                        // evaluate argument values to get runtime types
                        let mut evaluated_args: Vec<Val> = Vec::new();
                        for a in &args.0 {
                            evaluated_args.push(self.eval_expr(a)?);
                        }

                        // build runtime arg types (simple mapping from Val to Type-like enum)
                        fn val_type(v: &Val) -> Result<String, Error> {
                            match v {
                                Val::Lit(Literal::Int(_)) => Ok("I32".to_string()),
                                Val::Lit(Literal::Bool(_)) => Ok("Bool".to_string()),
                                Val::Lit(Literal::String(_)) => Ok("String".to_string()),
                                Val::Lit(Literal::Unit) => Ok("Unit".to_string()),
                                Val::Mut(inner) => val_type(inner),
                                Val::RefVal(boxed) => val_type(boxed),
                                Val::RefName(_, _) => Ok("Ref".to_string()),
                                _ => Err("cannot determine runtime arg type".to_string()),
                            }
                        }

                        let arg_types: Result<Vec<String>, Error> =
                            evaluated_args.iter().map(val_type).collect();
                        let arg_types = arg_types?;

                        // find matching overload by exact parameter types
                        let mut candidates: Vec<FnDeclaration> = Vec::new();
                        for f in vec.into_iter() {
                            if f.parameters.0.len() != arg_types.len() {
                                continue;
                            }
                            let mut ok = true;
                            for (p, at) in f.parameters.0.iter().zip(arg_types.iter()) {
                                let pty = match p.ty {
                                    crate::ast::Type::I32 => "I32",
                                    crate::ast::Type::Bool => "Bool",
                                    crate::ast::Type::String => "String",
                                    crate::ast::Type::Unit => "Unit",
                                    crate::ast::Type::Ref(_, _) => "Ref",
                                };
                                if pty != at && pty != "Ref" {
                                    ok = false;
                                    break;
                                }
                            }
                            if ok {
                                candidates.push(f.clone());
                            }
                        }
                        if candidates.is_empty() {
                            return Err(format!(
                                "no matching overload for '{}' with runtime arg types {:?}",
                                name, arg_types
                            ));
                        }
                        if candidates.len() > 1 {
                            return Err(format!(
                                "ambiguous call to '{}' with runtime arg types {:?}",
                                name, arg_types
                            ));
                        }

                        // call the single candidate: reuse the same code as for Val::Fun
                        let func_decl = &candidates[0];

                        // enter new scope and bind evaluated args
                        self.env.push_scope();
                        for (param, arg_val) in func_decl
                            .parameters
                            .0
                            .iter()
                            .zip(evaluated_args.into_iter())
                        {
                            let stored_val = if param.mutable.0 {
                                Val::Mut(Box::new(arg_val))
                            } else {
                                arg_val
                            };
                            self.env.insert(param.id.clone(), stored_val);
                        }

                        let result = self.eval_block(&func_decl.body);
                        self.env.pop_scope()?;
                        result
                    }
                    other => Err(format!("error, it not a function '{:?}'", other)),
                }
            }
        }
    }

    pub fn eval_stmt(&mut self, stmt: &Statement) -> Result<Val, Error> {
        match stmt {
            Statement::Let(mutable, name, _ty, init) => {
                // init
                let val = if let Some(expr) = init {
                    self.eval_expr(expr)?
                } else {
                    Val::UnInit
                };

                // mut
                let final_val = if mutable.0 {
                    Val::Mut(Box::new(val))
                } else {
                    val
                };

                // insert in the current scope
                self.env.insert(name.clone(), final_val);
                Ok(Val::Lit(Literal::Unit))
            }
            Statement::Assign(lhs, rhs) => {
                // lhs as an Ident
                if let Expr::Ident(name) = lhs {
                    let val = self.eval_expr(rhs)?;
                    let current = self.env.lookup(name)?;
                    match current {
                        // variable declared mutable → replace inner value
                        Val::Mut(_) => {
                            self.env.update(name, Val::Mut(Box::new(val)))?;
                        }
                        // variable declared but uninitialized → allow first (late) initialization
                        Val::UnInit => {
                            self.env.update(name, val)?;
                        }
                        // otherwise it's an initialized immutable value: forbid assignment
                        _ => {
                            return Err(format!("assignment to immutable variable '{}'", name));
                        }
                    }
                    Ok(Val::Lit(Literal::Unit))
                // lhs with shape `*a = ...` where `a` is a reference
                } else if let Expr::UnOp(UnOp::Deref, inner) = lhs {
                    if let Expr::Ident(name) = &**inner {
                        let new_val = self.eval_expr(rhs)?;
                        // look up the variable that should hold the reference
                        let stored = self.env.lookup(name)?;
                        match stored {
                            // r is a reference value stored by-value
                            Val::RefVal(boxed) => match *boxed {
                                Val::Mut(_) => {
                                    // replace inner mutable value inside the reference
                                    let replacement =
                                        Val::RefVal(Box::new(Val::Mut(Box::new(new_val))));
                                    self.env.update(name, replacement)?;
                                    Ok(Val::Lit(Literal::Unit))
                                }
                                _ => Err(format!(
                                    "can't assign through immutable reference '{}'",
                                    name
                                )),
                            },
                            // r is a name-based reference aliasing another variable
                            Val::RefName(target, ref_mut) => {
                                if !ref_mut {
                                    return Err(format!(
                                        "can't assign through immutable reference '{}'",
                                        name
                                    ));
                                }
                                // look up the target variable
                                let target_stored = self.env.lookup(&target)?;
                                match target_stored {
                                    Val::Mut(_) => {
                                        self.env
                                            .update(&target, Val::Mut(Box::new(new_val)))?;
                                        Ok(Val::Lit(Literal::Unit))
                                    }
                                    _ => Err(format!(
                                        "can't assign through immutable reference '{}'",
                                        target
                                    )),
                                }
                            }
                            _ => Err(format!("variable '{}' isn't reference", name)),
                        }
                    } else {
                        Err("assignment to deref of non-identifier not yet supported".to_string())
                    }
                } else {
                    Err("assignment to non-identifier not yet supported".to_string())
                }
            }
            Statement::While(cond, body) => {
                loop {
                    let cond_val = self.eval_expr(cond)?;
                    if !cond_val.get_bool()? {
                        break;
                    }
                    self.eval_block(body)?;
                }
                Ok(Val::Lit(Literal::Unit))
            }
            Statement::Expr(expr) => self.eval_expr(expr),
            Statement::Fn(_fn_decl) => Ok(Val::Lit(Literal::Unit)),
        }
    }

    pub fn eval_block(&mut self, block: &Block) -> Result<Val, Error> {
        self.env.push_scope();
        // hoist all funct declarations so they can be called before def
        for stmt in &block.statements {
            if let Statement::Fn(fdecl) = stmt {
                // insert as overloads-aware
                self.env.insert_overload(fdecl.id.clone(), fdecl.clone());
            }
        }
        let mut last_val = Val::Lit(Literal::Unit);
        for stmt in &block.statements {
            last_val = self.eval_stmt(stmt)?;
        }
        self.env.pop_scope()?;

        // If ends with ";" return Unit, else return last value
        if block.semi {
            Ok(Val::Lit(Literal::Unit))
        } else {
            Ok(last_val)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Val;
    use crate::ast::Literal;
    use crate::ast::{Block, Expr, Prog};
    use crate::common::{parse_test, Eval};
    use crate::parse::parse;

    #[test]
    fn test_expr() {
        let v = parse_test::<Expr, Val>(
            "        
        1 + 1
    ",
        );

        assert_eq!(v.unwrap().get_int().unwrap(), 2);
    }

    #[test]
    fn test_bool() {
        let v = parse_test::<Block, Val>(
            "
    {
        let a = true && false;
        a
    }
    ",
        );

        assert!(!v.unwrap().get_bool().unwrap());
    }

    #[test]
    fn test_bool_bang2() {
        let v = parse_test::<Block, Val>(
            "
    {
        let a = (!true) && false;
        a
    }
    ",
        );

        assert!(!v.unwrap().get_bool().unwrap());
    }

    #[test]
    fn test_bool_bang() {
        let v = parse_test::<Block, Val>(
            "
    {
        let a = true && !false;
        a
    }
    ",
        );

        assert!(v.unwrap().get_bool().unwrap());
    }

    #[test]
    fn test_block_let() {
        let v = parse_test::<Block, Val>(
            "
    {
        let a: i32 = 1;
        let b: i32 = 2;

        a + b
    }",
        );
        assert_eq!(v.unwrap().get_int().unwrap(), 3);
    }

    #[test]
    fn test_block_let_shadow() {
        let v = parse_test::<Block, Val>(
            "
    {
        let a: i32 = 1;
        let b: i32 = 2;
        let a: i32 = 3;
        let b: i32 = 4;

        a + b
    }",
        );
        assert_eq!(v.unwrap().get_int().unwrap(), 7);
    }

    #[test]
    fn test_local_block() {
        let v = parse_test::<Block, Val>(
            "
    {
        let a = 1;
        let b = {
            let b = a;
            b * 2
        };

        b
    }
    ",
        );

        assert_eq!(v.unwrap().get_int().unwrap(), 2);
    }

    #[test]
    fn test_block_assign() {
        let v = parse_test::<Block, Val>(
            "
    {
        let mut a: i32 = 1;
        a = a + 2;
        a
    }",
        );
        assert_eq!(v.unwrap().get_int().unwrap(), 3);
    }

    #[test]
    fn test_assignment_to_immutable_errors() {
        // runtime should reject assignment to non-mut variable
        let blk: Block = parse("{ let a: i32 = 1; a = 2; }");
        let r = crate::common::Eval::<Val>::eval(&blk);
        assert!(
            r.is_err(),
            "expected runtime error for assignment to immutable var, got {:?}",
            r
        );
    }

    #[test]
    fn test_assign_through_deref_mutable() {
        let v = parse_test::<Block, Val>(
            "
    {
        let mut x: i32 = 1;
        let r = &mut x;
        *r = 2;
        *r
    }
    ",
        );
        assert_eq!(v.unwrap().get_int().unwrap(), 2);
    }

    #[test]
    fn test_regress_assign_through_ref_aliasing() {
        let v = parse_test::<Block, Val>(
            "
    {
        let mut x: i32 = 1;
        let r = &mut x;
        *r = 5;
        x
    }
    ",
        );
        assert_eq!(v.unwrap().get_int().unwrap(), 5);
    }

    #[test]
    fn test_assign_through_deref_immutable_ref_errors() {
        // reference is immutable, assignment through deref must fail
        let blk: Block = parse("{ let mut x: i32 = 1; let r = &x; *r = 2; }");
        let r = crate::common::Eval::<Val>::eval(&blk);
        assert!(
            r.is_err(),
            "expected runtime error assigning through immutable ref, got {:?}",
            r
        );
    }

    #[test]
    fn test_expr_if_then_else() {
        let v = parse_test::<Block, Val>(
            "
    {
        let mut a: i32 = 1;
        a = if a > 0 { a + 1 } else { a - 2 };
        a
    }",
        );

        assert_eq!(v.unwrap().get_int().unwrap(), 2);
    }

    #[test]
    fn test_while() {
        let v = parse_test::<Block, Val>(
            "
    {
        let mut a = 2;
        let mut b = 0;
        while a > 0 {
            a = a - 1;
            b = b + 1;
        }
        b
    }
    ",
        );

        assert_eq!(v.unwrap().get_int().unwrap(), 2);
    }

    #[test]
    fn test_prog() {
        let v = parse_test::<Prog, Val>(
            "
    fn main() {
        let a = 1;
        a
    }
    ",
        );

        assert_eq!(v.unwrap().get_int().unwrap(), 1);
    }

    #[test]
    fn test_local_fn() {
        let v = parse_test::<Prog, Val>(
            "
    fn main() {
        fn f(i: i32, j: i32) -> i32 {
            i + j
        }
        let a = f(1, 2);
        println!(\"a = {} and another a = {}\", a, a);
    }
    ",
        );

        assert_eq!(v.unwrap(), Val::Lit(Literal::Unit));
    }

    #[test]
    fn test_check_if_then_else_shadowing() {
        let v = parse_test::<Block, Val>(
            "
        {
            let a: i32 = 1 + 2; // a == 3
            let mut a: i32 = 2 + a; // a == 5
            if true {
                a = a - 1;      // outer a == 4
                let mut a: i32 = 0; // inner a == 0
                a = a + 1;      // inner a == 1
            } else {
                a = a - 1;
            };
            a   // a == 4
        }
        ",
        );

        assert_eq!(v.unwrap().get_int().unwrap(), 4);
    }
}
