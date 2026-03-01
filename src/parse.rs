use crate::ast::{
    Arguments, BinOp, Block, Expr, FnDeclaration, Literal, Mutable, Parameter, Parameters, Prog,
    Statement, Type, UnOp,
};

use syn::{
    parse::{Parse, ParseStream},
    Result, Token,
};

/// A small helper function for parsing source strings.
pub fn parse<T: Parse>(src: &str) -> T {
    try_parse::<T>(src).unwrap()
}

/// A small helper function for parsing source strings.
pub fn try_parse<T: Parse>(src: &str) -> Result<T> {
    let ts: proc_macro2::TokenStream = src.parse()?;
    syn::parse2::<T>(ts)
}

/// Might be useful if you are struggling with parsing something and you want to see what the
/// tokens are that syn produces.
pub fn try_parse_debug<T: Parse + std::fmt::Debug>(src: &str) -> Result<T> {
    println!("parsing source string:\n{}", src);
    let ts: proc_macro2::TokenStream = src.parse()?;
    println!("tokens:\n{}", ts);
    let result = syn::parse2::<T>(ts);
    println!("parsed AST:\n{:?}", result);
    result
}

// Back-port your parser
// You may want to put the tests in a module.
// See e.g., the vm.rs

impl Parse for Literal {
    fn parse(input: ParseStream) -> Result<Self> {
        // Use the "built in" syn parser for literals
        let lit: syn::Lit = input.parse()?;

        let lit = match lit {
            syn::Lit::Int(n) => Literal::Int(n.base10_parse().unwrap()),
            syn::Lit::Bool(b) => Literal::Bool(b.value),
            // for now only Int and Bool are covered
            syn::Lit::Str(s) => Literal::String(s.value()),
            _ => unimplemented!(),
        };
        Ok(lit)
    }
}

#[cfg(test)]
mod parse_lit {
    use super::*;
    use crate::test_util::assert_parse_fail;

    #[test]
    fn parse_lit_int() {
        let lit: Literal = parse("1");
        assert_eq!(lit, Literal::Int(1));
    }

    #[test]
    fn parse_lit_neg_int() {
        let lit: Literal = parse("-1");
        assert_eq!(lit, Literal::Int(-1));
    }

    #[test]
    fn parse_lit_bool_false() {
        let lit: Literal = parse("false");
        assert_eq!(lit, Literal::Bool(false));
    }

    #[test]
    fn parse_lit_string() {
        let lit: Literal = parse("\"abba\"");
        assert_eq!(lit, Literal::String("abba".to_string()));
    }

    #[test]
    fn parse_lit_fail() {
        assert_parse_fail::<Literal>("a");
        assert_parse_fail::<Literal>("-");
        assert_parse_fail::<Literal>("'hello'");
    }
}

impl Parse for BinOp {
    fn parse(input: ParseStream) -> Result<Self> {
        // check if next token is `+`
        if input.peek(Token![+]) {
            // consume the token
            let _: Token![+] = input.parse()?;
            Ok(BinOp::Add)
        } else if input.peek(Token![-]) {
            let _: Token![-] = input.parse()?;
            Ok(BinOp::Sub)
        } else if input.peek(Token![*]) {
            let _: Token![*] = input.parse()?;
            Ok(BinOp::Mul)
        } else if input.peek(Token![/]) {
            let _: Token![/] = input.parse()?;
            Ok(BinOp::Div)
        } else if input.peek(Token![&&]) {
            let _: Token![&&] = input.parse()?;
            Ok(BinOp::And)
        } else if input.peek(Token![||]) {
            let _: Token![||] = input.parse()?;
            Ok(BinOp::Or)
        } else if input.peek(Token![==]) {
            let _: Token![==] = input.parse()?;
            Ok(BinOp::Eq)
        } else if input.peek(Token![<]) {
            let _: Token![<] = input.parse()?;
            Ok(BinOp::Lt)
        } else if input.peek(Token![>]) {
            let _: Token![>] = input.parse()?;
            Ok(BinOp::Gt)
        // other matching tokens goes here
        } else {
            // to explicitly create an error at the current position
            input.step(|cursor| Err(cursor.error("expected binary operator")))
        }
    }
}

#[cfg(test)]
mod parse_binop {
    use super::*;
    use crate::test_util::assert_parse_fail;

    #[test]
    fn add() {
        let op: BinOp = parse("+");
        assert_eq!(op, BinOp::Add);
    }

    #[test]
    fn sub() {
        let op: BinOp = parse("-");
        assert_eq!(op, BinOp::Sub);
    }

    #[test]
    fn mul() {
        let op: BinOp = parse("*");
        assert_eq!(op, BinOp::Mul);
    }

    #[test]
    fn div() {
        let op: BinOp = parse("/");
        assert_eq!(op, BinOp::Div);
    }

    #[test]
    fn and() {
        let op: BinOp = parse("&&");
        assert_eq!(op, BinOp::And);
    }

    #[test]
    fn or() {
        let op: BinOp = parse("||");
        assert_eq!(op, BinOp::Or);
    }

    #[test]
    fn eq() {
        let op: BinOp = parse("==");
        assert_eq!(op, BinOp::Eq);
    }

    #[test]
    fn lt() {
        let op: BinOp = parse("<");
        assert_eq!(op, BinOp::Lt);
    }

