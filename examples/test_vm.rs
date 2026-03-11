fn main() {
    use mips::{asm::*, instr::Instr, instrs::Instrs, rf::Reg};

    let instrs: Vec<Instr> = vec![
        // Initialize SP
        addi(Reg::sp, Reg::zero, 10000),
        // Jump to main label
        b_label("main"),
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
        Ok(_) => println!("SUCCESS"),
        Err(e) => println!("FAILED: {:?}", e),
    }
}
