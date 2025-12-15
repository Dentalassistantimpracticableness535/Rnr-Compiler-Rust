# EBNF Grammar

Terminal symbols: `ident`, `int`, `bool` (`true`/`false`), `string`, punctuation
`+ - * / && || == < > ! = ; , ( ) { } -> fn let mut while if else println! & *`.

Program:

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
                            | Expr (may be final element without `;`)

Expr        ::= Equality
Equality    ::= Logic { `==` Logic }
Logic       ::= And { `||` And }
And         ::= Sum { `&&` Sum }
Sum         ::= Product { ( `+` | `-` ) Product }
Product     ::= Unary { ( `*` | `/` ) Unary }
Unary       ::= ( `!` | `-` | `*` ) Unary | Primary
Primary     ::= literal
                            | ident
                            | ident `!` `(` Arguments `)` 
                            | ident `(` Arguments `)`  
                            | `&` [ `mut` ] Primary 
                            | `(` Expr `)`
                            | IfThenElse
                            | Block

IfThenElse  ::= `if` Expr Block [ `else` ( IfThenElse | Block ) ]

Type        ::= `i32` | `bool` | `String` | `()`

ReferenceType ::= `&` Type | `&mut` Type
