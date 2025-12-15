# d7050e_lab6

A small, educational compiler toolchain for the RnR language. This repository contains a parser, AST representation, type checker, code generator, and optional runtime integration; the lab's goal is to assemble these pieces and provide a usable command-line interface (CLI) to build, inspect and run RnR programs.

## Contents

- `src/` : compiler implementation (parser, AST, type checker, codegen, VM integration, tests).
- `examples/` : small RnR example programs used for testing and demonstration.
- `out.asm` : example output file produced by code generation runs.
- `CHANGELOG.md`, `REFLECTION.md`, `ebnf.md` : project documentation and course deliverables.

## Features
# RnR Compiler — Projet

Ce dépôt contient une chaîne d'outils pédagogique pour le langage RnR : analyse lexicale/ syntaxique, représentation d'AST, vérification de types, génération de code et intégration d'un petit runtime/VM. Le README suivant vise un lecteur extérieur au cours — développeur ou évaluateur souhaitant comprendre, compiler et exécuter le projet.

## Vue d'ensemble

Le projet est une implémentation simple d'un compilateur pour un langage didactique (RnR). Il permet de :

- parser un fichier source RnR en un AST,
- effectuer une vérification de types basique,
- générer une représentation d'assemblage simplifiée (asm) et l'écrire sur disque,
- exécuter le code généré via une VM intégrée (dépendant du support du crate `mips`),
- fournir une interface en ligne de commande (CLI) pour chainer ces étapes.

Objectifs principaux : expérimentation pédagogique, tests d'exemples et exploration du pipeline compilation → exécution.

## Contenu du dépôt

- `Cargo.toml` : configuration du projet Rust et dépendances.
- `src/` : code source principal.
	- `main.rs` : point d'entrée CLI.
	- `parse.rs` : analyseur / parseur.
	- `ast.rs`, `ast_traits.rs` : définitions de l'AST et utilitaires.
	- `type_check.rs` : vérification de types et diagnostics.
	- `codegen.rs` : génération de code (asm simplifié).
	- `vm.rs` : VM/runtime et intégration d'exécution.
	- autres fichiers utilitaires (`env.rs`, `error.rs`, `common.rs`, etc.).
- `examples/` : petits programmes RnR pour démonstration et tests (ex. `gen_add.rs`, `run_print.rs`).
- `out.asm` : exemple de sortie générée par le codegen (peut être régénéré).
- `ebnf.md` : grammaire du langage RnR.
- `CHANGELOG.md`, `REFLECTION.md` : documentation du projet et notes de développement.
- `tests/` : tests unitaires et d'intégration.

## Fonctionnalités principales

- Parsing et construction d'un AST lisible.
- Dump de l'AST vers un fichier pour inspection.
- Vérification de types avec rapports d'erreur (basique).
- Génération d'un assemblage textuel simple et option d'écriture sur disque.
- Exécution du code généré via une VM intégrée (quand disponible).
- CLI pour combiner étapes et options (voir ci‑dessous).

## Installation et compilation

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

- Parse and save the AST:

```bash
cargo run -- -i examples/gen_add.rs -a ast.json
```

- Type check and emit assembly to `out.asm`:

```bash
cargo run -- -i examples/gen_add.rs -t -c -asm out.asm
```

- Generate and execute with VM:

```bash
cargo run -- -i examples/run_print.rs -c -r
```

If the CLI has changed since this README was written, `-h` will show the current options.

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
