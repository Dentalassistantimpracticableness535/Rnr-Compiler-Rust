use rnr::ast::{Block, Expr, Type};
use rnr::parse::parse;
use rnr::test_util::{assert_block_type_fail, assert_type};
use rnr::type_check::TypeChecker;

// I use a lot AI to make these tests

#[test]
fn literal_types() {
    let blk: Block = parse("{ 1 }");
    assert_type(&blk, Type::I32);
    let blk: Block = parse("{ true }");
    assert_type(&blk, Type::Bool);
    let blk: Block = parse("{ () }");
    assert_type(&blk, Type::Unit);
}

#[test]
fn arithmetic_ops() {
    let blk: Block = parse("{ 1 + 2 }");
    assert_type(&blk, Type::I32);
    let blk: Block = parse("{ 1 * (2 + 3) }");
    assert_type(&blk, Type::I32);
    let blk: Block = parse("{ 1 + true }");
    assert_block_type_fail(&blk);
}

#[test]
fn let_and_assign() {
    let blk: Block = parse("{ let a: i32 = 1; a }");
    assert_type(&blk, Type::I32);
    let blk: Block = parse("{ let a: i32 = true; a }");
    assert_block_type_fail(&blk);
    let blk: Block = parse("{ let a; }");
    assert_block_type_fail(&blk);
}

#[test]
fn if_exprs() {
    let blk: Block = parse("{ if true { 1 } else { 2 } }");
    assert_type(&blk, Type::I32);
    let blk: Block = parse("{ if 1 { 1 } else { 2 } }");
    assert_block_type_fail(&blk);
}

#[test]
fn var_access_in_child_scope() {
    let blk: Block = parse("{ let a: i32 = 1; { a } }");
    assert_type(&blk, Type::I32);
}

#[test]
fn shadowing_between_scopes() {
    let blk: Block = parse("{ let a: i32 = 1; { let a: bool = true; a } }");
    assert_type(&blk, Type::Bool);
    let blk: Block = parse("{ let a: i32 = 1; { let a: bool = true; a }; a }");
    assert_type(&blk, Type::I32);
}

#[test]
fn shadowing_same_scope_allowed() {
    let blk: Block = parse("{ let a: i32 = 1; let a: i32 = 3; a }");
    assert_type(&blk, Type::I32);
}

#[test]
fn var_not_declared_is_error() {
    let blk: Block = parse("{ a }");
    assert_block_type_fail(&blk);
}

#[test]
fn exit_scope_variable_inaccessible() {
    let blk: Block = parse("{ { let a: i32 = 1; }; a }");
    assert_block_type_fail(&blk);
}

#[test]
fn assignment_correct_same_type() {
    let blk: Block = parse("{ let mut a: i32 = 1; a = 2; a }");
    assert_type(&blk, Type::I32);
}

#[test]
fn assignment_type_mismatch_is_error() {
    let blk: Block = parse("{ let a: i32 = 1; a = true; }");
    assert_block_type_fail(&blk);
}

#[test]
fn assignment_before_declaration_is_error() {
    let blk: Block = parse("{ a = 1 }");
    assert_block_type_fail(&blk);
}

#[test]
fn assignment_to_non_mutable_forbidden() {
    let blk: Block = parse("{ let a: i32 = 1; a = 2; }");
    assert_block_type_fail(&blk);
}

#[test]
fn fn_declaration_and_call() {
    let blk: Block = parse("{ fn f(x: i32) -> i32 { x + 1 } f(1) }");
    assert_type(&blk, Type::I32);
}

#[test]
fn fn_wrong_arg_count_is_error() {
    let blk: Block = parse("{ fn f(x: i32) -> i32 { x + 1 } f(1,2) }");
    assert_block_type_fail(&blk);
}

#[test]
fn fn_wrong_arg_type_is_error() {
    let blk: Block = parse("{ fn f(x: i32) -> i32 { x + 1 } f(true) }");
    assert_block_type_fail(&blk);
}