    #[test]
    fn gt() {
        let op: BinOp = parse(">");
        assert_eq!(op, BinOp::Gt);
    }

    #[test]
    fn parse_op_fail() {
        assert_parse_fail::<BinOp>("1");
        assert_parse_fail::<BinOp>("x");
        assert_parse_fail::<BinOp>(".");
    }
}

impl Parse for UnOp {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![!]) {
            let _: Token![!] = input.parse()?;
            Ok(UnOp::Bang)
        } else if input.peek(Token![-]) {
            let _: Token![-] = input.parse()?;
            Ok(UnOp::Neg)
        } else if input.peek(Token![*]) {
            let _: Token![*] = input.parse()?;
            Ok(UnOp::Deref)
        } else {
            input.step(|cursor| Err(cursor.error("expected unary operator")))
        }
    }
}

#[cfg(test)]
mod parse_unop {
    use super::*;
    use crate::test_util::assert_parse_fail;

    #[test]
    fn bang() {
        let op: UnOp = parse("!");
        assert_eq!(op, UnOp::Bang);
    }

    #[test]
    fn neg() {
        let op: UnOp = parse("-");
        assert_eq!(op, UnOp::Neg);
    }

    #[test]
    fn parse_unop_fail() {
        assert_parse_fail::<UnOp>(".");
        assert_parse_fail::<UnOp>("/");
        assert_parse_fail::<UnOp>("x");
        assert_parse_fail::<UnOp>("1");
        assert_parse_fail::<UnOp>("i32");
        assert_parse_fail::<UnOp>("true");
        assert_parse_fail::<UnOp>("()");
    }
}

impl Parse for Expr {
    fn parse(input: ParseStream) -> Result<Self> {
        let left = parse_operand(input)?;
        // Now check if the rest is an Op Expr...
        if peek_op(input) {
            // In that case, we have to parse the rest of the expression.
            parse_binary_op_expr(input, left, 0)
        } else {
            // Otherwise, the first part was the whole expression.
            Ok(left)
        }
    }
}

// NOTE: About `peek_op` and `peek_prio`:
// We need to be able to look ahead at the next operator, but without parsing an
// Op and thereby consuming its tokens. I have not found a good way to either do
// something like `input.peek(Op)` or `input.unread(op)` when using syn.
// So forking and parsing is the most concise solution I could find. /chrfin

/// Check if the next token is some (binary) operator.
fn peek_op(input: ParseStream) -> bool {
    input.fork().parse::<BinOp>().is_ok()
}

/// Get the priority of the operator ahead. Assumes there is one!
fn peek_prio(input: ParseStream) -> u8 {
    input.fork().parse::<BinOp>().unwrap().priority()
}

/// Check if the next token is some (unary) operator.
fn peek_unop(input: ParseStream) -> bool {
    input.fork().parse::<UnOp>().is_ok()
}

/// Check if we've reached the end of a binary operator expression.
/// Depending on how expressions are parsed, an expression is usually terminated by the
/// input running out. But we may also run into some token that means the expression is done.
fn end_of_expr(input: ParseStream) -> bool {
    if input.is_empty() {
        true
    } else {
        // These may not be needed in practice (if we use something like
        // `syn::parenthesized!(...)` or `parse_terminated` for example).
        // But, in principle, we could for example reach the end of an array element or function
        // argument, etc. So let's be general.
        input.peek(Token![,])
            || input.peek(Token![;])
            || input.peek(syn::token::Brace)
            || input.peek(syn::token::Bracket)
            || input.peek(syn::token::Paren)
    }
}

/// Parse what could be an operand, i.e. the first part of a binary expression.
/// This could be a literal, an identifier, a unary op, or an expression in parentheses.
/// For example: `3 + ...`, `x + ...`, `!true && ...`, `(1+2) + ...`, or `[1,2,3][0] + ...`.
fn parse_operand(input: ParseStream) -> Result<Expr> {
    // handle if
    if input.peek(Token![if]) {
        let if_expr: IfThenOptElse = input.parse()?;
        return Ok(Expr::IfThenElse(Box::new(if_expr.0), if_expr.1, if_expr.2));
    }
    // handle blocks
    if input.peek(syn::token::Brace) {
        let block: Block = input.parse()?;
        return Ok(Expr::Block(block));
    }
    // handle reference operator `&` and `&mut`
    if input.peek(Token![&]) {
        let _: Token![&] = input.parse()?;
        let mutable = if input.peek(Token![mut]) {
            let _: Token![mut] = input.parse()?;
            Mutable(true)
        } else {
            Mutable(false)
        };
        let inner = parse_operand(input)?;
        return Ok(Expr::Ref(Box::new(inner), mutable));
    }
    if input.peek(syn::Lit) {
        Ok(Expr::from(input.parse::<Literal>()?))
    } else if input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        // Check if it's ()
        if content.is_empty() {
            Ok(Expr::Lit(Literal::Unit))
        } else {
            let expr = Expr::parse(&content)?;
            Ok(Expr::Par(Box::new(expr)))
        }
    } else if input.peek(syn::Ident) {
        parse_ident_or_call(input)
    } else if peek_unop(input) {
        // expecting unary operator
        let op: UnOp = input.parse()?;
        let right = parse_operand(input)?;
        Ok(Expr::UnOp(op, Box::new(right)))
    } else {
        input.step(|cursor| {
            Err(cursor
                .error("expected literal, identifier, parenthesized expression, or unary operator"))
        })
    }
}

