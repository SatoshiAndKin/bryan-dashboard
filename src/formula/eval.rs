use super::ast::{BinOp, Expr};
use crate::eth::BlockHead;
use crate::model::cell::{CellRef, CellValue};
use crate::model::table::TableModel;

/// Evaluation context: the current table, sibling tables on the same sheet,
/// and all tables across all sheets for cross-sheet references.
pub struct EvalContext<'a> {
    pub current: &'a TableModel,
    /// Tables on the same sheet (for TABLE::A1 without sheet qualifier)
    pub siblings: &'a [TableModel],
    /// All (sheet_name, table) pairs across all sheets (for SHEET::TABLE::A1)
    pub all_tables: Vec<(&'a str, &'a TableModel)>,
    /// Latest block head from Ethereum RPC (if connected)
    pub block_head: Option<&'a BlockHead>,
    /// Cache of eth_getBalance results: address -> wei balance as string
    pub balance_cache: Option<&'a std::collections::HashMap<String, String>>,
    /// Addresses that need balance lookups (collected during eval)
    pub pending_lookups: Option<&'a std::cell::RefCell<Vec<String>>>,
}

impl<'a> EvalContext<'a> {
    #[allow(dead_code)]
    pub fn single(table: &'a TableModel) -> Self {
        Self {
            current: table,
            siblings: &[],
            all_tables: Vec::new(),
            block_head: None,
            balance_cache: None,
            pending_lookups: None,
        }
    }

    pub fn with_siblings(table: &'a TableModel, siblings: &'a [TableModel]) -> Self {
        Self {
            current: table,
            siblings,
            all_tables: Vec::new(),
            block_head: None,
            balance_cache: None,
            pending_lookups: None,
        }
    }

    #[allow(dead_code)]
    pub fn full(
        table: &'a TableModel,
        siblings: &'a [TableModel],
        all_tables: Vec<(&'a str, &'a TableModel)>,
    ) -> Self {
        Self {
            current: table,
            siblings,
            all_tables,
            block_head: None,
            balance_cache: None,
            pending_lookups: None,
        }
    }

    /// Find a table by name on the same sheet (no sheet qualifier)
    fn find_table(&self, name: &str) -> Option<&'a TableModel> {
        self.siblings.iter().find(|t| t.name == name)
    }

    /// Find a table by sheet name + table name (cross-sheet)
    fn find_table_on_sheet(&self, sheet_name: &str, table_name: &str) -> Option<&'a TableModel> {
        self.all_tables
            .iter()
            .find(|(sn, t)| *sn == sheet_name && t.name == table_name)
            .map(|(_, t)| *t)
    }

    /// Resolve a cross-table ref that may or may not have a sheet qualifier
    fn resolve_cross_ref(
        &self,
        sheet: &Option<String>,
        table_name: &str,
    ) -> Option<&'a TableModel> {
        match sheet {
            Some(sn) => self.find_table_on_sheet(sn, table_name),
            None => self.find_table(table_name),
        }
    }
}

