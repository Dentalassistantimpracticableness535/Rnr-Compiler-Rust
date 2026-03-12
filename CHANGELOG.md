# Changelog

## Contributors / Reviewers

Peer-review feedback incorporated into the project:

- **Guillaume Darras**: Pointed out that `if` expressions without an `else` should ensure the `then` branch
  is `Unit` when used as a statement expression; suggested a failing test — implemented as
  `if_without_else_then_must_be_unit`.
- **Rasmus Kebert**: Suggested adding `impl Eval<Type> for Block` to simplify tests; reported
  shadowing-related tests which led to clarifications around variable shadowing semantics.
- **Anton Nyström**: Requested end-to-end tests that parse source strings, generate assembly and run the
  generated code; this motivated adding integration tests for codegen + MIPS VM execution.


## Notable Features

- **Mutability enforcement** at both compile time and runtime.
- **Reference support** (`&expr`, `&mut expr`, `*expr`) in AST, type checker, interpreter, and codegen.
- **Function overloading**: multiple `fn` declarations with the same name and different parameter types.
- **Function hoisting**: local `fn` declarations are visible before the line they appear on.
- **Local (nested) functions**: fully supported in interpreter and codegen (MIPS).
- **Short-circuit evaluation** for `&&` and `||`.

---

## Timeline

## 2025-11-23

- Added `src/env.rs`: a stackable environment (stack of scopes) with `push_scope`, `pop_scope`, `insert`, `lookup`, and `update`.
- VM updates (`src/vm.rs`):
  - `VM` now uses `Env` to manage variables and scopes.
  - Implemented evaluation of expressions and statements: binary/unary operators, `let`, `assign`, `while`, `if/else`, and `block` semantics.
  - Simplified support for `println!` (macro) as a special built-in call.
  - Correct block behavior (local scope, return value rules, `semi` flag).
- Parser changes (`src/parse.rs`):
  - `if` and block `{ ... }` constructs are recognized as expressions (handled in `Expr::parse`).
  - `else if` is supported (treated as `else { if ... }`).
  - `parse_ident_or_call` now handles macros `ident!(...)` (e.g. `println!`).
  - Statement termination rules inside blocks: `let` requires `;`, `fn` and `while` do not, expressions/assignments may omit the `;` if they are the last element.

## 2025-11-26

- VM (`src/vm.rs`):
  - Added runtime function values `Val::Fun(FnDeclaration)` and implemented local function calls.
    Calls evaluate arguments, push a new scope, bind parameters (creating mutable bindings for `mut` params),
    evaluate the function body, then pop the scope and return the result.
  - Implemented hoisting of local `fn` declarations at block entry: the VM inserts `FnDecl` bindings
    into the current scope before executing statements, allowing calls to functions defined later in
    the same block.
  - Implemented references and dereference support (`&`, `&mut`, `*`): `&ident` produces
    `Val::RefName(name, is_mut)` (an alias to a named binding) and `&expr` produces
    `Val::RefVal(Box<Val>)` (a boxed value); assignment through dereference is allowed only when
    the referenced cell is mutable.
  - Extended equality (`==`) to compare `int`, `bool`, `string`, and unit values.

- Parser (`src/parse.rs`):
  - Adjusted parsing so that `if` and braced blocks can appear in operand position and participate
    in binary expressions (fixes parsing for expressions like `1 + if ... { } else { } * 2`).

- Display / AST printing (`src/ast_traits.rs`):
  - Centralised block/statement separator printing so `;` placement is consistent between Parse and Display
    (fixes parse(Display(ast)) round-trip mismatches).

- Tests:
  - Integration tests and unit tests were exercised during these changes; the integration test that failed
    due to `if`-operand parsing was fixed by the parser change. (Run `cargo test` to see current results.)

## 2025-11-30

- Type checker (`src/type_check.rs`):
  - Implemented a static type checker covering literals, expressions, statements, blocks, `if`/`while`,
    let-bindings, assignments, and function declarations/calls.
  - Enforced mutability: `let mut x: T = e` binds a mutable variable; assignments to non-`mut` variables
    are rejected at type-check time.
  - Added simple function overloading (polymorphism) at static/type-check time: multiple `fn` declarations
    with the same name are allowed if their parameter-type lists differ. Calls are resolved by exact
    parameter-type matching; ambiguous or missing matches produce type errors.
  - Implemented hoisting and checking of local `fn` declarations during block checking.

