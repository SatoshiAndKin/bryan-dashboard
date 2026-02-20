use crate::model::CellRef;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    CellRef(CellRef),
    /// TABLE_NAME::A1
    CrossTableRef(String, CellRef),
    Range(CellRef, CellRef),
    /// TABLE_NAME::A1:B5
    CrossTableRange(String, CellRef, CellRef),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    UnaryNeg(Box<Expr>),
    FuncCall(String, Vec<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}
