# Changelog for RNR

YOUR CHANGES/ADDED FEATURES HERE


Note : I used AI to help to write this

  ## Contributors / Reviewers

  The following peer-review comments were incorporated into the project. Each entry lists the
  reviewer identifier and a short description of their contribution or suggestion (see `reviews.txt`).

  - Guillaume Darras: pointed out that `if` expressions without an `else` should ensure the `then` branch
    is `Unit` when used as a statement expression; suggested a failing test and we implemented
    `if_without_else_then_must_be_unit` to enforce this behavior.
  - Rasmus Kebert: suggested adding `impl crate::common::Eval<Type> for Block` (and similar helpers)
    to simplify tests; reported two shadowing-related tests which led to checks and clarifications
    around variable shadowing semantics in the type checker and `insert` behavior.
  - Anton Nyström: requested end-to-end tests that parse source strings, generate assembly and run the
    generated code to verify runtime behaviour; this motivated adding integration tests for codegen+VM.
  - Lisa QUANTIN: asked for tests that also check runtime output (not just generated ASM) to better
    validate scoping and local-variable behaviour; inspired creation of capture-enabled println tests
    and improvements to the test harness so outputs can be asserted programmatically.


## Additional features implemented

- Mutability enforcement at runtime: the VM rejects assignment to variables that were not created as mutable cells (`let mut` or `mut` parameters).
- Reference support in AST/VM/type-checker: `&expr` / `&mut expr` and `*expr` with typed `Ref` in the static checker and `Val::Ref` / `Val::Mut` at runtime; assignment-through-deref checks.
- Function overloading preserved at runtime: `Env::insert_overload` and `Val::Overloads` allow multiple `fn` declarations with the same name to coexist; the VM attempts best-effort overload selection by runtime argument shapes when static resolution is not sufficient.
- Hoisting + overload-aware hoisting: local `fn` declarations are hoisted into the block's scope before statement execution and hoisting preserves overload groups.


## Timeline global features

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
    Calls evaluate arguments, push a new scope, bind parameters (creating `Val::Mut` for `mut` params),
    evaluate the function body, then pop the scope and return the result.
  - Implemented hoisting of local `fn` declarations at block entry: the VM inserts `FnDecl` bindings
    into the current scope before executing statements, allowing calls to functions defined later in
    the same block.
  - Implemented references and dereference support (`&`, `&mut`, `*`), including assignment through
    dereference when the referenced cell is mutable.
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
  - Updated `ebnf.md` to include reference types and clarify the syntax/semantics of `&`/`&mut` and `mut` parameters/locals.
  - Updated `sos.md` to explain runtime immutability enforcement, assignment-through-deref semantics, and static vs runtime overload resolution.

Notes:
- The type checker currently resolves overloads by exact match only (no subtyping/conversions).
- Advanced control-flow/return-path analysis is still TODO (some tests are still `#[ignore]`).

Notes and next steps:
- The VM now supports local function calls and references; update to type checking and further builtins can follow.
- Recommended next steps: run the full test suite locally (`cargo test`) or enable CI to ensure regressions are caught.

See `ebnf.md` and `sos.md` for up-to-date grammar and semantics documentation.

## 2025-12-07

- Codegen improvements:
  - Support for emitting addresses for `&ident` (addressing via `fp` offsets) and lowering of
    assignments through dereference (`*r = v`) into load/store sequences.
  - Single-frame reservation for locals: collect locals first and reserve stack space once per function.
  - String interning into a `.data` section and conditional emission of a `println` runtime stub when a
    `bal println` call is detected (detection implemented by tokenizing emitted instruction lines).
  - `push_reg`/`pop_reg` fixes to correctly use target registers.

- Runtime / asm runner:
   - Replaced the ad-hoc textual `asm_runner` with execution via the `mips` crate; codegen can
     now convert its textual ASM into `mips::instrs::Instrs` (see `codegen::generate_prog_to_instrs`) and
     run on `mips::vm::Mips` for more realistic testing.

- VM and semantics:
  - Reference representation refined: `RefName(name, is_mut)` for aliasing references to named bindings
    and `RefVal(Box<Val>)` for boxed reference values created from non-ident expressions. Deref/assign
    semantics updated accordingly.
  - `let` without initializer now supported: variables without an initializer are bound to a default
    value (e.g. `0` for integers or `()` for unit); annotated `let id: T;` is accepted when a default for `T` is defined.

- Tests & documentation:
  - Added/updated unit and integration tests for codegen, VM, and runtime behaviors (including deref-assignment tests).
  - Added documentation files: `SUBMISSION_REPORT.md`, `ASM_RUNNER_EXPL_FR.md`, `ASM_RUNNER_WALKTHROUGH_FR.md`,
    `TESTS_RUNTIME_CALLS_WALKTHROUGH_FR.md`, and `TESTS_CODEGEN_WALKTHROUGH_FR.md`.