/// Parse an expression consisting of binary operators, such as `1 + 2`, `1 + 2 + 3`,
/// `1 + 2 * 3 + 4`, or `(1 + 2) * (2 + 3)`.
/// To be more specific: given some beginning part of an expression (in `left`), parse the
/// remainder of the expression, starting with the next operator. For example, `left` might be
/// `1` or `1 + 2`, and the input might be `+ 2` or `+ 2 + 3`, etc.
/// The priority (or precedence) of operators is taken into account during parsing
/// so we get the correct AST.
fn parse_binary_op_expr(input: ParseStream, left: Expr, min_prio: u8) -> Result<Expr> {
    let op1: BinOp = input.parse()?;
    let prio1 = op1.priority();
    let right: Expr = parse_operand(input)?;
    if end_of_expr(input) {
        return Ok(Expr::bin_op(op1, left, right));
    }
    // At this point, we have left, op, right. And there are more tokens.
    // So now we need to check the next op to see whether we accumulate left or not.
    let prio2 = peek_prio(input);
    if prio2 > prio1 {
        // The next op has higher priority. So we have to parse the right side first.
        // Note that `right` becomes `left` in the recursive call.
        // Once the higher-priority part has been parsed, *that* becomes the new `right`
        // back here where we left off.
        let right = parse_binary_op_expr(input, right, prio1)?;
        let left = Expr::bin_op(op1, left, right);
        if end_of_expr(input) {
            Ok(left)
        } else {
            parse_binary_op_expr(input, left, min_prio)
        }
    } else {
        let left = Expr::bin_op(op1, left, right);
        if prio2 <= min_prio {
            Ok(left) // stop recursion and return to lower-prio op
        } else {
            parse_binary_op_expr(input, left, min_prio)
        }
    }
}

fn parse_ident_or_call(input: ParseStream) -> Result<Expr> {
    let ident: syn::Ident = input.parse()?;
    // Check for println!(...)
    if input.peek(Token![!]) {
        let _: Token![!] = input.parse()?;
        let content;
        syn::parenthesized!(content in input);
        let args = content.parse_terminated(Expr::parse, Token![,])?;

        Ok(Expr::Call(
            format!("{}!", ident),
            Arguments(args.into_iter().collect()),
        ))
    } else if input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        let args = content.parse_terminated(Expr::parse, Token![,])?;

        Ok(Expr::Call(
            ident.to_string(),
            Arguments(args.into_iter().collect()),
        ))
    } else {
        Ok(Expr::Ident(ident.to_string()))
    }
}

#[cfg(test)]
mod parse_expr {
    use super::*;
    use crate::ast::Expr;
    use crate::test_util::assert_parse_fail;

    // NOTE: Most tests that involve expressions have been moved to the `expr` module in
    // the `tests/integration_tests.rs` file.
    // Here we just focus on checking that parsing does not panic, and also checking that binary
    // operator expressions are correctly parsed (in terms of associativity and precedence).
    // In particular, these tests do not evaluate expressions!

    #[test]
    fn literal() {
        parse::<Expr>("123");
        parse::<Expr>("true");
        parse::<Expr>("()");
        parse::<Expr>("\"hello world\"");
    }

    #[test]
    fn binary_op() {
        parse::<Expr>("1 + 2");
        parse::<Expr>("1 - 2");
        parse::<Expr>("1 * 2");
        parse::<Expr>("1 / 2");
        parse::<Expr>("1 + 2 + 3");
        parse::<Expr>("1 * 2 - 3 / 1");
        parse::<Expr>("(1) + (2)");
        parse::<Expr>("(1 + 2) + (3 + 4)");
        parse::<Expr>("(1 * 2) - (3 / 4)");
        parse::<Expr>("true || false");
        parse::<Expr>("true && false");
        parse::<Expr>("true && (false || true) && false");
    }

    #[test]
    fn binary_op_comparisons() {
        parse::<Expr>("1 == 2");
        parse::<Expr>("1 < 2");
        parse::<Expr>("1 > 2");
        parse::<Expr>("1 + (2 * 3) == (2 - 3) / 4");
        parse::<Expr>("1 + 2 < 2 * 3");
        parse::<Expr>("1 + 2 > (2 - 1)");
        parse::<Expr>("true == true");
        parse::<Expr>("true || false == true && (false || true)");
    }

    #[test]
    fn unary_op() {
        parse::<Expr>("-(1)");
        parse::<Expr>("-(1+2)");
        parse::<Expr>("-(2 * 3 / 2)");
        parse::<Expr>("!true");
        parse::<Expr>("!true && true");
        parse::<Expr>("!true || !false");
        parse::<Expr>("!(true && !true)");
    }

    #[test]
    fn identifier() {
        parse::<Expr>("my_variable");
        parse::<Expr>("var");
        parse::<Expr>("var123");
        parse::<Expr>("_var_123_");
    }

