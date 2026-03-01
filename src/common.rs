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
    // parse as a Prog and generate assembly text (for debug output)
    let prog: crate::ast::Prog = crate::parse::parse(_src);
    let asm = crate::codegen::generate_prog_to_string(&prog).map_err(|e| {
        crate::error::Error::from(format!("codegen error: {}", e))
    })?;
    eprintln!("--- Generated ASM ---\n{}\n--- End ASM ---", asm);

    // convert to MIPS instructions and run 
    let instrs = crate::codegen::generate_prog_to_instrs(&prog)?;
    let mut mips_vm = mips::vm::Mips::new(instrs);
    match mips_vm.run() {
        Err(mips::error::Error::Halt) => {} // normal termination
        Ok(_) => {}                          // ran off the end (also OK)
        Err(e) => return Err(format!("MIPS VM error: {:?}", e)),
    }

    // extract register values
    let mut regs = std::collections::HashMap::new();
    for (name, reg) in [
        ("sp", mips::rf::Reg::sp),
        ("fp", mips::rf::Reg::fp),
        ("ra", mips::rf::Reg::ra),
        ("t0", mips::rf::Reg::t0),
        ("t1", mips::rf::Reg::t1),
        ("t2", mips::rf::Reg::t2),
        ("t3", mips::rf::Reg::t3),
        ("t4", mips::rf::Reg::t4),
        ("t5", mips::rf::Reg::t5),
        ("t6", mips::rf::Reg::t6),
        ("t7", mips::rf::Reg::t7),
    ] {
        regs.insert(name.to_string(), mips_vm.rf.get(reg) as i32);
    }

    let rf = TestRF { regs };
    Ok(TestMachine {
        rf,
        output: String::new(),
    })
}
