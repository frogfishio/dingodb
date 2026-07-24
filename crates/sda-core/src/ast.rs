use crate::number::ExactNum;

#[derive(Debug, Clone)]
pub enum Expr {
    Null,
    Bool(bool),
    Num(ExactNum),
    Str(String),
    Bytes(Vec<u8>),
    Ident(String),
    Placeholder,
    Seq(Vec<Expr>),
    Set(Vec<Expr>),
    Bag(Vec<Expr>),
    Map(Vec<(String, Expr)>),
    Prod(Vec<(String, Expr)>),
    BagKV(Vec<(String, Expr)>),
    Some_(Box<Expr>),
    None_,
    Ok_(Box<Expr>),
    Fail_(Box<Expr>, Box<Expr>),
    BinOp(BinOpKind, Box<Expr>, Box<Expr>),
    UnOp(UnOpKind, Box<Expr>),
    Pipe(Box<Expr>, Box<Expr>),
    Lambda(String, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Select(Box<Expr>, String, SelectMode),
    Comprehension {
        yield_expr: Option<Box<Expr>>,
        binding: String,
        collection: Box<Expr>,
        pred: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Concat,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Union,
    Inter,
    Diff,
    BUnion,
    BDiff,
    In,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOpKind {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectMode {
    Plain,
    Optional,
    Required,
}

pub struct Program {
    pub stmts: Vec<Stmt>,
}

pub enum Stmt {
    Let(String, Expr),
    Expr(Expr),
}