    #[test]
    fn operators_and_identifiers() {
        parse::<Expr>("-my_variable");
        parse::<Expr>("!var");
        parse::<Expr>("321 + var123 - 123");
        parse::<Expr>("(1 - _var_123_ * 2) == a && (!b || true) || !a");
    }

    // Trying to parse these expressions should fail.

    #[test]
    fn fail_tests() {
        assert_parse_fail::<Expr>("12 34");
        assert_parse_fail::<Expr>("+");
        assert_parse_fail::<Expr>("1+");
        assert_parse_fail::<Expr>("1++2");
        assert_parse_fail::<Expr>("1+2+3+4+");
        assert_parse_fail::<Expr>("(1+2+3+4");
        assert_parse_fail::<Expr>("1+2+3+4)");
        assert_parse_fail::<Expr>("1)+2+(3+4");
        assert_parse_fail::<Expr>("3(1+2)");
        assert_parse_fail::<Expr>("(1+2)3");
        assert_parse_fail::<Expr>("(1+2)(3+4)");
        assert_parse_fail::<Expr>("12!34");
        assert_parse_fail::<Expr>("true ! false");
        assert_parse_fail::<Expr>("(2 * 4) - ");
    }

    // Some helpers for building Expr ASTs.

    fn add<T1: Into<Expr>, T2: Into<Expr>>(left: T1, right: T2) -> Expr {
        Expr::bin_op(crate::ast::BinOp::Add, left.into(), right.into())
    }

    fn mul<T1: Into<Expr>, T2: Into<Expr>>(left: T1, right: T2) -> Expr {
        Expr::bin_op(crate::ast::BinOp::Mul, left.into(), right.into())
    }

    fn or<T1: Into<Expr>, T2: Into<Expr>>(left: T1, right: T2) -> Expr {
        Expr::bin_op(crate::ast::BinOp::Or, left.into(), right.into())
    }

    fn and<T1: Into<Expr>, T2: Into<Expr>>(left: T1, right: T2) -> Expr {
        Expr::bin_op(crate::ast::BinOp::And, left.into(), right.into())
    }

    fn eq<T1: Into<Expr>, T2: Into<Expr>>(left: T1, right: T2) -> Expr {
        Expr::bin_op(crate::ast::BinOp::Eq, left.into(), right.into())
    }

    pub fn paren(expr: Expr) -> Expr {
        Expr::Par(Box::new(expr))
    }

    // Here are some test cases that directly examine the AST that is built from the expressions to
    // make sure that precedence and associativity are handled correctly.

    #[test]
    fn precedence_and_associativity_1() {
        let expr: Expr = parse("1+2+3");
        let expected = add(add(1, 2), 3);
        assert_eq!(expr, expected);
    }

    #[test]
    fn precedence_and_associativity_2() {
        let expr: Expr = parse("1+2*3");
        let expected = add(1, mul(2, 3));
        assert_eq!(expr, expected);
    }

    #[test]
    fn precedence_and_associativity_3() {
        let expr: Expr = parse("1+2*3+4");
        let expected = add(add(1, mul(2, 3)), 4);
        assert_eq!(expr, expected);
    }

    #[test]
    fn precedence_and_associativity_4() {
        let expr: Expr = parse("1+2*3 == 4");
        let expected = eq(add(1, mul(2, 3)), 4);
        assert_eq!(expr, expected);
    }

    #[test]
    fn precedence_and_associativity_5() {
        let expr: Expr = parse("1+2*3 == 1+2*3");
        let expected = eq(add(1, mul(2, 3)), add(1, mul(2, 3)));
        assert_eq!(expr, expected);
    }

    #[test]
    fn precedence_and_associativity_6() {
        let expr: Expr = parse("1*2+3*4+5*6");
        let expected = add(add(mul(1, 2), mul(3, 4)), mul(5, 6));
        assert_eq!(expr, expected);
    }

    #[test]
    fn precedence_and_associativity_7() {
        let expr: Expr = parse("1+2 * 3+4 == 5*(6+7) + 8*9");
        let left = add(add(1, mul(2, 3)), 4);
        let right = add(mul(5, paren(add(6, 7))), mul(8, 9));
        let expected = eq(left, right);
        assert_eq!(expr, expected);
    }

    // NOTE: priorities: `==` > `&&` > `||`
    // (Comparisons take precedence over and/or!)

    #[test] // 1 2 3
    fn precedence_and_associativity_123() {
        let expr: Expr = parse("true || true && true == true");
        let expected = or(true, and(true, eq(true, true)));
        assert_eq!(expr, expected);
    }

    #[test] // 1 3 2
    fn precedence_and_associativity_132() {
        let expr: Expr = parse("true || true == true && true");
        let expected = or(true, and(eq(true, true), true));
        assert_eq!(expr, expected);
    }

    #[test] // 2 1 3
    fn precedence_and_associativity_213() {
        let expr: Expr = parse("true && true || true == true");
        let expected = or(and(true, true), eq(true, true));
        assert_eq!(expr, expected);
    }

    #[test] // 2 3 1
    fn precedence_and_associativity_231() {
        let expr: Expr = parse("true && true == true || true");
        let expected = or(and(true, eq(true, true)), true);
        assert_eq!(expr, expected);
    }

    #[test] // 3 1 2
    fn precedence_and_associativity_312() {
        let expr: Expr = parse("true == true || true && true");
        let expected = or(eq(true, true), and(true, true));
        assert_eq!(expr, expected);
    }

