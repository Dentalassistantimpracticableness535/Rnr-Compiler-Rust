use crate::error::Error;
use crate::vm::Val;
use crate::ast::FnDeclaration;
use std::collections::HashMap;

/// stack of scopes with  HashMap<String, Val> for each scope
/// when we enter bloc { ... } → push_scope()
/// we leave bloc → pop_scope()
#[derive(Debug, Clone)]
pub struct Env {
    /// stack of scopes: the last element is the current scope (the most local)
    scopes: Vec<HashMap<String, Val>>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            scopes: vec![HashMap::new()],
        }
    }
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    pub fn pop_scope(&mut self) -> Result<(), Error> {
        // error if we try to pop the global scope
        if self.scopes.len() <= 1 {
            return Err("cannot pop global scope".to_string());
        }
        self.scopes.pop();
        Ok(())
    }

    pub fn insert(&mut self, name: String, value: Val) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    /// Insert function declaration into scope (but we keep overloads).
    /// If a funct with a same name already exists in the same scope and is a, we 
    /// append this declaration to the overload set.
    pub fn insert_overload(&mut self, name: String, decl: FnDeclaration) {
        if let Some(scope) = self.scopes.last_mut() {
            use crate::vm::Val;
            if let Some(existing) = scope.get_mut(&name) {
                match existing {
                    Val::Fun(prev) => {
                        let mut v = Vec::new();
                        v.push(prev.clone());
                        v.push(decl);
                        *existing = Val::Overloads(v);
                    }
                    Val::Overloads(vec) => {
                        vec.push(decl);
                    }
                    _ => {scope.insert(name, Val::Fun(decl));}
                }
            } else {
                scope.insert(name, Val::Fun(decl));
            }
        }
    }

    /// search variable (from the most local to the most global)
    pub fn lookup(&self, name: &str) -> Result<Val, Error> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Ok(val.clone());
            }
        }
        Err(format!("variable '{}' not found", name))
    }

    /// update existing variable
    pub fn update(&mut self, name: &str, value: Val) -> Result<(), Error> {
        // from the most local to the most global we search the first occurrence
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        Err(format!("variable '{}' not found for assignment", name))
    }
}

impl Default for Env {
    fn default() -> Self {
        Self::new()
    }
}

// Tests built with AI
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Literal;

    #[test]
    fn test_env_basic() {
        let mut env = Env::new();
        env.insert("x".to_string(), Val::Lit(Literal::Int(42)));

        let val = env.lookup("x").unwrap();
        assert_eq!(val, Val::Lit(Literal::Int(42)));
    }

    #[test]
    fn test_env_shadowing() {
        let mut env = Env::new();
        env.insert("x".to_string(), Val::Lit(Literal::Int(1)));

        env.push_scope();
        env.insert("x".to_string(), Val::Lit(Literal::Int(2)));

        let val = env.lookup("x").unwrap();
        assert_eq!(val, Val::Lit(Literal::Int(2)));

        env.pop_scope().unwrap();

        let val = env.lookup("x").unwrap();
        assert_eq!(val, Val::Lit(Literal::Int(1)));
    }

    #[test]
    fn test_env_update() {
        let mut env = Env::new();
        env.insert("x".to_string(), Val::Lit(Literal::Int(1)));

        env.update("x", Val::Lit(Literal::Int(2))).unwrap();

        let val = env.lookup("x").unwrap();
        assert_eq!(val, Val::Lit(Literal::Int(2)));
    }

    #[test]
    fn test_env_update_in_outer_scope() {
        let mut env = Env::new();
        env.insert("x".to_string(), Val::Lit(Literal::Int(1)));

        env.push_scope();
        env.update("x", Val::Lit(Literal::Int(2))).unwrap();

        let val = env.lookup("x").unwrap();
        assert_eq!(val, Val::Lit(Literal::Int(2)));

        env.pop_scope().unwrap();

        let val = env.lookup("x").unwrap();
        assert_eq!(val, Val::Lit(Literal::Int(2)));
    }

    #[test]
    fn test_env_not_found() {
        let env = Env::new();
        let result = env.lookup("x");
        assert!(result.is_err());
    }

    #[test]
    fn test_env_update_not_found() {
        let mut env = Env::new();
        let result = env.update("x", Val::Lit(Literal::Int(1)));
        assert!(result.is_err());
    }

    #[test]
    fn test_env_multiple_scopes() {
        let mut env = Env::new();
        env.insert("a".to_string(), Val::Lit(Literal::Int(1)));

        env.push_scope();
        env.insert("b".to_string(), Val::Lit(Literal::Int(2)));

        env.push_scope();
        env.insert("c".to_string(), Val::Lit(Literal::Int(3)));

        assert_eq!(env.lookup("a").unwrap(), Val::Lit(Literal::Int(1)));
        assert_eq!(env.lookup("b").unwrap(), Val::Lit(Literal::Int(2)));
        assert_eq!(env.lookup("c").unwrap(), Val::Lit(Literal::Int(3)));

        env.pop_scope().unwrap();

        assert!(env.lookup("c").is_err());
        assert_eq!(env.lookup("a").unwrap(), Val::Lit(Literal::Int(1)));
        assert_eq!(env.lookup("b").unwrap(), Val::Lit(Literal::Int(2)));

        env.pop_scope().unwrap();

        assert!(env.lookup("b").is_err());
        assert_eq!(env.lookup("a").unwrap(), Val::Lit(Literal::Int(1)));
    }

    #[test]
    fn test_insert_overload_preserves_overloads() {
        use crate::parse::parse;
        use crate::ast::Prog;

        let mut env = Env::new();
        // two functions with same name but different parameter types
        let prog: Prog = parse("fn f(x: i32) -> i32 { x } fn f(x: bool) -> i32 { 0 }");
        for f in prog.0.iter() {
            env.insert_overload(f.id.clone(), f.clone());
        }

        let val = env.lookup("f").unwrap();
        match val {
            crate::vm::Val::Overloads(vec) => assert_eq!(vec.len(), 2),
            _ => panic!("expected overloads for 'f'"),
        }
    }
}
