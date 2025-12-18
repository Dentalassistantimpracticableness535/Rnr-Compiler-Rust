use crate::ast::{Block, FnDeclaration, Literal, Mutable, Parameter, Parameters, Type};
use crate::error::Error;
use crate::vm::Val;
use regex::Regex;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::thread::ThreadId;

pub type Intrinsic = fn(Vec<Val>) -> Result<Val, Error>;

static OUTPUT_CAPTURE: OnceLock<Mutex<HashMap<ThreadId, String>>> = OnceLock::new();

pub fn start_capture() {
    let m = OUTPUT_CAPTURE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = m.lock().unwrap();
    guard.insert(std::thread::current().id(), String::new());
}

pub fn take_capture() -> Option<String> {
    if let Some(m) = OUTPUT_CAPTURE.get() {
        let mut guard = m.lock().unwrap();
        guard.remove(&std::thread::current().id())
    } else {
        None
    }
}

fn println_intrinsic(args: Vec<Val>) -> Result<Val, Error> {
    if args.is_empty() {
        if let Some(m) = OUTPUT_CAPTURE.get() {
            if let Some(s) = m.lock().unwrap().get_mut(&std::thread::current().id()) {
                s.push('\n');
                return Ok(Val::Lit(Literal::Unit));
            }
        }
        println!();
        return Ok(Val::Lit(Literal::Unit));
    }

    // 1st arg must be string
    let fmt = match &args[0] {
        Val::Lit(Literal::String(s)) => s.clone(),
        _ => return Err("ICE - no formatting string in println!".to_string()),
    };

    let re = Regex::new(r"\{(:\?)?\}").unwrap();
    let parts: Vec<&str> = re.split(&fmt).collect();

    // to convert Val into printable string
    fn val_to_string(v: &Val) -> String {
        match v {
            Val::Lit(Literal::String(s)) => s.clone(),
            Val::Lit(Literal::Int(i)) => i.to_string(),
            Val::Lit(Literal::Bool(b)) => b.to_string(),
            Val::Lit(Literal::Unit) => "()".to_string(),
            Val::Mut(inner) => val_to_string(inner),
            Val::RefVal(boxed) => val_to_string(boxed),
            other => format!("{:?}", other),
        }
    }

    // Build the final string
    let mut out = String::new();
    out.push_str(parts.first().copied().unwrap_or(""));
    for (text, v) in parts.iter().skip(1).zip(args.iter().skip(1)) {
        out.push_str(&val_to_string(v));
        out.push_str(text);
    }
    out.push('\n');

    if let Some(m) = OUTPUT_CAPTURE.get() {
        if let Some(s) = m.lock().unwrap().get_mut(&std::thread::current().id()) {
            s.push_str(&out);
            return Ok(Val::Lit(Literal::Unit));
        }
    }

    print!("{}", out);
    Ok(Val::Lit(Literal::Unit))
}

pub fn vm_println() -> (FnDeclaration, Intrinsic) {
    (
        FnDeclaration {
            id: "println!".to_string(),
            parameters: Parameters(vec![Parameter {
                mutable: Mutable(false),
                id: "str".to_string(),
                ty: Type::String,
            }]),
            ty: None,
            body: Block {
                statements: vec![],
                semi: false,
            },
        },
        println_intrinsic,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn regex_test() {
        let re = Regex::new(r"\{(:\?)?\}").unwrap();
        let split = re.split("a {} b {:?} c");
        let vec: Vec<&str> = split.collect();
        println!("{:?}", vec);
    }
}
