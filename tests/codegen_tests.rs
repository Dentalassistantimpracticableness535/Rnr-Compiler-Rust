use rnr::ast;
use rnr::ast::*;
use rnr::codegen::generate_prog_to_string;
use rnr::common::codegen_test;

#[test]
fn gen_int_literal_in_main() {
    let f = FnDeclaration {
        id: "main".to_string(),
        parameters: Parameters(vec![]),
        ty: None,
        body: Block::new(vec![Statement::Expr(Expr::Lit(Literal::Int(1)))], true),
    };
    let prog = Prog(vec![f]);
    let out = generate_prog_to_string(&prog).expect("codegen");
    assert!(out.contains("main:"));
    assert!(out.contains("addi t0, zero, 1"));
}

#[test]
fn gen_string_and_println() {
    let call = Expr::Call(
        "println!".to_string(),
        Arguments(vec![Expr::Lit(Literal::String("hello".to_string()))]),
    );
    let f = FnDeclaration {
        id: "main".to_string(),
        parameters: Parameters(vec![]),
        ty: None,
        body: Block::new(vec![Statement::Expr(call)], true),
    };
    let prog = Prog(vec![f]);
    let out = generate_prog_to_string(&prog).expect("codegen");
    assert!(out.contains(".data"));
    assert!(out.contains("la   t0"));
    assert!(out.contains("bal  println"));
}

#[test]
fn gen_deref_assign_addressing() {
    use rnr::ast::{Block, Expr, Literal, Mutable, Statement, Type};

    // fn main() -> i32 { let mut x = 1; let r = &mut x; *r = 5; x }
    let stmts = vec![
        Statement::Let(
            Mutable(true),
            "x".to_string(),
            Some(Type::I32),
            Some(Expr::Lit(Literal::Int(1))),
        ),
        Statement::Let(
            Mutable(false),
            "r".to_string(),
            None,
            Some(Expr::Ref(
                Box::new(Expr::Ident("x".to_string())),
                Mutable(true),
            )),
        ),
        Statement::Assign(
            Expr::UnOp(
                rnr::ast::UnOp::Deref,
                Box::new(Expr::Ident("r".to_string())),
            ),
            Expr::Lit(Literal::Int(5)),
        ),
        Statement::Expr(Expr::Ident("x".to_string())),
    ];
    let body = Block::new(stmts, true);
    let f = rnr::ast::FnDeclaration {
        id: "main".to_string(),
        parameters: rnr::ast::Parameters(vec![]),
        ty: Some(Type::I32),
        body,
    };
    let prog = rnr::ast::Prog(vec![f]);
    let out = rnr::codegen::generate_prog_to_string(&prog).expect("codegen");

    // x should be assigned at some negative offset (first local -> -4), and r at -8
    assert!(
        out.contains("addi t0, fp, -4"),
        "expected address load for x"
    );
    assert!(
        out.contains("lw   t1, -8(fp)"),
        "expected load of r's stored address into t1"
    );
    assert!(
        out.contains("sw   t0, 0(t1)"),
        "expected store via deref into 0(t1)"
    );
}

// ----------------------
// End-to-end runtime tests (moved from e2e files)
// ----------------------

#[test]
fn gen_scoping_and_shadowing() {
    // inner let-bindings shadow outer ones
    let src = r#"
    fn main() { shadow(5) }
    fn shadow(x: i32) -> i32 { let x = x + 1; let x = x + 1; x }
    "#;

    let prog: ast::Prog = rnr::parse::parse(src);
    let asm = generate_prog_to_string(&prog).expect("codegen");
    assert!(
        asm.contains("shadow:"),
        "asm should contain shadow label:\n{}",
        asm
    );

    let m = codegen_test::<ast::Prog>(src).expect("run");
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 7);
}

#[test]
fn gen_println_runs_and_generates_asm() {
    let src = r#"
    fn main() { println!("hello {}", 42) }
    "#;

    let prog: ast::Prog = rnr::parse::parse(src);
    let asm = generate_prog_to_string(&prog).expect("codegen");
    assert!(asm.contains(".data") || asm.contains(".asciiz"),
        "expected .data section in asm:\n{}", asm);
    assert!(
        asm.contains("bal  println") || asm.contains("bal println"),
        "asm missing println call:\n{}",
        asm
    );
    let _m = codegen_test::<ast::Prog>(src).expect("run");
}

