use crate::ast::*;
use crate::error::Error;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TypeEnv {
    // store (Type, Mutable) per identifier so we can enforce mutability
    scopes: Vec<HashMap<String, (Type, Mutable)>>,
}
impl Default for FunEnv {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) -> Result<(), Error> {
        if self.scopes.len() == 1 {
            Err("can't pop global scope".to_string())
        } else {
            self.scopes.pop();
            Ok(())
        }
    }

    /// Insert a variable in the current (top) scope.
    // Allow shadowing/redeclaration in the same scope by replacing previous binding
    pub fn insert(&mut self, id: String, ty: Type, mutable: Mutable) -> Result<(), Error> {
        if let Some(top) = self.scopes.last_mut() {
            top.insert(id, (ty, mutable));
        }
        Ok(())
    }

    /// Lookup only the type of an identifier.
    pub fn lookup_type(&self, id: &str) -> Result<Type, Error> {
        for scope in self.scopes.iter().rev() {
            if let Some((t, _m)) = scope.get(id) {
                return Ok(t.clone());
            }
        }
        Err(format!("var '{}' isn't declared", id))
    }

    /// Lookup the full variable info (type and mutability).
    pub fn lookup_var(&self, id: &str) -> Result<(Type, Mutable), Error> {
        for scope in self.scopes.iter().rev() {
            if let Some((t, m)) = scope.get(id) {
                return Ok((t.clone(), *m));
            }
        }
        Err(format!("var '{}' isn't declared", id))
    }
}
impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}
impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}
/// A simple function environment to store FnDeclaration by name.
#[derive(Debug)]
pub struct FunEnv {
    // allow multiple overloads per name in the same scope
    scopes: Vec<HashMap<String, Vec<FnDeclaration>>>,
}

impl FunEnv {
    pub fn new() -> Self {
        FunEnv {
            scopes: vec![HashMap::new()],
        }
    }
    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }
    pub fn pop(&mut self) -> Result<(), Error> {
        if self.scopes.len() == 1 {
            Err("cannot pop global fun scope".to_string())
        } else {
            self.scopes.pop();
            Ok(())
        }
    }

    /// Insert a function declaration in the current scope. If a function with the
    /// same name and identical parameter types already exists in the same scope,
    /// return an error. Otherwise push this overload into the vector.
    pub fn insert(&mut self, id: String, decl: FnDeclaration) -> Result<(), Error> {
        if let Some(top) = self.scopes.last_mut() {
            let entry = top.entry(id.clone()).or_default();
            // check for identical signature in this scope
            for f in entry.iter() {
                if f.parameters.0.len() == decl.parameters.0.len()
                    && f.parameters
                        .0
                        .iter()
                        .zip(decl.parameters.0.iter())
                        .all(|(a, b)| a.ty == b.ty)
                {
                    return Err(format!(
                        "function '{}' with same signature already declared in this scope",
                        id
                    ));
                }
            }
            entry.push(decl);
        }
        Ok(())
    }

    /// Lookup overloads for a name across scopes (outermost first -> last match wins as a set).
    pub fn lookup_overloads(&self, id: &str) -> Result<Vec<FnDeclaration>, Error> {
        for scope in self.scopes.iter().rev() {
            if let Some(v) = scope.get(id) {
                return Ok(v.clone());
            }
        }
        Err(format!("function '{}' not found", id))
    }
}

