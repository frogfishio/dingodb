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
    /// ENR1 attach sugar: `enrich { field: expr, ... }` (usually after `|>`).
    ///
    /// Evaluates against the pipe placeholder `_` as left carrier; each row is
    /// bound as `l` while field expressions run, then attached via Map/`+`.
    Enrich(Vec<(String, Expr)>),
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

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

/// ENR1 source declaration kind ([`ENR1.md`](../../enr-core/ENR1.md) §06).
///
/// Semantic expectations only — not acquisition / transport. Hosts bind the
/// actual dataset; Index uniqueness is a claim, not an automatic collapse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceKind {
    /// Unique key semantics (no silent multi-match).
    Index,
    /// Duplicate keys allowed.
    MultiIndex,
    /// No uniqueness claim.
    Dataset,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let(String, Expr),
    Expr(Expr),
    /// `source name : Index[K, V]` — ENR1 semantic source declaration (eval no-op).
    Source {
        name: String,
        kind: SourceKind,
        /// Optional type parameters as written (`Str`, `Customer`, …); documentation only.
        type_params: Vec<String>,
    },
}
