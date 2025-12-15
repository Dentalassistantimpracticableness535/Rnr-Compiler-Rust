use crate::error::Error;

pub trait Eval<T: Clone> {
    fn eval(&self) -> Result<T, Error>
    where
        T: Clone;
}

pub fn parse<T1, T2>(s: &str) -> T1
where
    T1: syn::parse::Parse + std::fmt::Display,
    T2: Clone,
{
    let ts: proc_macro2::TokenStream = s.parse().unwrap();
    let r: T1 = syn::parse2(ts).unwrap();
    println!("{}", r);
    r
}

pub fn parse_test<T1, T2>(s: &str) -> Result<T2, Error>
where
    T1: syn::parse::Parse + std::fmt::Display + Eval<T2>,
    T2: std::fmt::Debug + Clone,
{
    let bl = parse::<T1, T2>(s);
    let v = bl.eval()?;
    println!("\nreturn {:?}", v);
    Ok(v)
}

/// Codegen + run helper used by tests: compile source to asm, run a simple
/// assembler/VM and return a small test-machine exposing register reads.
pub struct TestRF {
    regs: std::collections::HashMap<String, i32>,
}

impl TestRF {
    pub fn get(&self, r: mips::rf::Reg) -> i32 {
        let name = match r {
            mips::rf::Reg::sp => "sp",
            mips::rf::Reg::fp => "fp",
            mips::rf::Reg::ra => "ra",
            mips::rf::Reg::t0 => "t0",
            mips::rf::Reg::t1 => "t1",
            mips::rf::Reg::t2 => "t2",
            mips::rf::Reg::t3 => "t3",
            mips::rf::Reg::t4 => "t4",
            mips::rf::Reg::t5 => "t5",
            mips::rf::Reg::t6 => "t6",
            mips::rf::Reg::t7 => "t7",
            _ => "t0",
        };
        *self.regs.get(name).unwrap_or(&0)
    }
}

pub struct TestMachine {
    pub rf: TestRF,
    pub output: String,
}

