use d7050e_lab4::ast::*;
use d7050e_lab4::codegen::generate_prog_to_string;

fn main() {
    let add = FnDeclaration {
        id: "add".to_string(),
        parameters: Parameters(vec![
            Parameter {
                mutable: Mutable(false),
                id: "a".to_string(),
                ty: Type::I32,
            },
            Parameter {
                mutable: Mutable(false),
                id: "b".to_string(),
                ty: Type::I32,
            },
        ]),
        ty: Some(Type::I32),
        body: Block::new(
            vec![Statement::Expr(Expr::BinOp(
                BinOp::Add,
                Box::new(Expr::Ident("a".to_string())),
                Box::new(Expr::Ident("b".to_string())),
            ))],
            false,
        ),
    };

    let main = FnDeclaration {
        id: "main".to_string(),
        parameters: Parameters(vec![]),
        ty: None,
        body: Block::new(
            vec![Statement::Expr(Expr::Call(
                "add".to_string(),
                Arguments(vec![Expr::Lit(Literal::Int(2)), Expr::Lit(Literal::Int(3))]),
            ))],
            true,
        ),
    };

    let prog = Prog(vec![add, main]);
    let asm = generate_prog_to_string(&prog).expect("codegen failed");
    println!("{}", asm);
}
