use super::ast::{BinOp, Expr};
use crate::model::cell::{CellRef, CellValue};
use crate::model::table::TableModel;

/// Evaluation context: the current table plus sibling tables for cross-table refs
pub struct EvalContext<'a> {
    pub current: &'a TableModel,
    pub siblings: &'a [TableModel],
}

impl<'a> EvalContext<'a> {
    pub fn single(table: &'a TableModel) -> Self {
        Self {
            current: table,
            siblings: &[],
        }
    }

    pub fn with_siblings(table: &'a TableModel, siblings: &'a [TableModel]) -> Self {
        Self {
            current: table,
            siblings,
        }
    }

    fn find_table(&self, name: &str) -> Option<&'a TableModel> {
        self.siblings.iter().find(|t| t.name == name)
    }
}

pub fn evaluate(expr: &Expr, ctx: &EvalContext) -> CellValue {
    match expr {
        Expr::Number(n) => CellValue::Number(*n),
        Expr::CellRef(r) => resolve_cell(r, ctx.current),
        Expr::CrossTableRef(table_name, r) => {
            if let Some(table) = ctx.find_table(table_name) {
                resolve_cell(r, table)
            } else {
                CellValue::Error("#REF!".to_string())
            }
        }
        Expr::Range(_, _) => CellValue::Error("#PARSE!".to_string()),
        Expr::CrossTableRange(_, _, _) => CellValue::Error("#PARSE!".to_string()),
        Expr::UnaryNeg(inner) => match evaluate(inner, ctx) {
            CellValue::Number(n) => CellValue::Number(-n),
            CellValue::Empty => CellValue::Number(0.0),
            other => other,
        },
        Expr::BinOp(left, op, right) => {
            let l = evaluate(left, ctx);
            let r = evaluate(right, ctx);
            eval_binop(&l, *op, &r)
        }
        Expr::FuncCall(name, args) => eval_func(name, args, ctx),
    }
}

fn resolve_cell(r: &CellRef, table: &TableModel) -> CellValue {
    if r.col >= table.cols || r.row >= table.rows {
        return CellValue::Error("#REF!".to_string());
    }
    let cell = table.cell_ref_value(r);
    cell.computed.clone()
}

fn to_number(v: &CellValue) -> Result<f64, CellValue> {
    match v {
        CellValue::Number(n) => Ok(*n),
        CellValue::Empty => Ok(0.0),
        CellValue::Text(s) => s
            .parse::<f64>()
            .map_err(|_| CellValue::Error("#VALUE!".to_string())),
        CellValue::Error(e) => Err(CellValue::Error(e.clone())),
    }
}

fn eval_binop(left: &CellValue, op: BinOp, right: &CellValue) -> CellValue {
    let l = match to_number(left) {
        Ok(n) => n,
        Err(e) => return e,
    };
    let r = match to_number(right) {
        Ok(n) => n,
        Err(e) => return e,
    };
    match op {
        BinOp::Add => CellValue::Number(l + r),
        BinOp::Sub => CellValue::Number(l - r),
        BinOp::Mul => CellValue::Number(l * r),
        BinOp::Div => {
            if r == 0.0 {
                CellValue::Error("#DIV/0!".to_string())
            } else {
                CellValue::Number(l / r)
            }
        }
    }
}

fn eval_func(name: &str, args: &[Expr], ctx: &EvalContext) -> CellValue {
    match name {
        "SUM" => {
            let values = collect_values(args, ctx);
            let mut sum = 0.0;
            for v in &values {
                match to_number(v) {
                    Ok(n) => sum += n,
                    Err(e) => return e,
                }
            }
            CellValue::Number(sum)
        }
        "AVG" | "AVERAGE" => {
            let values = collect_values(args, ctx);
            if values.is_empty() {
                return CellValue::Error("#DIV/0!".to_string());
            }
            let mut sum = 0.0;
            let mut count = 0usize;
            for v in &values {
                match v {
                    CellValue::Empty => {}
                    _ => match to_number(v) {
                        Ok(n) => {
                            sum += n;
                            count += 1;
                        }
                        Err(e) => return e,
                    },
                }
            }
            if count == 0 {
                CellValue::Error("#DIV/0!".to_string())
            } else {
                CellValue::Number(sum / count as f64)
            }
        }
        _ => CellValue::Error(format!("#NAME? {}", name)),
    }
}

fn collect_range(start: &CellRef, end: &CellRef, table: &TableModel) -> Vec<CellValue> {
    let mut values = Vec::new();
    let min_col = start.col.min(end.col);
    let max_col = start.col.max(end.col);
    let min_row = start.row.min(end.row);
    let max_row = start.row.max(end.row);
    for r in min_row..=max_row {
        for c in min_col..=max_col {
            let cell_ref = CellRef::new(c, r);
            values.push(resolve_cell(&cell_ref, table));
        }
    }
    values
}

fn collect_values(args: &[Expr], ctx: &EvalContext) -> Vec<CellValue> {
    let mut values = Vec::new();
    for arg in args {
        match arg {
            Expr::Range(start, end) => {
                values.extend(collect_range(start, end, ctx.current));
            }
            Expr::CrossTableRange(table_name, start, end) => {
                if let Some(table) = ctx.find_table(table_name) {
                    values.extend(collect_range(start, end, table));
                } else {
                    values.push(CellValue::Error("#REF!".to_string()));
                }
            }
            _ => {
                values.push(evaluate(arg, ctx));
            }
        }
    }
    values
}
