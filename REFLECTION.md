# Reflection

## What I learned

- **Full compiler pipeline**: parsing with `syn`, building a typed AST, type checking,
  tree-walking interpretation, and MIPS code generation — each stage has its own set of invariants.
- **MIPS code generation**: stack-frame conventions (`$ra`, `$fp`, `$sp`), register allocation
  for temporaries, and how high-level constructs (if/else, while, function calls, references)
  map to branch/jump instructions.
- **Reference semantics**: implementing `&` and `&mut` required careful thought in every layer —
  parsing (`&Type`, `&mut Type`), type checking (mutability rules), the VM (`RefVal`/`RefName`
  variants), and codegen (address-of vs load-word).
- **Testing discipline**: writing small focused tests for each feature, checking failure cases,
  and using integration tests to catch regressions across the pipeline.
- **Tooling**: `clap` for CLI, `syn`/`quote`/`proc-macro2` for parsing, and the `mips` crate
  for executing generated code in a sandboxed MIPS simulator.

## Main challenges

- **MIPS simulator integration**: replacing a hand-written simulator with `mips::vm::Mips::new(instrs).run()`
  required adapting the codegen bootstrap sequence (`addi sp, zero, 10000` → `bal main` → `halt`)
  and fixing register conventions.
- **Short-circuit evaluation**: `&&` and `||` must skip the right operand at the VM level and
  generate branch instructions at the codegen level — easy to get wrong.
- **Function overloading**: the type checker must match calls to the correct overload based on
  argument types; the VM must register all top-level functions before evaluation.
- **Reference/deref in codegen**: `&x` must emit the *address* of a local, while `*x` must
  emit a `lw` through the stored pointer. Getting the stack offsets right was tricky.
- **println! output**: the MIPS VM does not capture `println!` output into a string, so codegen
  tests can only assert on register values and ASM structure, not printed output.

## How I solved problems

- Compared my implementation against two reference projects to identify gaps.
- Added targeted tests for each missing feature before writing the fix.
- Used `cargo test` constantly — a failing test was usually enough to pinpoint the bug.
- Traced through generated ASM by hand when the MIPS VM reported unexpected register values.

## Highs and lows

- **High**: seeing the full pipeline work end-to-end — parse → type-check → codegen → MIPS execution
  returning the correct value in `$v0`.
- **High**: reference/deref passing all 12 integration tests after implementing `RefName`/`RefVal`.
- **Low**: long debugging sessions on codegen register ordering and off-by-one stack offsets.
- **Low**: discovering that `mul`/`div` are not available as single MIPS instructions in the
  `mips` crate, which limits some tests.

## What I would do differently

- Start with the MIPS crate from the beginning instead of writing a hand-rolled simulator.
- Write codegen tests that check register state rather than printed output — more robust.
- Add property-based tests for the Display/Parse round-trip early on.
