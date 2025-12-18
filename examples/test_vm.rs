fn main() {
    use mips::{asm::*, instr::Instr, instrs::Instrs, rf::Reg};

    let mut instrs: Vec<Instr> = vec![];

    // Initialize SP
    instrs.push(addi(Reg::sp, Reg::zero, 10000));

    // Jump to main label
    instrs.push(b_label("main"));

    // Halt
    instrs.push(halt());

    // Main: addi t0, zero, 42 ; jr ra
    instrs.push(addu(Reg::zero, Reg::zero, Reg::zero).label("main"));
    instrs.push(addi(Reg::t0, Reg::zero, 42));
    instrs.push(jr(Reg::ra));

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