#[test]
fn fn_not_declared_is_error() {
    let blk: Block = parse("{ f(1) }");
    assert_block_type_fail(&blk);
}

#[test]
fn fn_shadowing_same_scope_forbidden() {
    let blk: Block = parse("{ fn f() -> i32 { 1 } fn f() -> i32 { 2 } }");
    assert_block_type_fail(&blk);
}

#[test]
fn fn_in_child_scope_does_not_mask_global_after_exit() {
    let blk: Block = parse("{ fn f() -> i32 { 1 } { fn f() -> i32 { 2 } }; f() }");
    assert_type(&blk, Type::I32);
}

#[test]
fn fn_return_type_mismatch_is_error() {
    let prog: rnr::ast::Prog = parse("fn f(x: i32) -> i32 { true }");
    let mut tc = TypeChecker::new();
    let r = tc.check_prog(&prog);
    assert!(
        r.is_err(),
        "expected function return type mismatch to fail but got {:?}",
        r
    );
}

#[test]
#[ignore]
fn fn_missing_return_path_error() {
    let blk: Block = parse("{ fn f() -> i32 { if true { return 1; } } }");
    assert_block_type_fail(&blk);
}

#[test]
fn test_check_if_then_else_shadowing_type() {
    let v: Block = parse(
        "
        {
            let mut a: i32 = 1 + 2; // a == 3
            let mut a: i32 = 2 + a; // a == 5
            if true {
                a = a - 1;      // outer a == 4
                let mut a: i32 = 0; // inner a == 0
                a = a + 1       // inner a == 1
            } else {
                a = a - 1
            };
            a   // a == 4
        }
        ",
    );
    assert_type(&v, Type::I32);
}

#[test]
fn test_block_let_shadow_type() {
    let v: Expr = parse(
        "
    {
        let a: i32 = 1;
        let b: i32 = 2;
        let a: i32 = 3;
        let b: i32 = 4;

        a + b
    }",
    );
    assert_type(&v, Type::I32);
}

#[test]
fn if_without_else_then_must_be_unit() {
    // The then-block must be Unit when there is no else
    let blk: Block = parse("{ if true { 1 } }");
    assert_block_type_fail(&blk);
}

#[test]
fn if_branch_type_mismatch_is_error() {
    let blk: Block = parse("{ if true { 1 } else { true } }");
    assert_block_type_fail(&blk);
}

#[test]
fn if_condition_non_bool_is_error() {
    let blk: Block = parse("{ if 1 { 1 } else { 2 } }");
    assert_block_type_fail(&blk);
}

#[test]
fn comparison_eq_typechecks() {
    let blk: Block = parse("{ 1 == 2 }");
    assert_type(&blk, Type::Bool);
}

#[test]
fn arithmetic_on_bool_is_error() {
    let blk: Block = parse("{ true + false }");
    assert_block_type_fail(&blk);
}

#[test]
fn static_overload_resolution_success() {
    let blk: Block = parse("{ fn f(x: i32) -> i32 { x } fn f(x: bool) -> i32 { 0 } f(1) }");
    assert_type(&blk, Type::I32);
    let blk: Block = parse("{ fn f(x: i32) -> i32 { x } fn f(x: bool) -> i32 { 0 } f(true) }");
    assert_type(&blk, Type::I32);
}

#[test]
fn static_overload_no_match_is_error() {
    let blk: Block = parse("{ fn f(x: bool) -> i32 { 0 } f(1) }");
    assert_block_type_fail(&blk);
}

#[test]
fn typecheck_assign_through_deref() {
    let blk: Block = parse("{ let mut a: i32 = 1; let r = &mut a; *r = 2; a }");
    assert_type(&blk, Type::I32);
    let blk: Block = parse("{ let mut a: i32 = 1; let r = &a; *r = 2; }");
    assert_block_type_fail(&blk);
}