pub fn codegen_test<T>(_src: &str) -> Result<TestMachine, Error>
where
    T: syn::parse::Parse,
{
    // We only use Prog in practice; parse generically and attempt to downcast.
    // Parse as a Prog and generate assembly.
    let prog: crate::ast::Prog = crate::parse::parse(_src);
    let asm = crate::codegen::generate_prog_to_string(&prog).map_err(|e| {
        // convert to Error if necessary
        crate::error::Error::from(format!("codegen error: {}", e))
    })?;
    eprintln!("--- Generated ASM ---\n{}\n--- End ASM ---", asm);

    // Simple assembler/runner (adapted from asm_runner::run_asm)
    let mut data: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut data_order: Vec<String> = Vec::new();
    let mut instrs: Vec<String> = Vec::new();
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
            instrs.push(line.to_string());
        }
    }

    let mut label_map: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (i, line) in instrs.iter().enumerate() {
        if line.ends_with(":") {
            let lbl = line.trim_end_matches(":").to_string();
            label_map.insert(lbl, i);
        }
    }

    let mut regs: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
    regs.insert("sp".to_string(), 1_000_000);
    regs.insert("fp".to_string(), 1_000_000);
    regs.insert("ra".to_string(), 0);
    regs.insert("t0".to_string(), 0);
    regs.insert("t1".to_string(), 0);
    regs.insert("t2".to_string(), 0);
    regs.insert("t3".to_string(), 0);
    regs.insert("t4".to_string(), 0);
    regs.insert("t5".to_string(), 0);
    regs.insert("t6".to_string(), 0);
    regs.insert("t7".to_string(), 0);

    let mut mem: std::collections::HashMap<i32, i32> = std::collections::HashMap::new();

    let mut ip: usize = 0;
    let mut steps: usize = 0;
    let max_steps: usize = std::env::var("CODEGEN_TEST_MAX_STEPS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(200_000); // safety cap (smaller by default for faster diagnostics)
    eprintln!("--- Begin simulation (max_steps={}) ---", max_steps);
    // Start capturing println output
    crate::intrinsics::start_capture();
    while ip < instrs.len() {
        steps += 1;
        if steps > max_steps {
            eprintln!("timeout at step {} ip {}", steps, ip);
            // dump a short window of instructions around ip for diagnosis
            let start = if ip >= 10 { ip - 10 } else { 0 };
            let end = std::cmp::min(instrs.len(), ip + 10);
            eprintln!("--- instr window {}..{} ---", start, end);
            for i in start..end {
                eprintln!("{:04}: {}", i, instrs[i]);
            }
            return Err(format!("execution timeout after {} steps", steps));
        }
        if steps % 10000 == 0 {
            if let Some(instr) = instrs.get(ip) {
                eprintln!("step {} ip {} instr={}", steps, ip, instr);
            } else {
                eprintln!("step {} ip {} (no instr)", steps, ip);
            }
        }
        let line = instrs[ip].trim();
        if line.ends_with(":") {
            ip += 1;
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0] {
            "addi" => {
                let rd = parts[1].trim_end_matches(',');
                let rs = parts[2].trim_end_matches(',');
                let imm: i32 = parts[3].parse().unwrap_or(0);
                let v = *regs.get(rs).unwrap_or(&0) + imm;
                regs.insert(rd.to_string(), v);
            }
            "add" | "sub" | "mul" | "div" | "and" | "or" => {
                let rd = parts[1].trim_end_matches(',');
                let r1 = parts[2].trim_end_matches(',');
                let r2 = parts[3];
                let v1 = *regs.get(r1).unwrap_or(&0);
                let v2 = *regs.get(r2).unwrap_or(&0);
                let res = match parts[0] {
                    "add" => v1.wrapping_add(v2),
                    "sub" => v1.wrapping_sub(v2),
                    "mul" => v1.wrapping_mul(v2),
                    "div" => {
                        if v2 == 0 {
                            0
                        } else {
                            v1 / v2
                        }
                    }
                    "and" => {
                        if v1 != 0 && v2 != 0 {
                            1
                        } else {
                            0
                        }
                    }
                    "or" => {
                        if v1 != 0 || v2 != 0 {
                            1
                        } else {
                            0
                        }
                    }
                    _ => 0,
                };
                regs.insert(rd.to_string(), res);
            }
            "slt" => {
                let rd = parts[1].trim_end_matches(',');
                let r1 = parts[2].trim_end_matches(',');
                let r2 = parts[3];
                let v1 = *regs.get(r1).unwrap_or(&0);
                let v2 = *regs.get(r2).unwrap_or(&0);
                regs.insert(rd.to_string(), if v1 < v2 { 1 } else { 0 });
            }
            "la" => {
                let rd = parts[1].trim_end_matches(',');
                let lbl = parts[2];
                let labid = if let Some(pos) = data_order.iter().position(|x| x == lbl) {
                    -((pos as i32) + 1)
                } else {
                    -1
                };
                regs.insert(rd.to_string(), labid);
            }
            "sw" => {
                let reg = parts[1].trim_end_matches(',');
                let memspec = parts[2];
                if let Some((off_s, base_s)) = memspec.split_once('(') {
                    let off: i32 = off_s.parse().unwrap_or(0);
                    let base = base_s.trim_end_matches(')');
                    let base_v = *regs.get(base).unwrap_or(&0);
                    let addr = base_v + off;
                    let val = *regs.get(reg).unwrap_or(&0);
                    mem.insert(addr, val);
                }
            }
            "lw" => {
                let rd = parts[1].trim_end_matches(',');
                let memspec = parts[2];
                if let Some((off_s, base_s)) = memspec.split_once('(') {
                    let off: i32 = off_s.parse().unwrap_or(0);
                    let base = base_s.trim_end_matches(')');
                    let base_v = *regs.get(base).unwrap_or(&0);
                    let addr = base_v + off;
                    let val = *mem.get(&addr).unwrap_or(&0);
                    regs.insert(rd.to_string(), val);
                }
            }
            "beq" => {
                let r1 = parts[1].trim_end_matches(',');
                let r2 = parts[2].trim_end_matches(',');
                let lbl = parts[3];
                if *regs.get(r1).unwrap_or(&0) == *regs.get(r2).unwrap_or(&0) {
                    ip = *label_map.get(lbl).ok_or("label not found")?;
                    continue;
                }
            }
            "b" => {
                let lbl = parts[1];
                ip = *label_map.get(lbl).ok_or("label not found")?;
                continue;
            }
            "bal" => {
                let lbl = parts[1];
                if std::env::var("CODEGEN_DEBUG_BAL").is_ok() {
                    eprintln!(
                        "-- BAL debug (ip={}): sp={} fp={} ra={} -> {}",
                        ip,
                        regs.get("sp").unwrap_or(&0),
                        regs.get("fp").unwrap_or(&0),
                        regs.get("ra").unwrap_or(&0),
                        lbl
                    );
                }
                if lbl == "println" {
                    use crate::ast::Literal;
                    use crate::vm::Val;
                    // Nb of args is passed in register t2
                    let argc = *regs.get("t2").unwrap_or(&0) as usize;
                    let mut v_args: Vec<Val> = Vec::new();
                    let sp = *regs.get("sp").unwrap_or(&0);
                    for i in 0..argc {
                        let addr = sp + (i as i32) * 4;
                        let val = *mem.get(&addr).unwrap_or(&0);
                        if val < 0 {
                            let idx = (-val - 1) as usize;
                            if let Some(lbln) = data_order.get(idx) {
                                if let Some(s) = data.get(lbln) {
                                    v_args.push(Val::Lit(Literal::String(s.clone())));
                                } else {
                                    v_args.push(Val::Lit(Literal::Unit));
                                }
                            } else {
                                v_args.push(Val::Lit(Literal::Unit));
                            }
                        } else {
                            v_args.push(Val::Lit(Literal::Int(val)));
                        }
                    }
                    let (_decl, intrinsic) = crate::intrinsics::vm_println();
                    intrinsic(v_args)?;
                }
                regs.insert("ra".to_string(), (ip as i32) + 1);
                ip = *label_map.get(lbl).ok_or("label not found")?;
                continue;
            }
            "jr" => {
                let r = parts[1];
                let target = *regs.get(r).unwrap_or(&0);
                if std::env::var("CODEGEN_DEBUG_JR").is_ok() {
                    eprintln!(
                        "-- JR debug (ip={}): sp={} fp={} ra={} t0={} t1={}",
                        ip,
                        regs.get("sp").unwrap_or(&0),
                        regs.get("fp").unwrap_or(&0),
                        regs.get("ra").unwrap_or(&0),
                        regs.get("t0").unwrap_or(&0),
                        regs.get("t1").unwrap_or(&0)
                    );
                    let spv = *regs.get("sp").unwrap_or(&0);
                    for off in (-16..=16).step_by(4) {
                        let addr = spv + off;
                        let val = mem.get(&addr).unwrap_or(&0);
                        eprintln!(" mem[{}] = {}", addr, val);
                    }
                }
                // If target is 0, assume this is the return from main (ra was
                // initialized to 0) and terminate the simulation successfully.
                if target == 0 {
                    break;
                }
                if target < 0 {
                    let idx = (-target - 1) as usize;
                    ip = idx;
                    continue;
                } else {
                    ip = target as usize;
                    continue;
                }
            }
            _ => {}
        }
        ip += 1;
    }

    // Retrieve captured output (if any) and return it in the TestMachine
    let output = crate::intrinsics::take_capture().unwrap_or_default();
    let rf = TestRF { regs };
    Ok(TestMachine { rf, output })
}
