use d7050e_lab4::ast::*;
use d7050e_lab4::common::Eval;
use d7050e_lab4::vm::Val;

#[test]
fn vm_call_add_returns_value() {
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
            false,
        ),
    };

    // create a block that hoists the functions and then calls main
    let mut stmts: Vec<Statement> = Vec::new();
    stmts.push(Statement::Fn(add));
    stmts.push(Statement::Fn(main));
    stmts.push(Statement::Expr(Expr::Call(
        "main".to_string(),
        Arguments(vec![]),
    )));
    let block = Block::new(stmts, false);
    let val = block.eval().expect("vm eval");
    match val {
        Val::Lit(Literal::Int(i)) => assert_eq!(i, 5),
        other => panic!("unexpected return: {:?}", other),
    }
}
