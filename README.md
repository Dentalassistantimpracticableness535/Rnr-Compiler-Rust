# rnr — A Compiler for a Rust-like Language

**rnr** is a compiler toolchain for a small statically-typed language with Rust-like syntax. It covers the full compilation pipeline, from source code to MIPS machine instructions, and includes a tree-walking interpreter for direct execution.

Built entirely in Rust as a university compiler-construction project (D7050E, Luleå University of Technology), then refined and extended independently.

## The RnR Language

RnR is a small imperative language with Rust-inspired syntax. Example:

```rust
fn gcd(a: i32, b: i32) -> i32 {
    if b == 0 {
        a
    } else {
        gcd(b, a - b * (a / b))
    }
}

fn main() -> i32 {
    gcd(48, 18)
}
```

**Supported features:**
- Functions (top-level and nested/local), recursion, mutual recursion
- `let` / `let mut` bindings with optional type annotations
- Arithmetic (`+`, `-`, `*`, `/`), comparisons (`==`, `<`, `>`), logic (`&&`, `||`, `!`)
- `if` / `else if` / `else` (as expressions), `while` loops
- References (`&x`, `&mut x`) and dereference (`*r`)
- `println!` with format strings
- Function overloading (dispatch by parameter types)
- Short-circuit evaluation for `&&` and `||`

See [ebnf.md](ebnf.md) for the full grammar specification.

## Architecture

```
Source (.rnr)
    │
    ▼
┌──────────┐     ┌────────────┐     ┌───────────┐     ┌──────────────┐
│  Parser   │────▶│    AST     │────▶│   Type    │────▶│  Code Gen    │
│  (syn)    │     │            │     │  Checker  │     │  (MIPS)      │
└──────────┘     └────────────┘     └───────────┘     └──────┬───────┘
                       │                                      │
                       ▼                                      ▼
                 ┌───────────┐                         ┌──────────────┐
                 │Interpreter│                         │   MIPS VM    │
                 │(tree-walk)│                         │ (execution)  │
                 └───────────┘                         └──────────────┘
```

| Stage | Module | Description |
|-------|--------|-------------|
| **Parsing** | `parse.rs` | Recursive descent parser built on the `syn` crate. Handles operator precedence, blocks-as-expressions, references, macros. |
| **AST** | `ast.rs`, `ast_traits.rs` | Typed AST with `Display` and `Parse` round-trip support. |
| **Type Checking** | `type_check.rs` | Static type checker with scoped environments, mutability enforcement, function overload resolution, and reference types (`&T`, `&mut T`). |
| **Interpreter** | `vm.rs`, `env.rs` | Tree-walking evaluator with scoped environments, mutable references, function hoisting, and short-circuit evaluation. |
| **Code Generation** | `codegen.rs` | Emits MIPS assembly (text), then assembles to machine instructions via the `mips` crate. Stack-frame based: arguments above `fp`, saved `ra`/`fp`, locals below `fp`. |
| **Intrinsics** | `intrinsics.rs` | Built-in functions (`println!`, `assert_eq!`). |
| **CLI** | `bin/rnr.rs` | Single binary to parse, type-check, interpret, compile, and run. Built with `clap`. |

## Building

Requires a recent Rust toolchain (1.70+).

```bash
cargo build --release
```

## Usage

```bash
# Parse a file and dump the AST
cargo run -- -i examples/gen_add.rnr -a ast.json

# Type-check a program
cargo run -- -i examples/gen_add.rnr -t

# Run with the tree-walking interpreter
cargo run -- -i examples/gen_add.rnr --vm

# Compile to MIPS assembly and display it
cargo run -- -i examples/gen_add.rnr -c

# Compile to MIPS, save assembly, and execute
cargo run -- -i examples/gen_add.rnr -c --asm out.asm -r
```

### CLI Flags

