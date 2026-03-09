# Reflection

## Lab 1 — Introduction to Rust

- **Basic understanding of the Rust tool-chain installation and use.**
  -> I had some difficulties at first, I used ChatGPT to help me with the setup. After that it went smoothly with cargo.

- **Use of editing environment for Rust development.**
  -> VSCode with rust-analyzer was very helpful : the inline error messages, type hints, and auto-completion helped me understand faster what was going on.

- **How to organize data in structs, enums, arrays, and vectors.**
  -> I discovered how these structures work in Rust. The enum `Result<T, E>` was a bit confusing at first but now I get it.

- **Use of macros (e.g., `println!`, `vec!`)**
  -> All was clear, nothing too surprising.

- **The use of Traits, Debug, Iterator**
  -> Same, everything was understandable. I saw how Debug gives custom formatting and how iterators work on vectors.

- **Use of generics**
  -> I was a bit surprised by `Result<i32, Error>` and didn't fully understand how it worked at first, but after the exercises it clicked.

- **The Clone trait and the Copy trait in relation to "move" and "copy" behavior.**
  -> This part was well explained. Seeing the difference before/after implementing Clone/Copy really helped me understand ownership.

- **References and borrow checking.**
  -> The borrow system was new to me. It seemed strict but I understand now why its necessary for memory safety.

## Lab 2 — Parsing to an AST

- **Regular expressions and their limitations.**
  -> I used the `regex` crate to match patterns in strings. It works for simple stuff but I quickly saw that regex can't handle nested structures or operator precedence : you need a real parser for that.

- **Simple AST for expressions.**
  -> Building an AST manually (just creating the tree by hand with structs/enums) and then evaluating it was a good introduction. It made me realize that this is how source code gets turned into something the computer can work with.

- **Parsing with `syn` and `TokenStream`.**
  -> Extending the parser to handle mul/div and parentheses was the tricky part. Getting operator precedence right took me some time, I had to think about it with pen and paper to understand the recursive descent approach.

- **EBNF specification.**
  -> Documenting the grammar helped me think more clearly about what expressions are valid and how to avoid ambiguity.

## Lab 3 — Parser & VM (Natural interpretation)

- **Abstract Syntax Tree (AST) to represent RNR.**
  -> I designed the AST using Rust enums and structs. Each variant of `Expr` and `Statement` maps to a language construct (if, while, function call, etc.). Having a clean AST made the rest of the work much easier.

- **Parsing from Rust to AST.**
  -> I implemented `Parse` for all the AST types using the `syn` crate. The hardest part was getting operator precedence right : I had to implement a proper precedence climbing parser to handle things like `2 + 3 * 4` correctly. Also, making `if` and blocks work as expressions inside binary operations was tricky.

- **Variable environment representing state.**
  -> I implemented a stacked environment (`Env`) with push/pop scope. At first I had separate environments for variables and functions, but after a peer review I realized it was better to use a single env to handle cases where a variable shadows a function name.

- **Natural interpretation.**
  -> The VM evaluates the AST by walking it recursively. Getting blocks right (local scope, return value from last expression, the `semi` flag) was probably the thing that took the most time. Also implementing function hoisting, where `fn` declarations are visible before the line they appear on, was an interesting concept I hadn't thought about before.

I also implemented short-circuit evaluation for `&&` and `||` and chained `else if`. Arrays are not supported.

## Lab 4 — Type Checker

- **The role of semantic analysis in a compiler.**
  -> I understood that type checking happens after parsing and catches errors that the grammar alone cannot express (like adding a bool to an int). Its basically another layer of validation before you can generate code.

- **The basics of type environments and type checking.**
  -> Very similar to the VM but instead of values I track types. I reused the same `Env` structure with `push_scope`/`pop_scope`. The `unify` function checks that two types match and reports an error otherwise.

- **Type inference and simple polymorphism (overloading).**
  -> I implemented function overloading: you can have multiple `fn` with the same name but different parameter types, and the type checker resolves calls by exact match. This was interesting to implement, I store a vector of signatures per name in the `FunEnv`.

- **Mutability check.**
  -> Assignment to a non-`mut` variable is rejected at type-check time. I also enforce it at runtime in the VM as an extra safety net, but the static check is the main one.

Overall this lab made me appreciate how much work a real type checker does. Even for our small language there are a lot of edge cases to handle.

## Lab 5 — Code Generation (MIPS)

- **The role of backend code generation in a compiler.**
  -> Going from AST to actual assembly was interesting. It really made the whole pipeline concrete, you see your high-level code turn into load/store/add/branch instructions.

- **RISC-like instruction sets and memory regions.**
  -> I already had some notions from a previous course, but working with the `mips` crate and implementing the stack frame layout in practice was a good refresher. Having to manage sp, fp, ra myself made me understand how fragile low-level code is.

- **The basics of stack machines.**
  -> The approach is: evaluate expressions by pushing results onto the stack, then pop operands, compute, push result back. It's simple in theory but in practice you have to be very careful with the order of push/pop, especially for binary operations where left and right operands matter.

- **Managing state for code generation (function frames).**
  -> This was the most time-consuming part. The stack frame layout (arguments above fp, ra and old_fp saved, locals below fp) has to be exact or everything breaks. I spent a lot of time debugging off-by-one errors in fp-relative offsets.

- **Automated code generation for AST constructs.**
  -> I wrote a `CodeGen` struct that traverses the AST recursively, similar in structure to the type checker. For each node type I emit the corresponding MIPS instructions. The `gen_block` function was especially tricky : I had a bug where blocks without a tail expression (like `{ let x = 3; }`) didn't push a unit value onto the stack, which caused stack corruption in nested blocks.

One limitation: the `mips` crate VM doesn't support `mul` or `div` instructions, so those tests are ignored. The textual assembly output is still correct though.

## Lab 6 — CLI

- **Rudimentary shell/terminal interaction.**
  -> I created a binary `rnr` that reads a source file, parses it, and optionally runs type checking, the VM, and/or code generation depending on the flags.

- **Command line parsing.**
  -> I used the `clap` crate with the `#[derive(Parser)]` macro. It was really nice to use, you just annotate a struct and clap generates the argument parsing, help messages, and validation for you. Connecting all the previous labs into one program was satisfying.

The flags are: `--input`, `--ast`, `--type_check`, `--vm`, `--code_gen`, `--asm`, `--run`. The instructions mentioned `-vm` and `-asm` as short flags but `clap` only supports single-character short flags, so I kept them as long flags only.

## General reflection

### Highs
- Seeing a program I wrote in RnR syntax get parsed, type-checked, compiled to MIPS, and then run correctly on the VM. Thats the moment where the whole pipeline clicks.
- The peer reviews were really useful, other students pointed out missing tests and edge cases I hadn't considered (like checking runtime output, or testing function hoisting properly).

### Lows
- Debugging codegen bugs. When a generated program loops forever or gives the wrong result, you have to stare at the assembly and trace through register values manually. It's tedious but it teaches you to be careful.
- The `gen_block` / semi flag bug took me a while to track down. The symptom was wrong return values from nested blocks, and the cause was deep in the codegen logic.

### Did I learn something new?
Yes : I learned the full pipeline from source code to machine instructions, which I had only seen in theory before. I also got much more comfortable with Rust itself (ownership, pattern matching, error handling). Working with `syn` for parsing was interesting, and using `clap` for the CLI showed me how Rust crates can make things that would be tedious by hand very clean.

The biggest takeaway is probably about testing: writing tests early and often saved me many times, especially when refactoring the codegen or fixing bugs in the VM. Every time I fixed something I added a regression test, and that habit paid off.