    #[test] // 3 2 1
    fn precedence_and_associativity_321() {
        let expr: Expr = parse("true == true && true || true");
        let expected = or(and(eq(true, true), true), true);
        assert_eq!(expr, expected);
    }
}

//
// We want to parse strings like
// `if expr { then block }`
// and
// `if expr { then block } else { else block }
//
// The else arm is optional

struct IfThenOptElse(Expr, Block, Option<Block>);

impl Parse for IfThenOptElse {
    fn parse(input: ParseStream) -> Result<IfThenOptElse> {
        input.parse::<Token![if]>()?;
        let cond: Expr = input.parse()?;
        let then_block: Block = input.parse()?;
        let else_block = if input.peek(Token![else]) {
            input.parse::<Token![else]>()?;
            // Handle "else if" (treated as "else { if ... }")
            if input.peek(Token![if]) {
                let nested_if: IfThenOptElse = input.parse()?;
                Some(Block {
                    statements: vec![Statement::Expr(Expr::IfThenElse(
                        Box::new(nested_if.0),
                        nested_if.1,
                        nested_if.2,
                    ))],
                    semi: false,
                })
            } else {
                Some(input.parse::<Block>()?)
            }
        } else {
            None
        };
        Ok(IfThenOptElse(cond, then_block, else_block))
    }
}

#[cfg(test)]
mod parse_if {
    use super::*;
    use crate::ast::Expr;

    // This test is not really a test of our parser
    // Added just a reference to how Rust would treat the nesting.
    #[test]
    #[allow(unused_must_use)]
    fn test_if_then_else_nested_rust() {
        if false {
            2;
        } else {
            if true {
                3 + 5;
            }
        };
    }

    // This test is not really a test of our parser
    // Added just a reference to how Rust would treat the nesting.
    #[test]
    #[allow(unused_must_use)]
    fn test_if_then_else_nested_rust2() {
        if false {
            2;
        } else if true {
            3 + 5;
        };
    }

    // NOTE: These tests just parse some if-expressions and just (implicitly) check that there are
    // no panics.

    #[test]
    fn test_if_then_else_nested2() {
        let src = "
        if false {
            2;
        } else if true {
            3 + 5;
        }";
        let e: Expr = parse(src);
    }

    #[test]
    fn test_if_then_else_nested() {
        let src = "
        if false {
            2;
        } else {
            if true {
                3 + 5;
            }
        }";
        let e: Expr = parse(src);
    }

    #[test]
    fn test_if_then_else_nested3() {
        let src = "
        if false {
            2;
        } else if true {
            3 + 5;
        } else if false {
            let a : i32 = 0;
        } else {
            5
        }
        ";
        let e: Expr = parse(src);
    }

    #[test]
    fn test_expr_if_then_else() {
        let src = "if a > 0 {1} else {2}";
        let e: Expr = parse(src);
    }
}

use quote::quote;

impl Parse for Type {
    fn parse(input: ParseStream) -> Result<Type> {
        let syn_ty: syn::Type = input.parse()?;
        fn convert(ty: &syn::Type) -> std::result::Result<Type, String> {
            use quote::quote;
            // handle reference types: &T, &mut T
            if let syn::Type::Reference(r) = ty {
                let mutable = r.mutability.is_some();
                let inner = convert(&r.elem)?;
                return Ok(Type::Ref(Box::new(inner), Mutable(mutable)));
            }
            let s = quote!(#ty).to_string();
            match s.as_str() {
                "i32" => Ok(Type::I32),
                "bool" => Ok(Type::Bool),
                "String" => Ok(Type::String),
                "()" => Ok(Type::Unit),
                _ => Err(format!("unsupported type: {}", s)),
            }
        }
        convert(&syn_ty).map_err(|msg| input.error(msg))
    }
}

#[cfg(test)]
mod parse_type {
    use super::*;
    use crate::test_util::assert_parse_fail;

    #[test]
    fn parse_type_i32() {
        let typ: Type = parse("i32");
        assert_eq!(typ, Type::I32);
    }

    #[test]
    fn parse_type_bool() {
        let typ: Type = parse("bool");
        assert_eq!(typ, Type::Bool);
    }

    #[test]
    fn parse_type_unit() {
        let typ: Type = parse("()");
        assert_eq!(typ, Type::Unit);
    }

    #[test]
    fn parse_type_ref_i32() {
        let typ: Type = parse("&i32");
        assert_eq!(typ, Type::Ref(Box::new(Type::I32), Mutable(false)));
    }

    #[test]
    fn parse_type_ref_mut_i32() {
        let typ: Type = parse("&mut i32");
        assert_eq!(typ, Type::Ref(Box::new(Type::I32), Mutable(true)));
    }

    #[test]
    fn parse_type_ref_bool() {
        let typ: Type = parse("&bool");
        assert_eq!(typ, Type::Ref(Box::new(Type::Bool), Mutable(false)));
    }

    #[test]
    fn parse_type_ref_ref_i32() {
        let typ: Type = parse("&&i32");
        assert_eq!(
            typ,
            Type::Ref(
                Box::new(Type::Ref(Box::new(Type::I32), Mutable(false))),
                Mutable(false)
            )
        );
    }

