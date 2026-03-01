#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I32,
    Bool,
    String,
    Unit,
    Ref(Box<Type>, Mutable),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Mutable(pub bool);

#[derive(Debug, Clone, PartialEq)]
pub struct Parameters(pub Vec<Parameter>);

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub mutable: Mutable,
    pub id: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDeclaration {
    pub id: String,
    pub parameters: Parameters,
    pub ty: Option<Type>,
    pub body: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Prog(pub Vec<FnDeclaration>);

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let(Mutable, String, Option<Type>, Option<Expr>),
    Assign(Expr, Expr),
    While(Expr, Block),
    Expr(Expr),
    Fn(FnDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub semi: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Arguments(pub Vec<Expr>);

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Bang,  // Logical negation (!)
    Neg,   // Numeric negation (-)
    Deref, // Dereference (*)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(String),
    Lit(Literal),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    Par(Box<Expr>),
    Call(String, Arguments),
    IfThenElse(Box<Expr>, Block, Option<Block>),
    Block(Block),
    UnOp(UnOp, Box<Expr>),
    Ref(Box<Expr>, Mutable),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Bool(bool),
    Int(i32),
    String(String),
    Unit,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Eq,
    Lt,
    Gt,
}

// Expression priority according to https://doc.rust-lang.org/reference/expressions.html
impl BinOp {
    pub fn priority(&self) -> u8 {
        match self {
            BinOp::Mul => 5,
            BinOp::Div => 5,
            BinOp::Add => 4,
            BinOp::Sub => 4,
            BinOp::Eq => 3,
            BinOp::Lt => 3,
            BinOp::Gt => 3,
            BinOp::And => 2,
            BinOp::Or => 1,
        }
    }
}

impl Block {
    pub fn new(statements: Vec<Statement>, semi: bool) -> Self {
        Block { statements, semi }
    }
}

impl From<Expr> for Statement {
    fn from(expr: Expr) -> Self {
        Statement::Expr(expr)
    }
}

impl From<Expr> for Block {
    fn from(expr: Expr) -> Self {
        Block {
            statements: vec![expr.into()],
            semi: false,
        }
    }
}

impl From<Statement> for Block {
    fn from(stmt: Statement) -> Self {
        let semi = !matches!(stmt, Statement::Expr(_));
        Block {
            statements: vec![stmt],
            semi,
        }
    }
}
