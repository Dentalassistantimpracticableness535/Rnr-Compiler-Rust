fn main() {
    use mips::{asm::*, instr::Instr, instrs::Instrs, rf::Reg};

    let instrs: Vec<Instr> = vec![
        // Initialize SP
        addi(Reg::sp, Reg::zero, 10000),
        // Call main (bal sets ra to next instruction = halt)
        bal_label("main"),
        // Halt
        halt(),
        // main: addi t0, zero, 42 ; jr ra
        addu(Reg::zero, Reg::zero, Reg::zero).label("main"),
        addi(Reg::t0, Reg::zero, 42),
        jr(Reg::ra),
    ];

    println!("Created {} instructions", instrs.len());
    for (i, instr) in instrs.iter().enumerate() {
        println!("  [{}] {:?}", i, instr);
    }

    let instrs_obj = Instrs::new_from_slice(&instrs);
    let mut m = mips::vm::Mips::new(instrs_obj);

    println!("\nRunning mips VM...");
    match m.run() {
        Ok(_) | Err(mips::error::Error::Halt) => {
            let ret = m.rf.get(Reg::t0) as i32;
            println!("SUCCESS — t0 = {}", ret);
        }
        Err(e) => println!("FAILED: {:?}", e),
    }
}