| Flag | Description |
|------|-------------|
| `-i`, `--input <path>` | Source file to compile |
| `-a`, `--ast <path>` | Write parsed AST to file |
| `-t`, `--type_check` | Run the static type checker |
| `--vm` | Execute with the tree-walking interpreter |
| `-c`, `--code_gen` | Generate and display MIPS assembly |
| `--asm <path>` | Write generated assembly to file |
| `-r`, `--run` | Compile to MIPS and execute on the VM |

## Testing

The project has **207 tests** covering all stages of the pipeline:

```bash
cargo test
```

| Suite | Tests | Coverage |
|-------|-------|----------|
| Unit tests (parser, VM, env, AST traits) | 112 | Parsing, evaluation, scoping, references |
| Codegen tests | 25 (+2 ignored) | MIPS generation, execution, stack frames, local functions |
| Integration tests | 39 (+3 ignored) | End-to-end: parse → type-check → interpret |
| Type checker tests | 30 (+1 ignored) | Static type errors, overloading, mutability |
| Runtime call tests | 1 | VM function call semantics |

```bash
# Lint — 0 warnings
cargo clippy --all-targets

# Format check
cargo fmt --check
```

## Project Structure

```
rnr/
├── src/
│   ├── bin/
│   │   └── rnr.rs          # CLI binary — argument parsing, pipeline orchestration
│   ├── lib.rs              # Library root, re-exports all modules
│   ├── ast.rs              # AST node definitions (Prog, FnDeclaration, Expr, Statement, …)
│   ├── ast_traits.rs       # Display and Parse implementations for all AST nodes
│   ├── parse.rs            # Recursive descent parser (built on the syn token-stream crate)
│   ├── type_check.rs       # Static type checker: scoped environments, overload resolution, mutability
│   ├── vm.rs               # Tree-walking interpreter: evaluation, scopes, references, hoisting
│   ├── env.rs              # Generic scoped environment (push/pop scope, lookup, update)
│   ├── codegen.rs          # MIPS code generator: emit text ASM → assemble to mips::Instrs → run
│   ├── intrinsics.rs       # Built-in functions: println!, assert_eq!
│   ├── common.rs           # Shared Eval trait, TestMachine helper for codegen tests
│   ├── error.rs            # Unified Error type (String alias + From impls)
│   └── test_util.rs        # assert_eval!, assert_type! and similar test macros
├── tests/
│   ├── integration_tests.rs    # End-to-end: parse → type-check → interpret (42 tests)
│   ├── codegen_tests.rs        # MIPS codegen + execution tests (27 tests)
│   ├── type_check_tests.rs     # Static type checker tests (31 tests)
│   └── runtime_call_tests.rs   # VM function-call semantics (1 test)
├── examples/
│   ├── gen_add.rnr         # Simple add function: top-level fn call
│   ├── run_print.rnr       # println! demo
│   ├── gen_add.rs          # Rust driver: programmatic codegen for gen_add
│   ├── run_print.rs        # Rust driver: programmatic codegen for run_print
│   └── test_vm.rs          # Rust driver: hand-assembled MIPS test
├── ebnf.md                 # Formal EBNF grammar of the RnR language
├── CHANGELOG.md            # Development history and notable changes
├── REFLECTION.md           # Personal retrospective on the build process
├── Cargo.toml              # Package metadata and dependencies
└── LICENSE                 # MIT license
```

## Known Limitations

- The `mips` crate VM does not support `mul`/`div` instructions, multiplication and division generate correct assembly but cannot be executed on the VM.
- No closures: local functions cannot capture variables from their enclosing scope.
- No arrays, structs, or heap allocation.
- Type error messages do not include source line numbers.

## Dependencies

- [`syn`](https://crates.io/crates/syn): Rust token stream parsing
- [`clap`](https://crates.io/crates/clap): CLI argument parsing
- [`mips`](https://vesuvio-git.neteq.ltu.se/pln/mips.git): MIPS assembler and VM
- [`regex`](https://crates.io/crates/regex): Pattern matching for intrinsics

## License

MIT: see [LICENSE](LICENSE).