- VM and runtime alignment:
  - VM now enforces immutability at runtime: assignments to variables not created as mutable cells will
    raise a runtime error (`assignment to immutable variable 'x'`). The type checker already rejects
    such assignments statically; the VM check provides additional safety at runtime.
  - Function hoisting and overload preservation: `Env::insert_overload` preserves multiple `fn`
    declarations in the same scope by turning a single `Val::Fun` into `Val::Overloads(Vec<FnDeclaration>)`.
    This makes it possible for the VM to store and attempt runtime overload resolution when needed.
  - Runtime overload resolution: when multiple overloads exist for the same name the VM attempts a
    best-effort selection by comparing runtime argument shapes (e.g. `i32`, `bool`, `String`, `()` or reference cells).
    If no unique match is found the VM signals a runtime error (no match or ambiguous match). The static type checker
    still prefers exact, compile-time resolution where possible.
  - References and dereference: `&expr` / `&mut expr` produce `Ref` runtime values. Deref (`*expr`) returns
    the inner value; assignment through dereference is allowed only when the reference cell is mutable.

- Tests & Utilities:
  - Added comprehensive unit tests for scopes, assignments, functions, `if` semantics, and operators.
  - Added test helpers in `src/test_util.rs`: `assert_type`, `assert_type_fail`, and `assert_block_type_fail`.
  - Implemented `Eval<Type>` for `Block` so tests can evaluate blocks through the common test utilities.

- Documentation updates:
  - Updated `ebnf.md` to include reference types (`&Type`, `&mut Type`) integrated into the `Type` rule.

Notes:
- The type checker resolves overloads by exact match only (no subtyping/conversions).
- `fn_missing_return_path_error` is still `#[ignore]` (control-flow return-path analysis not implemented).

See `ebnf.md` for up-to-date grammar documentation.

## 2025-12-07

- Codegen (`src/codegen.rs`):
  - Support for emitting addresses for `&ident` (addressing via `fp` offsets) and lowering of
    assignments through dereference (`*r = v`) into load/store sequences.
  - Single-frame reservation for locals: collect locals first and reserve stack space once per function.
  - String interning into a `.data` section and conditional emission of a `println` runtime stub.
  - `push_reg`/`pop_reg` fixes to correctly use target registers.
  - `gen_block` respects `Block.semi`: only propagates `is_tail` when the last statement is a
    tail expression; pushes unit (0) when the block produces no value. Fixes stack corruption in
    `gen_shadow_nested_blocks` and similar patterns.
  - `Expr::Block` always pushes exactly one value: calls `gen_block(b, true)` for value blocks,
    or `gen_block(b, false)` + push unit for `semi=true` blocks.
  - Bootstrap sequence: `addi sp, zero, 10000` → `bal_label("main")` → `halt()`, so
    `jr ra` from `main` lands on `halt`.
  - `mul`/`div`: textual ASM is emitted normally; `generate_prog_to_instrs` returns `Err(...)` for
    these instructions since the `mips` crate VM does not support them.

- Test infrastructure (`src/common.rs`):
  - Replaced hand-written MIPS simulator with `mips::vm::Mips::new(instrs).run()`.
    `codegen_test` now calls `generate_prog_to_instrs` and runs the real `mips` crate VM.
  - `TestMachine.output` is always `String::new()` — the MIPS VM does not capture I/O output.

- VM semantics (`src/vm.rs`):
  - Reference representation: `Val::RefName(name, is_mut)` for aliasing references to named
    bindings; `Val::RefVal(Box<Val>)` for boxed values from non-ident expressions.
  - Short-circuit evaluation for `&&` and `||`; `BinOp::And`/`BinOp::Or` in `BinOp::eval` are
    `unreachable!()`.
  - `Eval<Val> for Prog` registers all top-level functions before evaluating `main`.

- Tests:
  - `gen_recursive_factorial` and `gen_divide_by_zero_behavior` marked `#[ignore]`
    (mul/div not supported by `mips` crate VM).
  - Codegen `println` tests: dead `m.output` assertions removed (output not captured by MIPS VM).
  - 12 of 13 `ref_deref` integration tests un-ignored (all pass); `gcd_harder` stays ignored
    (cross-scope `RefName` aliasing not yet supported).
  - `short_circuit` integration test un-ignored.

## 2025-12-10

- Codegen (`src/codegen.rs`):
  - **Local (nested) function codegen**: `Statement::Fn` in `gen_stmt` now generates the
    function's code (label, prologue, body, epilogue) instead of silently skipping it.
    A `b skip_fn_X` instruction prevents sequential execution from falling into the function body;
    the enclosing function's codegen state (params, locals, pending locals, next_local offset) is
    saved and restored around the nested `generate_function` call.
  - CLI `-r` flag now displays the return value: `mips run: t0 = <value>`.

- Examples:
  - Fixed `test_vm.rs`: `b_label("main")` → `bal_label("main")` (was causing an infinite loop
    because `b` does not set `ra`, so `jr ra` jumped to address 0).

- Tests:
  - Added `gen_local_fn_inside_main`, `gen_two_local_fns`, `gen_local_fn_with_locals` codegen tests.
  - Total: 207 tests passing, 0 failed, 6 ignored.

- Code quality:
  - `cargo clippy --all-targets` — 0 warnings, 0 errors.