    #[test]
    fn parse_type_fail() {
        assert_parse_fail::<Type>("u32");
        assert_parse_fail::<Type>("I32");
        assert_parse_fail::<Type>("123");
        assert_parse_fail::<Type>("boolean");
        assert_parse_fail::<Type>("Bool");
        assert_parse_fail::<Type>("true");
        assert_parse_fail::<Type>("false");
    }
}

impl Parse for Arguments {
    fn parse(input: ParseStream) -> Result<Arguments> {
        let content;
        syn::parenthesized!(content in input);
        let args = content.parse_terminated(Expr::parse, Token![,])?;
        Ok(Arguments(args.into_iter().collect()))
    }
}

#[cfg(test)]
mod parse_fn_calls {
    use super::*;
    use crate::test_util::assert_parse_fail;

    #[test]
    fn args() {
        parse::<Arguments>("(1)");
        parse::<Arguments>("(a)");
        parse::<Arguments>("(a, b)");
        parse::<Arguments>("(a + 1, b * 2)");
        parse::<Arguments>("(1, 2, 3 + 4)");
    }

    #[test]
    fn function_call() {
        parse::<Expr>("foo()");
        parse::<Expr>("foo(1)");
        parse::<Expr>("foo(true)");
        parse::<Expr>("foo(true, false)");
        parse::<Expr>("foo(true || false)");
        parse::<Expr>("foo(1 + 2)");
        parse::<Expr>("foo(1, 2)");
        parse::<Expr>("foo(1, 2 + 2)");
        parse::<Expr>("foo(my_variable)");
        parse::<Expr>("foo(a, b, c)");
        parse::<Expr>("foo(\"passing a string\")");
        parse::<Expr>("ident({1}, {let a = 6; a },)");
    }

    #[test]
    fn function_call_extra_comma() {
        parse::<Expr>("foo(1,)");
        parse::<Expr>("foo(1, 2,)");
        parse::<Expr>("foo(1, 2 + 2,)");
        parse::<Expr>("foo(a,)");
        parse::<Expr>("foo(a, b, c,)");
        parse::<Expr>("foo(true, false,)");
    }

    #[test]
    fn fail_tests() {
        assert_parse_fail::<Expr>("foo(,)");
        assert_parse_fail::<Expr>("foo(+)");
        assert_parse_fail::<Expr>("foo(1+)");
        assert_parse_fail::<Expr>("foo(2 * 4, -)");
    }
}

impl Parse for Parameter {
    fn parse(input: ParseStream) -> Result<Parameter> {
        let mutable = if input.peek(Token![mut]) {
            let _: Token![mut] = input.parse()?;
            Mutable(true)
        } else {
            Mutable(false)
        };

        let id: syn::Ident = input.parse()?;
        let _: Token![:] = input.parse()?; // parse colon
        let ty: Type = input.parse()?;

        Ok(Parameter {
            mutable,
            id: id.to_string(),
            ty,
        })
    }
}

// Here we take advantage of the parser function `parse_terminated`
impl Parse for Parameters {
    fn parse(input: ParseStream) -> Result<Parameters> {
        let content;
        syn::parenthesized!(content in input);
        let params = content.parse_terminated(Parameter::parse, Token![,])?;
        Ok(Parameters(params.into_iter().collect()))
    }
}

impl Parse for FnDeclaration {
    fn parse(input: ParseStream) -> Result<FnDeclaration> {
        let _: Token![fn] = input.parse()?; // parse fn
        let id: syn::Ident = input.parse()?;
        let parameters: Parameters = input.parse()?;
        let ty = if input.peek(Token![->]) {
            let _: Token![->] = input.parse()?; // parse ->
            Some(input.parse::<Type>()?)
        } else {
            None
        };
        let body: Block = input.parse()?;

        Ok(FnDeclaration {
            id: id.to_string(),
            parameters,
            ty,
            body,
        })
    }
}

#[cfg(test)]
mod parse_fn_declaration {
    use super::*;
    use crate::test_util::assert_parse_fail;

    #[test]
    fn param() {
        parse::<Parameter>("a: i32");
        parse::<Parameter>("b: bool");
    }

    #[test]
    fn params() {
        parse::<Parameters>("(a: i32)");
        parse::<Parameters>("(a: i32,)");
        parse::<Parameters>("(b: bool)");
        parse::<Parameters>("(a: i32, b: bool)");
        parse::<Parameters>("(a: i32, b: bool,)");
    }

    #[test]
    fn fn_no_type() {
        parse::<FnDeclaration>("fn foo() {}");
        parse::<FnDeclaration>("fn foo(a: i32, b: bool) {}");
    }

    #[test]
    fn fn_with_type() {
        parse::<FnDeclaration>("fn foo() -> i32 {}");
        parse::<FnDeclaration>("fn foo(a: i32, b: bool) -> i32 {}");
        parse::<FnDeclaration>("fn foo() -> () {}");
        parse::<FnDeclaration>("fn foo(a: i32, b: bool) -> () {}");
        parse::<FnDeclaration>("fn foo() -> bool {}");
        parse::<FnDeclaration>("fn foo(a: i32, b: bool) -> bool {}");
    }

    #[test]
    fn test_println() {
        let src = "println!(\"{}\", 1)";
        let expr: Expr = parse(src);
    }

