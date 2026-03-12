# Development Reflection

A personal retrospective on building the rnr compiler, what I learned, what was hard, and what I would do differently.

## Phase 1  Getting Started with Rust

- **Toolchain & environment**: I had some difficulties at first with the Rust setup. After figuring out `cargo`, `rustc`, and `rust-analyzer` in VSCode, everything went smoothly, inline error messages, type hints, and auto-completion helped a lot.
- **Data structures**: Learning how `enum`, `struct`, `Vec`, and `Result<T, E>` work in Rust. The `Result` type was confusing at first but after a few exercises it clicked.
- **Ownership and borrowing**: The borrow checker was new to me. It felt strict at first, but I now understand why it's necessary for memory safety. Seeing the difference between `Clone`/`Copy` and move semantics in practice was very helpful.
- **Traits and generics**: `Debug`, `Display`, `Iterator`, and generic types like `Result<i32, Error>` all made sense after working through examples.

## Phase 2 : Parsing

- **Regular expressions vs. real parsing**: I used the `regex` crate for simple pattern matching, but quickly realized that regex can't handle nested structures or operator precedence, you need a proper parser.
- **Building an AST**: Constructing the AST manually (with Rust enums/structs) and then evaluating it was a great introduction. It made me understand how source code gets turned into a structured representation that a compiler can work with.
- **Parsing with `syn`**: Extending the parser to handle operators, precedence, and parentheses was the hardest part. Getting operator precedence right required implementing a recursive descent / precedence climbing parser. I had to think about it with pen and paper before the code worked.
- **EBNF grammar**: Documenting the grammar formally helped me think clearly about what expressions are valid and how to avoid ambiguity.

## Phase 3 : Interpreter (Tree-Walking VM)

- **AST design**: I designed the AST using Rust enums and structs. Each variant of `Expr` and `Statement` maps to a language construct. Having a clean AST made the rest of the work much easier.
- **Scoped environment**: I implemented a stacked environment (`Env`) with push/pop scope. After a peer review I realized it was better to use a single env for both variables and functions, to handle cases where a variable shadows a function name.
- **Evaluation**: The VM evaluates the AST by walking it recursively. Getting blocks right (local scope, return value from last expression, the `semi` flag) was the most time-consuming part. Also implementing function hoisting, where `fn` declarations are visible before the line they appear on, was an interesting concept.
- **Short-circuit evaluation** for `&&` and `||`, chained `else if`, and reference/dereference support (`&`, `&mut`, `*`).

## Phase 4 : Type Checker

- **Semantic analysis**: Type checking catches errors that the grammar alone cannot express (like adding a `bool` to an `i32`). It's another layer of validation before code generation.
- **Type environments**: Very similar to the VM, but tracking types instead of values. I reused the same `Env` structure.
- **Function overloading**: Multiple `fn` declarations with the same name but different parameter types are allowed, and the type checker resolves calls by exact match. Storing a vector of signatures per name was an interesting design choice.
- **Mutability check**: Assignment to a non-`mut` variable is rejected statically. The VM also enforces it at runtime as an extra safety net.

This phase made me appreciate how much work a real type checker does, even for a small language there are many edge cases.

## Phase 5 : Code Generation (MIPS)

- **From AST to assembly**: Going from a high-level tree to load/store/branch instructions made the whole pipeline concrete. Seeing your source code turn into actual machine instructions is satisfying.
- **Stack machine approach**: Evaluate expressions by pushing results onto the stack, then pop operands, compute, push result back. Simple in theory but easy to get wrong, especially the order of push/pop for binary operations.
- **Stack frame layout**: Arguments above `fp`, `ra` and old `fp` saved, locals below `fp`. Getting the offsets right was the most time-consuming part, lots of off-by-one debugging.
- **Local functions in codegen**: A late addition, nested `fn` declarations inside a function body are compiled by emitting their code with a jump-over instruction (so sequential execution doesn't fall through), then saving/restoring the enclosing function's codegen state.
- **The `gen_block` bug**: Blocks without a tail expression (like `{ let x = 3; }`) weren't pushing a unit value onto the stack, which caused stack corruption in nested blocks. Took a while to track down.

Limitation: the `mips` crate VM doesn't support `mul` or `div` instructions, so those tests are ignored. The textual assembly output is still correct.

## Phase 6 : CLI

- **`clap` for argument parsing**: I used `#[derive(Parser)]` and it was very clean, you annotate a struct and `clap` generates parsing, help messages, and validation. Connecting all the previous phases into one binary was satisfying.
- **Flags**: `--input`, `--ast`, `--type_check`, `--vm`, `--code_gen`, `--asm`, `--run`.

## General Takeaways

### What went well
- Seeing a program written in RnR syntax get parsed, type-checked, compiled to MIPS, and executed correctly on the VM. That's the moment where the whole pipeline clicks.
- Peer reviews were genuinely useful, other developers pointed out missing tests and edge cases I hadn't considered.
- Writing tests early and often saved me many times, especially when refactoring the codegen or fixing bugs in the VM.

### What was hard
- Debugging codegen bugs. When a generated program loops forever or gives the wrong result, you have to trace through assembly and register values manually.
- The `gen_block` / semi flag bug took a long time to track down. The symptom was wrong return values from nested blocks, and the cause was deep in the codegen logic.

### What I learned
- The full compilation pipeline from source code to machine instructions, something I had only seen in theory before.
- Much more comfortable with Rust itself: ownership, pattern matching, error handling, trait-based design.
- The `syn` crate for parsing and `clap` for CLI showed me how Rust's ecosystem can make complex tasks clean and ergonomic.
- The biggest takeaway is about testing discipline: every bug fix gets a regression test, and that habit pays off every time you refactor.