pub struct TypeChecker {
    env: TypeEnv, // for variables
    funs: FunEnv, // for functions
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker {
            env: TypeEnv::new(),
            funs: FunEnv::new(),
        }
    }

    pub fn unify(&self, got: Type, expected: Type) -> Result<Type, Error> {
        // to check if the type is correct
        if got == expected {
            Ok(expected)
        } else {
            Err(format!(
                "mismatch of the type, we were expected {:?} but got {:?}",
                expected, got
            ))
        }
    }

    pub fn type_of_literal(&self, lit: &Literal) -> Type {
        // to give the type of literal
        match lit {
            Literal::Int(_) => Type::I32,
            Literal::Bool(_) => Type::Bool,
            Literal::String(_) => Type::String,
            Literal::Unit => Type::Unit,
        }
    }

    pub fn check_expr(&mut self, expr: &Expr) -> Result<Type, Error> {
        match expr {
            Expr::Ident(id) => self.env.lookup_type(id),
            Expr::Lit(lit) => Ok(self.type_of_literal(lit)),
            Expr::Par(e) => self.check_expr(e),
            Expr::BinOp(op, l, r) => {
                let lt = self.check_expr(l)?;
                let rt = self.check_expr(r)?;
                match op {
                    BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                        self.unify(lt, Type::I32)?;
                        self.unify(rt, Type::I32)?;
                        Ok(Type::I32)
                    }
                    BinOp::And | BinOp::Or => {
                        self.unify(lt, Type::Bool)?;
                        self.unify(rt, Type::Bool)?;
                        Ok(Type::Bool)
                    }
                    BinOp::Eq => {
                        if lt == rt {
                            Ok(Type::Bool)
                        } else {
                            Err(format!(
                                "theses types can't be compared : {:?} and {:?}",
                                lt, rt
                            ))
                        }
                    }
                    BinOp::Lt | BinOp::Gt => {
                        self.unify(lt, Type::I32)?;
                        self.unify(rt, Type::I32)?;
                        Ok(Type::Bool)
                    }
                }
            }
            Expr::UnOp(op, e) => {
                let et = self.check_expr(e)?;
                match op {
                    UnOp::Neg => {
                        self.unify(et, Type::I32)?;
                        Ok(Type::I32)
                    }
                    UnOp::Bang => {
                        self.unify(et, Type::Bool)?;
                        Ok(Type::Bool)
                    }
                    UnOp::Deref => match et {
                        Type::Ref(inner, _m) => Ok(*inner),
                        other => Err(format!("cannot dereference non-reference type {:?}", other)),
                    },
                }
            }
            Expr::Call(name, args) => {
                if name == "println!" {
                    // accept any args and return Unit
                    for a in &args.0 {
                        let _ = self.check_expr(a)?;
                    }
                    Ok(Type::Unit)
                } else {
                    // resolve overloads by argument types
                    let overloads = self.funs.lookup_overloads(name)?;
                    let mut arg_types: Vec<Type> = Vec::new();
                    for a in &args.0 {
                        arg_types.push(self.check_expr(a)?);
                    }
                    // filter candidates matching param types exactly
                    let mut matches: Vec<FnDeclaration> = Vec::new();
                    for f in overloads.into_iter() {
                        if f.parameters.0.len() != arg_types.len() {
                            continue;
                        }
                        let mut ok = true;
                        for (p, at) in f.parameters.0.iter().zip(arg_types.iter()) {
                            if p.ty != *at {
                                ok = false;
                                break;
                            }
                        }
                        if ok {
                            matches.push(f.clone());
                        }
                    }
                    if matches.is_empty() {
                        return Err(format!(
                            "no matching overload for '{}' with args {:?}",
                            name, arg_types
                        ));
                    }
                    if matches.len() > 1 {
                        return Err(format!(
                            "ambiguous call to '{}' with arg types {:?}",
                            name, arg_types
                        ));
                    }
                    let f = &matches[0];
                    Ok(f.ty.clone().unwrap_or(Type::Unit))
                }
            }
            Expr::IfThenElse(cond, then_block, else_block) => {
                let ct = self.check_expr(cond)?;
                self.unify(ct, Type::Bool)?;
                let then_t = self.check_block(then_block)?;
                if let Some(else_blk) = else_block {
                    let else_t = self.check_block(else_blk)?;
                    if then_t == else_t {
                        Ok(then_t)
                    } else {
                        Err(format!(
                            "error, types are different : {:?} and {:?}",
                            then_t, else_t
                        ))
                    }
                } else {
                    self.unify(then_t, Type::Unit)?;
                    Ok(Type::Unit)
                }
            }
            Expr::Block(b) => self.check_block(b),
            Expr::Ref(inner, mutable) => {
                let it = self.check_expr(inner)?;
                Ok(Type::Ref(Box::new(it), *mutable))
            }
        }
    }

    pub fn check_stmt(&mut self, stmt: &Statement) -> Result<Type, Error> {
        match stmt {
            Statement::Let(mutable, id, ty_opt, init) => {
                if let Some(annot_ty) = ty_opt {
                    // with type annotation
                    if let Some(init_expr) = init {
                        let it = self.check_expr(init_expr)?;
                        self.unify(it, annot_ty.clone())?;
                    }
                    self.env.insert(id.clone(), annot_ty.clone(), *mutable)?;
                    Ok(Type::Unit)
                } else if let Some(init_expr) = init {
                    // whithout type annotation
                    let it = self.check_expr(init_expr)?;
                    self.env.insert(id.clone(), it, *mutable)?;
                    Ok(Type::Unit)
                } else {
                    // whithout initialization
                    Err(format!(
                        "can't infer type for uninitialized variable '{}'",
                        id
                    ))
                }
            }
            Statement::Assign(lhs, rhs) => {
                // simple variable assign
                if let Expr::Ident(name) = lhs {
                    let (lty, mutbl) = self.env.lookup_var(name)?;
                    // verify mutability
                    if !mutbl.0 {
                        return Err(format!("assignment to immutable variable '{}'", name));
                    }
                    let rty = self.check_expr(rhs)?;
                    self.unify(rty, lty)?;
                    Ok(Type::Unit)
                } else if let Expr::UnOp(UnOp::Deref, inner) = lhs {
                    // assign through deref (with identifier)
                    if let Expr::Ident(name) = &**inner {
                        let (vty, _vmut) = self.env.lookup_var(name)?;
                        match vty {
                            Type::Ref(boxed_ty, ref_mut) => {
                                if !ref_mut.0 {
                                    return Err(format!(
                                        "can't assign through immutable ref '{}'",
                                        name
                                    ));
                                }
                                let rty = self.check_expr(rhs)?;
                                self.unify(rty, *boxed_ty)?;
                                Ok(Type::Unit)
                            }
                            _ => Err(format!("variable '{}' isn't a reference", name)),
                        }
                    } else {
                        Err("assignment to deref of non-identifier not yet supported".to_string())
                    }
                } else {
                    Err("assignment to non-identifier not yet supported".to_string())
                }
            }
            Statement::While(cond, body) => {
                let ct = self.check_expr(cond)?;
                self.unify(ct, Type::Bool)?;
                self.check_block(body)?;
                Ok(Type::Unit)
            }
            Statement::Expr(e) => {
                let t = self.check_expr(e)?;
                Ok(t)
            }
            Statement::Fn(fdecl) => {
                // hoisted elsewhere; skip here
                Ok(Type::Unit)
            }
        }
    }

    pub fn check_block(&mut self, block: &Block) -> Result<Type, Error> {
        self.env.push();
        self.funs.push();
        // hoist functions in this block (so they are visible in all the bloc, even before declaration)
        for stmt in &block.statements {
            if let Statement::Fn(f) = stmt {
                self.funs.insert(f.id.clone(), f.clone())?;
            }
        }

        let mut last_ty = Type::Unit;
        for stmt in &block.statements {
            let ty = self.check_stmt(stmt)?;
            last_ty = ty;
        }

        self.funs.pop()?;
        self.env.pop()?;

        // if block.semi true => Unit
        if block.semi {
            Ok(Type::Unit)
        } else {
            Ok(last_ty)
        }
    }

    pub fn check_prog(&mut self, prog: &Prog) -> Result<(), Error> {
        // register all top-level functions
        for f in &prog.0 {
            self.funs.insert(f.id.clone(), f.clone())?;
        }
        // check each function once
        for f in &prog.0 {
            // new scope for function
            self.env.push();
            // bind parameters
            for p in &f.parameters.0 {
                self.env.insert(p.id.clone(), p.ty.clone(), p.mutable)?;
            }
            // check body
            let body_ty = self.check_block(&f.body)?;
            if let Some(ret_ty) = &f.ty {
                self.unify(body_ty, ret_ty.clone())?;
            }
            self.env.pop()?;
        }
        Ok(())
    }
}

// Implement Eval<Type> for Block so test helpers can call `p.eval()`
impl crate::common::Eval<Type> for Block {
    fn eval(&self) -> Result<Type, Error>
    where
        Type: Clone,
    {
        let mut tc = TypeChecker::new();
        tc.check_block(self)
    }
}

//
impl crate::common::Eval<Type> for Expr {
    fn eval(&self) -> Result<Type, Error>
    where
        Type: Clone,
    {
        let mut tc = TypeChecker::new();
        tc.check_expr(self)
    }
}

impl crate::common::Eval<Type> for Prog {
    fn eval(&self) -> Result<Type, Error>
    where
        Type: Clone,
    {
        let mut tc = TypeChecker::new();
        tc.check_prog(self)?;
        Ok(Type::Unit)
    }
}