    // Trying to parse these function declarations should fail.

    #[test]
    fn fail_tests() {
        assert_parse_fail::<Parameter>("123");
        assert_parse_fail::<Parameter>("i32");
        assert_parse_fail::<Parameter>("a = i32");
        assert_parse_fail::<FnDeclaration>("fn 123() {}");
        assert_parse_fail::<FnDeclaration>("fn foo(a, b: i32) {}");
        assert_parse_fail::<FnDeclaration>("fn foo(): i32 {}");
    }
}

impl Parse for Statement {
    fn parse(input: ParseStream) -> Result<Statement> {
        // parse fn_declaration
        if input.peek(Token![fn]) {
            let fn_decl: FnDeclaration = input.parse()?;
            return Ok(Statement::Fn(fn_decl));
        }

        // parse let
        if input.peek(Token![let]) {
            let _: Token![let] = input.parse()?;
            let mutable = if input.peek(Token![mut]) {
                let _: Token![mut] = input.parse()?;
                Mutable(true)
            } else {
                Mutable(false)
            };
            let id: syn::Ident = input.parse()?;
            let ty = if input.peek(Token![:]) {
                let _: Token![:] = input.parse()?;
                Some(input.parse::<Type>()?)
            } else {
                None
            };
            let expr = if input.peek(Token![=]) {
                let _: Token![=] = input.parse()?;
                Some(input.parse::<Expr>()?)
            } else {
                None
            };
            return Ok(Statement::Let(mutable, id.to_string(), ty, expr));
        }

        // parse while
        if input.peek(Token![while]) {
            let _: Token![while] = input.parse()?;
            let cond: Expr = input.parse()?;
            let block: Block = input.parse()?;
            return Ok(Statement::While(cond, block));
        }

        // otherwise
        let expr: Expr = input.parse()?;

        // parse assignment
        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            let rhs: Expr = input.parse()?;
            return Ok(Statement::Assign(expr, rhs));
        }

        // parse expr
        Ok(Statement::Expr(expr))
    }
}

#[cfg(test)]
mod parse_statement {
    use super::*;
    use crate::test_util::assert_parse_fail;

    #[test]
    fn test_statement_let_ty_expr() {
        let stmt: Statement = parse("let a: i32 = 2");
        let expected = Statement::Let(
            Mutable(false),
            "a".to_string(),
            Some(Type::I32),
            Some(Expr::Lit(Literal::Int(2))),
        );
        assert_eq!(stmt, expected);
    }

    #[test]
    fn test_statement_let_mut_ty_expr() {
        let stmt: Statement = parse("let mut a: i32 = 2");
        let expected = Statement::Let(
            Mutable(true),
            "a".to_string(),
            Some(Type::I32),
            Some(Expr::Lit(Literal::Int(2))),
        );
        assert_eq!(stmt, expected);
    }

    #[test]
    fn test_statement_let() {
        let stmt: Statement = parse("let a");
        let expected = Statement::Let(Mutable(false), "a".to_string(), None, None);
        assert_eq!(stmt, expected);
    }

    #[test]
    fn test_statement_assign() {
        let stmt: Statement = parse("a = false");
        let expected = Statement::Assign(
            Expr::Ident("a".to_string()),
            Expr::Lit(Literal::Bool(false)),
        );
        assert_eq!(stmt, expected);
    }

    #[test]
    fn test_statement_while() {
        let stmt: Statement = parse("while a {}");
        let expected = Statement::While(
            Expr::Ident("a".to_string()),
            Block {
                statements: vec![],
                semi: false,
            },
        );
        assert_eq!(stmt, expected);
    }

    #[test]
    fn test_statement_expr() {
        let stmt: Statement = parse("a");
        println!("stmt {:?}", stmt);
        assert_eq!(stmt, Statement::Expr(Expr::Ident("a".to_string())));
    }

    // Trying to parse these statements should fail.

    #[test]
    fn fail_tests() {
        assert_parse_fail::<Statement>("let a i32;");
        assert_parse_fail::<Statement>("let a: I32;");
        assert_parse_fail::<Statement>("let a: i32 == 3;");
        assert_parse_fail::<Statement>("let 123;");
        assert_parse_fail::<Statement>("let 123: i32;");
        assert_parse_fail::<Statement>("123_var = 3;");
        assert_parse_fail::<Statement>("while true { let x }");
        assert_parse_fail::<Statement>("while {}");
        // NOTE: we could also test something like "123 = 3", but we will want to allow the
        // left-hand side to be an expression (such as `xs[0] = 3`). So checking what kinds of
        // expressions are allowed on the left of an assignment would require a bit more work and
        // is probably best done by the type checker or VM.
    }
}

use syn::punctuated::Punctuated;

// Here we take advantage of the parser function `parse_terminated`
// impl Parse for Block {
//     fn parse(input: ParseStream) -> Result<Block> {
//         let content;
//         let _ = syn::braced!(content in input);

//         let bl: Punctuated<Statement, Token![;]> = content.parse_terminated(Statement::parse, Token![;])?;

//         // We need to retrieve the semi before we collect into a vector
//         // as into_iter consumes the value.
//         let semi = bl.trailing_punct();

