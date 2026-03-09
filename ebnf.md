# EBNF Grammar

Terminal symbols: `ident`, `int`, `bool` (`true`/`false`), `string`, punctuation
`+ - * / && || == < > ! = ; , ( ) { } -> fn let mut while if else println! & *`.

## Program

```
Prog        ::= { FnDecl }

FnDecl      ::= `fn` ident `(` Parameters `)` [ `->` Type ] Block

Parameters  ::= /* empty */ | Parameter { `,` Parameter } [ `,` ]
Parameter   ::= [ `mut` ] ident `:` Type

Arguments   ::= /* empty */ | Expr { `,` Expr } [ `,` ]

Block       ::= `{` { Statement } `}`

Statement   ::= `let` [ `mut` ] ident [ `:` Type ] [ `=` Expr ] `;`
              | Expr `=` Expr `;`
              | `while` Expr Block
              | FnDecl
              | Expr                     (* tail expression, no `;` *)
```

## Expressions (precedence low → high)

```
Expr        ::= Or
Or          ::= And { `||` And }
And         ::= Comparison { `&&` Comparison }
Comparison  ::= Sum { ( `==` | `<` | `>` ) Sum }
Sum         ::= Product { ( `+` | `-` ) Product }
Product     ::= Unary { ( `*` | `/` ) Unary }
Unary       ::= ( `!` | `-` | `*` ) Unary | Primary
Primary     ::= literal
              | ident
              | ident `!` `(` Arguments `)`        (* macro call, e.g. println! *)
              | ident `(` Arguments `)`            (* function call *)
              | `&` [ `mut` ] Primary              (* reference *)
              | `(` Expr `)`
              | IfThenElse
              | Block
```

## Control flow

```
IfThenElse  ::= `if` Expr Block [ `else` ( IfThenElse | Block ) ]
```

## Types

```
Type        ::= `i32` | `bool` | `String` | `()`
              | `&` Type                    (* immutable reference *)
              | `&` `mut` Type              (* mutable reference *)
```
