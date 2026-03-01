use rnr::ast::*;
use rnr::codegen::generate_prog_to_instrs;
use rnr::codegen::generate_prog_to_string;
use mips::vm::Mips;

fn main() {
    let f = FnDeclaration {
        id: "main".to_string(),
        parameters: Parameters(vec![]),
        ty: None,
        body: Block::new(
            vec![Statement::Expr(Expr::Call(
                "println!".to_string(),
                Arguments(vec![Expr::Lit(Literal::String("hello".to_string()))]),
            ))],
            true,
        ),
    };

    let prog = Prog(vec![f]);
    let asm = generate_prog_to_string(&prog).expect("codegen");
    println!("--- Generated assembly ---\n{}", asm);
    println!("--- Running assembly ---");
    let instrs = generate_prog_to_instrs(&prog).expect("to instrs");
    let mut m = Mips::new(instrs);
    // Halt is the expected termination, not an error
    match m.run() {
        Ok(_) => println!("Program completed successfully"),
        Err(mips::error::Error::Halt) => println!("Program completed successfully (halt)"),
        Err(e) => panic!("mips run error: {:?}", e),
    }
}
