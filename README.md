## RnR Compiler — Project Overview

This repository implements a compiler pipeline for the RnR language. It provides a parser, an AST representation, a simple type checker, a small code generator that emits a textual assembly format (`asm`), and an VM for executing the generated code.

## Repository layout

- `Cargo.toml` — Rust project metadata and dependencies.
- `src/` — source code:
	- `main.rs` — CLI entry point.
	- `parse.rs` — parser implementation.
	- `ast.rs`, `ast_traits.rs` — AST definitions and helpers.
	- `type_check.rs` — basic type checker and diagnostics.
	- `codegen.rs` — code generation to a simple `asm` format.
	- `vm.rs` — VM/runtime integration for executing generated code.
	- other utilities (`env.rs`, `error.rs`, `common.rs`, etc.).
- `examples/` — example RnR programs used for testing and demonstration.
- `out.asm` — example output produced by the code generator (can be regenerated).
- `ebnf.md` — grammar specification for RnR.
- `CHANGELOG.md`, `REFLECTION.md` — development notes and reflections.
- `tests/` — unit and integration tests.

## Features

- Parse source files into an AST and optionally dump the AST to a file.
- Perform a basic type checking pass and report errors.
- Generate a simple textual assembly (`asm`) representation and write it to disk.
- Optionally execute the generated code with a small VM (when dependencies allow).
- A CLI to run individual phases or chain them in a pipeline.

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
rnr -i examples/gen_add.rnr -t -c -asm out.asm
```

If `cargo run` selects a different default binary, you can still call the installed `rnr` directly after `cargo install`.

- If you just want to clean build artifacts before packaging:

```bash
cargo clean
```

If you use the installed binary (`cargo install`), the executable name will be `rnr` (package name and `clap` command name are already set to `rnr`).

## CLI usage (typical options)

Run `cargo run -- -h` to get the current, authoritative option list. Common flags implemented in this repository include:

- `-h`, `--help` — display help.
- `-i`, `--input <path>` — the RnR source file to compile (defaults to `main.rs` if omitted).
- `-a`, `--ast <path>` — write the parsed AST to `<path>`.
- `-t`, `--type_check` — run the type checker.
- `-c`, `--code_gen` — run code generation.
- `-asm <path>` — write generated assembly to `<path>`.
- `-vm`, `--virtual_machine` — execute generated code with the integrated VM.
- `-r` — run the generated `asm` with the runtime/VM (when supported).

Examples:

- Parse and save the AST (use the RnR source file):

```bash
cargo run -- -i examples/gen_add.rnr -a ast.json
```

- Type check and emit assembly to `out.asm`:

```bash
cargo run -- -i examples/gen_add.rnr -t -c -asm out.asm
```

- Generate and execute with VM:

```bash
cargo run -- -i examples/run_print.rnr -c -r
```

`-h` will show the current options.

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
