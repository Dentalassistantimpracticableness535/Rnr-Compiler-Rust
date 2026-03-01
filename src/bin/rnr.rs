use std::fs;
use std::path::PathBuf;

use clap::Parser;

/// CLI for the RnR compiler
#[derive(Parser, Debug)]
#[command(name = "rnr")]
struct Args {
    /// Input file to compile (default: main.rs)
    #[arg(short = 'i', long = "input")]
    input: Option<PathBuf>,

    /// Dump parsed AST to file
    #[arg(short = 'a', long = "ast")]
    ast: Option<PathBuf>,

    /// Run type checker
    #[arg(short = 't', long = "type_check")]
    type_check: bool,

    /// Run interpreter VM over AST
    #[arg(long = "vm")]
    vm: bool,

    /// Run code generation (generate asm)
    #[arg(short = 'c', long = "code_gen")]
    code_gen: bool,

    /// Dump generated asm to file
    #[arg(long = "asm")]
    asm: Option<PathBuf>,

    /// Run generated asm using simple asm runner
    #[arg(short = 'r', long = "run")]
    run: bool,
}

fn main() {
    let args = Args::parse();

    let input = args.input.unwrap_or_else(|| PathBuf::from("main.rs"));
    let src = match fs::read_to_string(&input) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to read input file '{}': {}", input.display(), e);
            std::process::exit(1);
        }
    };

    // Parse
    let prog_res = rnr::parse::try_parse::<rnr::ast::Prog>(&src);
    let prog = match prog_res {
        Ok(p) => p,
        Err(e) => {
            eprintln!("parse error: {}", e);
            std::process::exit(1);
        }
    };

    // Optionally dump AST (use Debug formatting)
    if let Some(path) = args.ast {
        if let Err(e) = fs::write(&path, format!("{:#?}", prog)) {
            eprintln!("failed to write AST to '{}': {}", path.display(), e);
            std::process::exit(1);
        }
        println!("Wrote AST to {}", path.display());
    }

    // Type check
    if args.type_check {
        let mut tc = rnr::type_check::TypeChecker::new();
        if let Err(e) = tc.check_prog(&prog) {
            eprintln!("type error: {}", e);
            std::process::exit(1);
        } else {
            println!("type check: OK");
        }
    }

    // Interpreter VM (evaluate AST)
    if args.vm {
        match <rnr::ast::Prog as rnr::common::Eval<rnr::vm::Val>>::eval(
            &prog,
        ) {
            Ok(v) => println!("vm result: {:?}", v),
            Err(e) => {
                eprintln!("vm error: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Codegen
    if args.code_gen || args.asm.is_some() || args.run {
        match rnr::codegen::generate_prog_to_string(&prog) {
            Ok(asm) => {
                if args.code_gen {
                    println!("--- Generated ASM ---\n{}\n--- End ASM ---", asm);
                }
                if let Some(path) = args.asm {
                    if let Err(e) = fs::write(&path, &asm) {
                        eprintln!("failed to write asm to '{}': {}", path.display(), e);
                        std::process::exit(1);
                    }
                    println!("Wrote asm to {}", path.display());
                }
                if args.run {
                    match rnr::codegen::generate_prog_to_instrs(&prog) {
                        Ok(instrs) => {
                            let mut m = mips::vm::Mips::new(instrs);
                            match m.run() {
                                Ok(_) => println!("Program completed"),
                                Err(mips::error::Error::Halt) => {
                                    println!("Program completed (halt)")
                                }
                                Err(e) => {
                                    eprintln!("runtime error: {:?}", e);
                                    std::process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("codegen->mips error: {:?}", e);
                            std::process::exit(1);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("codegen error: {}", e);
                std::process::exit(1);
            }
        }
    }
}