pub fn evaluate(expr: &Expr, ctx: &EvalContext) -> CellValue {
    match expr {
        Expr::Number(n) => CellValue::Number(*n),
        Expr::CellRef(r) => resolve_cell(r, ctx.current),
        Expr::CrossTableRef(sheet, table_name, r) => {
            if let Some(table) = ctx.resolve_cross_ref(sheet, table_name) {
                resolve_cell(r, table)
            } else {
                CellValue::Error("#REF!".to_string())
            }
        }
        Expr::Range(_, _) => CellValue::Error("#PARSE!".to_string()),
        Expr::CrossTableRange(_, _, _, _) => CellValue::Error("#PARSE!".to_string()),
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
        "BLOCK_NUMBER" | "BLOCK" => {
            if let Some(bh) = ctx.block_head {
                CellValue::Number(bh.number as f64)
            } else {
                CellValue::Error("#NO_RPC!".to_string())
            }
        }
        "BLOCK_HASH" => {
            if let Some(bh) = ctx.block_head {
                CellValue::Text(bh.hash.clone())
            } else {
                CellValue::Error("#NO_RPC!".to_string())
            }
        }
        "BLOCK_TIMESTAMP" => {
            if let Some(bh) = ctx.block_head {
                CellValue::Number(bh.timestamp as f64)
            } else {
                CellValue::Error("#NO_RPC!".to_string())
            }
        }
        "BLOCK_BASE_FEE" | "BASE_FEE" => {
            if let Some(bh) = ctx.block_head {
                match bh.base_fee {
                    Some(fee) => CellValue::Number(fee as f64),
                    None => CellValue::Empty,
                }
            } else {
                CellValue::Error("#NO_RPC!".to_string())
            }
        }
        "ETH_BALANCE" => {
            if args.is_empty() {
                return CellValue::Error("#ARGS! ETH_BALANCE(address)".to_string());
            }
            let addr_val = evaluate(&args[0], ctx);
            let addr = match &addr_val {
                CellValue::Text(s) => s.trim().to_lowercase(),
                CellValue::Number(n) => format!("{}", n),
                _ => return CellValue::Error("#VALUE! address required".to_string()),
            };
            if !addr.starts_with("0x") || addr.len() != 42 {
                return CellValue::Error("#VALUE! invalid address".to_string());
            }
            // Check cache
            if let Some(cache) = ctx.balance_cache {
                if let Some(hex_balance) = cache.get(&addr) {
                    // Parse hex balance to f64 (in wei)
                    let s = hex_balance.strip_prefix("0x").unwrap_or(hex_balance);
                    match u128::from_str_radix(s, 16) {
                        Ok(wei) => {
                            // Convert to ETH (divide by 1e18)
                            let eth = wei as f64 / 1e18;
                            return CellValue::Number(eth);
                        }
                        Err(_) => return CellValue::Error("#PARSE! balance".to_string()),
                    }
                }
            }
            // Not cached: request lookup
            if let Some(pending) = ctx.pending_lookups {
                let mut p = pending.borrow_mut();
                if !p.contains(&addr) {
                    p.push(addr);
                }
            }
            CellValue::Text("#LOADING...".to_string())
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
            Expr::CrossTableRange(sheet, table_name, start, end) => {
                if let Some(table) = ctx.resolve_cross_ref(sheet, table_name) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::graph::recalculate_table;
    use crate::formula::graph::recalculate_table_full;
    use crate::model::table::TableModel;

    fn make_table(rows: u32, cols: u32) -> TableModel {
        TableModel::new(1, "T".to_string(), rows, cols)
    }

    #[test]
    fn test_sum_with_empties() {
        let mut t = make_table(4, 1);
        t.set_cell_source(0, 0, "1".to_string());
        // row 1 empty
        t.set_cell_source(0, 2, "3".to_string());
        t.set_cell_source(0, 3, "=SUM(A1:A3)".to_string());
        recalculate_table(&mut t);
        assert_eq!(t.cells[&(0, 3)].computed, CellValue::Number(4.0));
    }

    #[test]
    fn test_avg_ignores_empties() {
        let mut t = make_table(4, 1);
        t.set_cell_source(0, 0, "10".to_string());
        // row 1 empty
        t.set_cell_source(0, 2, "20".to_string());
        t.set_cell_source(0, 3, "=AVG(A1:A3)".to_string());
        recalculate_table(&mut t);
        assert_eq!(t.cells[&(0, 3)].computed, CellValue::Number(15.0));
    }

    #[test]
    fn test_avg_all_empty() {
        let mut t = make_table(2, 1);
        t.set_cell_source(0, 1, "=AVG(A1:A1)".to_string());
        recalculate_table(&mut t);
        assert_eq!(
            t.cells[&(0, 1)].computed,
            CellValue::Error("#DIV/0!".to_string())
        );
    }

    #[test]
    fn test_unknown_function() {
        let mut t = make_table(1, 1);
        t.set_cell_source(0, 0, "=FOOBAR()".to_string());
        recalculate_table(&mut t);
        assert_eq!(
            t.cells[&(0, 0)].computed,
            CellValue::Error("#NAME? FOOBAR".to_string())
        );
    }

    #[test]
    fn test_ref_out_of_bounds() {
        let mut t = make_table(2, 2);
        t.set_cell_source(0, 0, "=Z99".to_string());
        recalculate_table(&mut t);
        assert_eq!(
            t.cells[&(0, 0)].computed,
            CellValue::Error("#REF!".to_string())
        );
    }

    #[test]
    fn test_text_coercion_in_arithmetic() {
        let mut t = make_table(2, 1);
        t.set_cell_source(0, 0, "5".to_string());
        t.set_cell_source(0, 1, "=A1+10".to_string());
        recalculate_table(&mut t);
        assert_eq!(t.cells[&(0, 1)].computed, CellValue::Number(15.0));
    }

    #[test]
    fn test_text_non_numeric_in_arithmetic() {
        let mut t = make_table(2, 1);
        t.set_cell_source(0, 0, "hello".to_string());
        t.set_cell_source(0, 1, "=A1+10".to_string());
        recalculate_table(&mut t);
        assert!(matches!(t.cells[&(0, 1)].computed, CellValue::Error(_)));
    }

    #[test]
    fn test_unary_neg_on_empty() {
        let mut t = make_table(2, 1);
        // A1 is empty
        t.set_cell_source(0, 1, "=-A1".to_string());
        recalculate_table(&mut t);
        assert_eq!(t.cells[&(0, 1)].computed, CellValue::Number(0.0));
    }

    #[test]
    fn test_cross_table_ref() {
        let mut t1 = TableModel::new(1, "Table 1".to_string(), 2, 2);
        t1.set_cell_source(0, 0, "42".to_string());
        recalculate_table(&mut t1);

        let mut t2 = TableModel::new(2, "Table 2".to_string(), 2, 2);
        t2.set_cell_source(0, 0, "=Table 1::A1".to_string());

        let siblings = vec![t1.clone()];
        recalculate_table_full(&mut t2, &siblings, None);
        assert_eq!(t2.cells[&(0, 0)].computed, CellValue::Number(42.0));
    }

    #[test]
    fn test_cross_table_ref_not_found() {
        let mut t = make_table(1, 1);
        t.set_cell_source(0, 0, "=NoTable::A1".to_string());
        recalculate_table(&mut t);
        assert_eq!(
            t.cells[&(0, 0)].computed,
            CellValue::Error("#REF!".to_string())
        );
    }

    #[test]
    fn test_block_timestamp_formula() {
        let bh = crate::eth::BlockHead {
            number: 100,
            hash: "0xabc".to_string(),
            timestamp: 1700000000,
            base_fee: Some(50),
        };
        let mut t = make_table(3, 1);
        t.set_cell_source(0, 0, "=BLOCK_TIMESTAMP()".to_string());
        t.set_cell_source(0, 1, "=BASE_FEE()".to_string());
        t.set_cell_source(0, 2, "=BLOCK()".to_string());
        recalculate_table_full(&mut t, &[], Some(&bh));
        assert_eq!(t.cells[&(0, 0)].computed, CellValue::Number(1700000000.0));
        assert_eq!(t.cells[&(0, 1)].computed, CellValue::Number(50.0));
        assert_eq!(t.cells[&(0, 2)].computed, CellValue::Number(100.0));
    }

    #[test]
    fn test_eth_balance_no_args() {
        let mut t = make_table(1, 1);
        t.set_cell_source(0, 0, "=ETH_BALANCE()".to_string());
        recalculate_table(&mut t);
        assert!(matches!(t.cells[&(0, 0)].computed, CellValue::Error(_)));
    }

    #[test]
    fn test_eth_balance_invalid_address() {
        let mut t = make_table(2, 1);
        t.set_cell_source(0, 0, "not_an_address".to_string());
        t.set_cell_source(0, 1, "=ETH_BALANCE(A1)".to_string());
        recalculate_table(&mut t);
        assert!(matches!(t.cells[&(0, 1)].computed, CellValue::Error(_)));
    }

    #[test]
    fn test_eth_balance_cached() {
        use crate::formula::graph::recalculate_table_with_ctx;

        let addr = "0x0000000000000000000000000000000000000001";
        let mut cache = std::collections::HashMap::new();
        // 1 ETH = 1e18 wei = 0xde0b6b3a7640000
        cache.insert(addr.to_string(), "0xde0b6b3a7640000".to_string());

        let mut t = make_table(2, 1);
        t.set_cell_source(0, 0, addr.to_string());
        t.set_cell_source(0, 1, "=ETH_BALANCE(A1)".to_string());
        recalculate_table_with_ctx(&mut t, &[], None, Some(&cache), None);
        assert_eq!(t.cells[&(0, 1)].computed, CellValue::Number(1.0));
    }

    #[test]
    fn test_eth_balance_pending_lookup() {
        use crate::formula::graph::recalculate_table_with_ctx;

        let addr = "0x0000000000000000000000000000000000000001";
        let pending = std::cell::RefCell::new(Vec::<String>::new());

        let mut t = make_table(2, 1);
        t.set_cell_source(0, 0, addr.to_string());
        t.set_cell_source(0, 1, "=ETH_BALANCE(A1)".to_string());
        recalculate_table_with_ctx(&mut t, &[], None, None, Some(&pending));
        assert_eq!(
            t.cells[&(0, 1)].computed,
            CellValue::Text("#LOADING...".to_string())
        );
        assert_eq!(pending.borrow().len(), 1);
        assert_eq!(pending.borrow()[0], addr);
    }

    #[test]
    fn test_nested_arithmetic() {
        let mut t = make_table(1, 1);
        t.set_cell_source(0, 0, "=(2+3)*(10-4)/2".to_string());
        recalculate_table(&mut t);
        assert_eq!(t.cells[&(0, 0)].computed, CellValue::Number(15.0));
    }

    #[test]
    fn test_sum_multiple_args() {
        let mut t = make_table(3, 2);
        t.set_cell_source(0, 0, "1".to_string());
        t.set_cell_source(1, 0, "2".to_string());
        t.set_cell_source(0, 1, "3".to_string());
        t.set_cell_source(0, 2, "=SUM(A1, B1, A2)".to_string());
        recalculate_table(&mut t);
        assert_eq!(t.cells[&(0, 2)].computed, CellValue::Number(6.0));
    }

    #[test]
    fn test_error_propagation_in_sum() {
        let mut t = make_table(3, 1);
        t.set_cell_source(0, 0, "1".to_string());
        t.set_cell_source(0, 1, "=1/0".to_string());
        t.set_cell_source(0, 2, "=SUM(A1:A2)".to_string());
        recalculate_table(&mut t);
        assert!(matches!(t.cells[&(0, 2)].computed, CellValue::Error(_)));
    }

    #[test]
    fn test_chained_formulas() {
        let mut t = make_table(3, 1);
        t.set_cell_source(0, 0, "10".to_string());
        t.set_cell_source(0, 1, "=A1*2".to_string());
        t.set_cell_source(0, 2, "=A2+5".to_string());
        recalculate_table(&mut t);
        assert_eq!(t.cells[&(0, 1)].computed, CellValue::Number(20.0));
        assert_eq!(t.cells[&(0, 2)].computed, CellValue::Number(25.0));
    }
}
