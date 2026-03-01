# Reflection

## What I learned

- Compiler pipeline : parsing, building an AST, type checking, code generation, and testing on a VM.
- Basics of low-level code : stack frames, return addresses, ... 
- Testing habits : write small tests, check failures, and add integration tests to catch regressions.
- Tools : using `clap` for CLI, `cargo test`, and running programs with the `mips` VM.

## Main challenges

- Small bugs in code generation (register order, immediates) cause runtime failures.
- Debugging runtime loops and overflow checks required careful inspection of generated asm.

## How I solved problems

- Run tests often and add small focused tests for each bug.
- Use peer reviews to find missing tests and edge cases.

## Highs and lows

- High: seeing generated code run and produce correct output.
- Low: long debugging sessions, but they teach careful reasoning.

Did I learn anything new? -> yes I learned more about the relation between high-level code and low-level assembly, and how
tools and tests help keep the compiler correct.