#[test]
fn gen_println_multiple_and_varargs() {
    let src = r#"
    fn main() { let x = 3; println!("a {} b {} {}", 1, x, 2) }
    "#;

    let prog: ast::Prog = rnr::parse::parse(src);
    let asm = generate_prog_to_string(&prog).expect("codegen");
    assert!(
        asm.contains("bal  println") || asm.contains("bal println"),
        "asm missing println call:\n{}",
        asm
    );
    let _m = codegen_test::<ast::Prog>(src).expect("run");
}

#[test]
fn gen_println_no_args_and_literal_only() {
    // no args
    let src1 = r#" fn main() { println!() } "#;
    let _m1 = codegen_test::<ast::Prog>(src1).expect("run");

    // literal only
    let src2 = r#" fn main() { println!("hello") } "#;
    let prog: ast::Prog = rnr::parse::parse(src2);
    let asm = generate_prog_to_string(&prog).expect("codegen");
    assert!(
        asm.contains("la   t0") || asm.contains(".asciiz"),
        "expected data in asm:\n{}",
        asm
    );
    let _m2 = codegen_test::<ast::Prog>(src2).expect("run");
}

#[test]
fn gen_shadow_nested_blocks() {
    let src = r#"
        fn main() -> i32 {
            let x = 1;
            {
                let x = 2;
                {
                    let x = 3;
                };
                x;
            };
            x
        }
        "#;
    // The outermost x should still be 1
    let m = codegen_test::<ast::Prog>(src).expect("run");
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 1);
}

#[test]
fn gen_shadow_param_and_local() {
    let src = r#"
    fn main() -> i32 { foo(4) }
    fn foo(x: i32) -> i32 { let x = x + 1; x }
    "#;
    let m = codegen_test::<ast::Prog>(src).expect("run");
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 5);
}

#[test]
fn gen_shadow_if_branches() {
    let src = r#"
    fn main() -> i32 { let a = 10; if a > 0 { let a = 1; a } else { let a = 2; a } }
    "#;
    let m = codegen_test::<ast::Prog>(src).expect("run");
    // condition true -> then branch returns 1
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 1);
}

// Variety tests

#[test]
fn gen_multi_params_and_return() {
    let src = r#"
    fn main() -> i32 { add3(1,2,3) }
    fn add3(a: i32, b: i32, c: i32) -> i32 { a + b + c }
    "#;
    let m = codegen_test::<rnr::ast::Prog>(src).expect("run");
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 6);
}

#[test]
#[ignore = "mul instruction not supported by the mips crate VM"]
fn gen_recursive_factorial() {
    // factorial(5) = 120
    let src = r#"
    fn main() -> i32 { fact(5) }
    fn fact(n: i32) -> i32 { if n == 0 { 1 } else { n * fact(n-1) } }
    "#;
    let m = codegen_test::<rnr::ast::Prog>(src).expect("run");
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 120);
}

#[test]
fn gen_refs_and_deref_mutation() {
    // let mut x = 1; let r = &mut x; *r = 5; x -> 5
    let src = r#"
    fn main() -> i32 {
        let mut x = 1;
        let r = &mut x;
        *r = 5;
        x
    }
    "#;
    let m = codegen_test::<rnr::ast::Prog>(src).expect("run");
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 5);
}

#[test]
fn gen_boolean_logic_and_conditionals() {
    // test boolean operators and branching
    let src = r#"
    fn main() -> i32 {
        let a = 3;
        let b = 4;
        if a < b && b > a { 1 } else { 0 }
    }
    "#;
    let m = codegen_test::<rnr::ast::Prog>(src).expect("run");
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 1);
}

#[test]
#[ignore = "div instruction not supported by the mips crate VM"]
fn gen_divide_by_zero_behavior() {
    // VM implements division with special-case: div by zero -> 0
    let src = r#" fn main() -> i32 { let a = 10 / 0 ; a } "#;
    let m = codegen_test::<rnr::ast::Prog>(src).expect("run");
    // Expect 0 (runtime defined behavior in VM)
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 0);
}

#[test]
fn gen_mutation_and_assign_to_ref() {
    // Assign to reference created from an lvalue
    let src = r#"
    fn main() -> i32 {
        let mut x = 2;
        let r = &mut x;
        *r = *r + 8;
        x
    }
    "#;
    let m = codegen_test::<rnr::ast::Prog>(src).expect("run");
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 10);
}

