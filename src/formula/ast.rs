use crate::model::CellRef;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    CellRef(CellRef),
    /// Optional sheet + table: Table 1::A1 or Sheet 1::Table 1::A1
    CrossTableRef(Option<String>, String, CellRef),
    Range(CellRef, CellRef),
    /// Optional sheet + table range: Table 1::A1:B5 or Sheet 1::Table 1::A1:B5
    CrossTableRange(Option<String>, String, CellRef, CellRef),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    UnaryNeg(Box<Expr>),
    FuncCall(String, Vec<Expr>),
    /// A named column reference like "Price" that resolves to a column by header name.
    /// (name, row) — the row comes from the context of the cell containing the formula.
    NamedRef(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}