//         Ok(Block {
//             // turn the Punctuated into a vector
//             statements: bl.into_iter().collect(),
//             semi,
//         })
//     }
// }

// Here we take advantage of the parser function `parse_terminated`
impl Parse for Block {
    fn parse(input: ParseStream) -> Result<Block> {
        let content;
        let _ = syn::braced!(content in input);

        // we need custom logic for fn/while that don't need ";"
        let mut statements = Vec::new();
        let mut semi = false;

        while !content.is_empty() {
            let stmt: Statement = content.parse()?;

            // fn and while don't need semicolon
            let is_fn_or_while = matches!(stmt, Statement::Fn(_) | Statement::While(_, _));
            // let always needs semicolon
            let is_let = matches!(stmt, Statement::Let(_, _, _, _));

            if content.peek(Token![;]) {
                let _: Token![;] = content.parse()?;
                semi = true;
            } else if is_fn_or_while {
                semi = false;
            } else if is_let {
                return Err(content.error("expected `;`"));
            } else if content.is_empty() {
                // last statement can omit semicolon
                semi = false;
            } else {
                return Err(content.error("expected `;`"));
            }

            statements.push(stmt);
        }

        Ok(Block { statements, semi })
    }
}

#[cfg(test)]
mod parse_block {
    use super::*;

    #[test]
    fn test_block_expr_fail() {
        let ts: proc_macro2::TokenStream = "{ let a = }".parse().unwrap();
        let stmt: Result<Statement> = syn::parse2(ts);
        println!("stmt {:?}", stmt);
        assert!(stmt.is_err());
    }

    #[test]
    fn test_block_semi() {
        let ts: proc_macro2::TokenStream = "
        {
            let a : i32 = 1;
            a = 5;
            a + 5;
        }"
        .parse()
        .unwrap();
        let bl: Block = syn::parse2(ts).unwrap();
        println!("bl {:?}", bl);
        assert_eq!(bl.statements.len(), 3);
        assert!(bl.semi);
    }

    #[test]
    fn test_block_no_semi() {
        let ts: proc_macro2::TokenStream = "
        {
            let a : i32 = 1;
            a = 5;
            a + 5
        }"
        .parse()
        .unwrap();
        let bl: Block = syn::parse2(ts).unwrap();
        println!("bl {:?}", bl);
        assert_eq!(bl.statements.len(), 3);
        assert!(!bl.semi);
    }

    #[test]
    fn test_block_fn() {
        let ts: proc_macro2::TokenStream = "
        {
            let a : i32 = 1;
            fn t() {}
            a = 5;
            a + 5
        }"
        .parse()
        .unwrap();
        let bl: Block = syn::parse2(ts).unwrap();
        println!("bl {:?}", bl);
        assert_eq!(bl.statements.len(), 4);
        assert!(!bl.semi);
    }

    #[test]
    fn test_block_while() {
        let ts: proc_macro2::TokenStream = "
        {
            let a : i32 = 1;
            while true {}
            a = 5;
            a + 5
        }"
        .parse()
        .unwrap();
        let bl: Block = syn::parse2(ts).unwrap();
        println!("bl {:?}", bl);
        assert_eq!(bl.statements.len(), 4);
        assert!(!bl.semi);
    }

    #[test]
    fn test_block2() {
        let ts: proc_macro2::TokenStream = "{ let b : bool = false; b = true }".parse().unwrap();
        let bl: Block = syn::parse2(ts).unwrap();
        println!("bl {:?}", bl);
        assert_eq!(bl.statements.len(), 2);
        assert!(!bl.semi);
    }

    #[test]
    fn test_expr_block() {
        let ts: proc_macro2::TokenStream = "
        {
            12
        }
        "
        .parse()
        .unwrap();
        println!("{:?}", ts);
        let e: Expr = syn::parse2(ts).unwrap();
        println!("e {:?}", e);
    }

    #[test]
    fn test_block_fail() {
        let ts: proc_macro2::TokenStream = "{ let a = 1 a = 5 }".parse().unwrap();
        let bl: Result<Block> = syn::parse2(ts);
        println!("bl {:?}", bl);

        assert!(bl.is_err());
    }
}

impl Parse for Prog {
    fn parse(input: ParseStream) -> Result<Prog> {
        let mut functions = Vec::new();
        while !input.is_empty() {
            let fn_decl: FnDeclaration = input.parse()?;
            functions.push(fn_decl);
        }

        Ok(Prog(functions))
    }
}

#[cfg(test)]
mod parse_prog {
    use super::*;

    #[test]
    fn test_prog() {
        let ts: proc_macro2::TokenStream = "
        fn a(a: i32) { let b = a; }
        fn b() -> i32 { 3 }

        fn main() {

        }
        "
        .parse()
        .unwrap();
        let pr: Result<Prog> = syn::parse2(ts);
        println!("prog\n{:?}", pr.unwrap());
    }

    #[test]
    fn test_ref_de_ref() {
        let ts: proc_macro2::TokenStream = "
        fn main() {
            let a = &1;
            let mut a = &mut 1;
            *a = *a + 1;
            println!(\"{}\", *a);
        }
        "
        .parse()
        .unwrap();
        let pr: Result<Prog> = syn::parse2(ts);
        println!("prog\n{:?}", pr.unwrap());
    }
}
