## RnR Compiler — Project Overview

This repository implements a compiler pipeline for the RnR language. It provides a parser, an AST representation, a simple type checker, a small code generator that emits a textual assembly format (`asm`), and an VM for executing the generated code.

## Repository layout

- `Cargo.toml` — Rust project metadata and dependencies (`rnr` package).
- `src/` — source code:
	- `bin/rnr.rs` — CLI entry point (binary).
	- `lib.rs` — library root, re-exports all modules.
	- `parse.rs` — parser implementation (uses `syn` crate).
	- `ast.rs`, `ast_traits.rs` — AST definitions, Display/Parse traits.
	- `type_check.rs` — type checker with function overload support.
	- `codegen.rs` — MIPS code generation (text ASM → `mips` crate `Instrs`).
	- `vm.rs` — tree-walking interpreter with reference/deref support.
	- `env.rs` — scoped environment for the interpreter.
	- `intrinsics.rs` — built-in functions (`println!`, `assert_eq!`).
	- `common.rs` — shared traits (`Eval`) and test helpers.
	- `error.rs`, `test_util.rs` — error type and test macros.
- `examples/` — example RnR programs (`.rnr` files) and Rust driver examples.
- `tests/` — integration and unit tests:
	- `integration_tests.rs` — parser, VM, Display round-trip tests.
	- `codegen_tests.rs` — MIPS code generation and execution tests.
	- `type_check_tests.rs` — type checker tests.
	- `runtime_call_tests.rs` — VM function call tests.
- `ebnf.md` — grammar specification for RnR.
- `CHANGELOG.md`, `REFLECTION.md` — development notes.
- `COMPARISON_REPORT.md` — detailed comparison with reference implementations.

## Features

- **Parser**: Parses RnR source into a typed AST using the `syn` crate. Supports
  functions, let/mut bindings, if/else, while, references (`&`, `&mut`), deref (`*`),
  `println!` with format strings, and operator precedence.
- **Type checker**: Checks types for expressions, statements, and function bodies.
  Supports function overloading. Reference types (`&i32`, `&mut i32`) are fully supported.
- **Interpreter (VM)**: Tree-walking evaluator with scoped environments, mutable references,
  short-circuit `&&`/`||`, late initialization, and function hoisting.
- **MIPS code generator**: Emits textual MIPS assembly, converts to `mips` crate `Instrs`,
  and executes via `Mips::new(instrs).run()`. Supports function calls, stack frames,
  references/deref, while loops, and conditionals.
- **CLI**: Parse, type-check, interpret, generate ASM, and run — all from a single binary.
- **Display/Parse round-trip**: All AST nodes implement `Display` and `Parse`.

## Building

Requirements: a recent Rust toolchain (rustc and cargo).

To build the project:

```bash
cargo build --release
```

To run the project in development mode:

```bash
cargo run -- [OPTIONS]
```

## Running the CLI

Commands to build and run the CLI and to run the example RnR files shipped in `examples/`.

- Build the project (release):

```bash
cargo build --release
```

- Run the compiler via `cargo run` :

```bash
cargo run -- -i examples/gen_add.rnr -a ast.json
```

- Generate assembly and run it with the VM (development run):

```bash
cargo run -- -i examples/run_print.rnr -c -r
```

- Install a local `rnr` binary (so you can call `rnr` directly):

```bash
cargo install --path .
# then run:
rnr -i examples/gen_add.rnr -t -c --asm out.asm
```

If `cargo run` selects a different default binary, you can still call the installed `rnr` directly after `cargo install`.

- If you just want to clean build artifacts before packaging:

```bash
cargo clean
```

If you use the installed binary (`cargo install`), the executable name will be `rnr` (package name and `clap` command name are already set to `rnr`).

## CLI usage

Run `cargo run -- -h` to get the full option list. 
Available flags :

- `-h`, `--help` — display help.
- `-i`, `--input <path>` — the RnR source file to compile (defaults to `main.rs` if omitted).
- `-a`, `--ast <path>` — write the parsed AST to `<path>`.
- `-t`, `--type_check` — run the type checker.
- `-c`, `--code_gen` — MIPS code generation and print ASM.
- `--asm <path>` — write generated assembly to `<path>`.
- `-vm` — execute generated code with the integrated VM.
- `-r`, `--run` — run the generated `asm` with the runtime/VM (when supported).


Examples:

```bash
# Parse and save the AST
cargo run -- -i examples/gen_add.rnr -a ast.json

# Type check and emit assembly
cargo run -- -i examples/gen_add.rnr -t -c --asm out.asm

# Generate and execute with MIPS VM
cargo run -- -i examples/run_print.rnr -c -r

# Run the tree-walking interpreter
cargo run -- -i examples/gen_add.rnr --vm
```

## Tests and development

Run tests:

```bash
cargo test
```

Formatting and linting:

```bash
cargo fmt
cargo clippy
```

## Notes
- IA was used to write this document (Claude Sonnet 4.6)