#[test]
fn gen_while_and_loop_interaction() {
    // small loop that decrements to zero
    let src = r#"
    fn main() -> i32 { countdown(4) }
    fn countdown(x: i32) -> i32 { while x > 0 { x = x - 1 }; x }
    "#;
    let m = codegen_test::<rnr::ast::Prog>(src).expect("run");
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 0);
}

#[test]
fn gen_call_simple_program() {
    let src = "
    fn main() { foo(3) }
    fn foo(x: i32) -> i32 { x + 2 }
    ";
    let m = codegen_test::<ast::Prog>(src).unwrap();
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 5);
}

#[test]
fn gen_call_nested_program() {
    let src = "
    fn main() { add2(0) }
    fn add2(x: i32) -> i32 { add1(add1(x)) }
    fn add1(x: i32) -> i32 { x + 1 }
    ";
    let m = codegen_test::<ast::Prog>(src).unwrap();
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 2);
}

#[test]
fn gen_recursion_sum() {
    let src = "
    fn main() { sum(3) }
    fn sum(n: i32) -> i32 { if n == 0 { 0 } else { n + sum(n-1) } }
    ";
    let m = codegen_test::<ast::Prog>(src).unwrap();
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 6);
}

#[test]
fn gen_big_frame_overflow_check() {
    let mut body = String::from("\n    fn main() { big(); } fn big() -> i32 { ");
    for i in 0..200 {
        body.push_str(&format!("let v{} = {} ;", i, i));
    }
    body.push_str(" 0 }\n");
    let m = codegen_test::<ast::Prog>(&body).unwrap();
    assert_eq!(m.rf.get(mips::rf::Reg::t0), 0);
}

// Integration tests (merged from asm_runner_integration.rs and mips_integration.rs)
#[test]
fn codegen_run_print_asm_finishes() {
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
    let _asm = generate_prog_to_string(&prog).expect("codegen");
    let instrs = rnr::codegen::generate_prog_to_instrs(&prog).expect("to instrs");
    let mut m = mips::vm::Mips::new(instrs);
    match m.run() {
        Ok(_) => {}
        Err(mips::error::Error::Halt) => {}
        Err(e) => panic!("mips run failed with unexpected error: {:?}", e),
    }
}

#[test]
fn codegen_mips_simple_arith_finishes() {
    let f = FnDeclaration {
        id: "main".to_string(),
        parameters: Parameters(vec![]),
        ty: Some(Type::I32),
        body: Block::new(
            vec![Statement::Expr(Expr::BinOp(
                BinOp::Add,
                Box::new(Expr::Lit(Literal::Int(1))),
                Box::new(Expr::Lit(Literal::Int(2))),
            ))],
            true,
        ),
    };

    let prog = Prog(vec![f]);
    let asm = generate_prog_to_string(&prog).expect("codegen");
    eprintln!("--- ASM for simple_arith ---\n{}", asm);
    let instrs = rnr::codegen::generate_prog_to_instrs(&prog).expect("to instrs");
    let mut m = mips::vm::Mips::new(instrs);
    match m.run() {
        Ok(_) => {}
        Err(mips::error::Error::Halt) => {}
        Err(e) => panic!("mips run failed with unexpected error: {:?}", e),
    }
}

#[test]
fn codegen_mips_call_function_finishes() {
    let foo = FnDeclaration {
        id: "foo".to_string(),
        parameters: Parameters(vec![]),
        ty: Some(Type::I32),
        body: Block::new(vec![Statement::Expr(Expr::Lit(Literal::Int(7)))], true),
    };

    let main = FnDeclaration {
        id: "main".to_string(),
        parameters: Parameters(vec![]),
        ty: Some(Type::I32),
        body: Block::new(
            vec![Statement::Expr(Expr::Call(
                "foo".to_string(),
                Arguments(vec![]),
            ))],
            true,
        ),
    };

    let prog = Prog(vec![foo, main]);
    let asm = generate_prog_to_string(&prog).expect("codegen");
    eprintln!("--- ASM for call_function ---\n{}", asm);
    let instrs = rnr::codegen::generate_prog_to_instrs(&prog).expect("to instrs");
    let mut m = mips::vm::Mips::new(instrs);
    match m.run() {
        Ok(_) => {}
        Err(mips::error::Error::Halt) => {}
        Err(e) => panic!("mips run failed with unexpected error: {:?}", e),
    }
}
