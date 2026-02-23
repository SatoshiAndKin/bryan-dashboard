use std::collections::{HashMap, HashSet, VecDeque};

use super::ast::Expr;
use super::eval::{evaluate_at, EvalContext};
use super::parser::parse_formula;
use crate::eth::BlockHead;
use crate::model::cell::{CellRef, CellValue};
use crate::model::table::TableModel;

fn extract_local_deps(expr: &Expr) -> Vec<CellRef> {
    let mut deps = Vec::new();
    match expr {
        Expr::Number(_) | Expr::StringLit(_) => {}
        Expr::CellRef(r) => deps.push(r.clone()),
        Expr::CrossTableRef(_, _, _) => {} // external dep, not tracked in local graph
        Expr::Range(start, end) => {
            let min_col = start.col.min(end.col);
            let max_col = start.col.max(end.col);
            let min_row = start.row.min(end.row);
            let max_row = start.row.max(end.row);
            for r in min_row..=max_row {
                for c in min_col..=max_col {
                    deps.push(CellRef::new(c, r));
                }
            }
        }
        Expr::CrossTableRange(_, _, _, _) => {} // external dep
        Expr::UnaryNeg(inner) => deps.extend(extract_local_deps(inner)),
        Expr::BinOp(l, _, r) => {
            deps.extend(extract_local_deps(l));
            deps.extend(extract_local_deps(r));
        }
        Expr::FuncCall(_, args) => {
            for arg in args {
                deps.extend(extract_local_deps(arg));
            }
        }
        Expr::NamedRef(_) => {} // resolved dynamically at eval time
    }
    deps
}

pub fn recalculate_table(table: &mut TableModel) {
    recalculate_table_full(table, &[], None);
}

pub fn recalculate_table_with_siblings(table: &mut TableModel, siblings: &[TableModel]) {
    recalculate_table_full(table, siblings, None);
}

pub fn recalculate_table_full(
    table: &mut TableModel,
    siblings: &[TableModel],
    block_head: Option<&BlockHead>,
) {
    let pending = std::cell::RefCell::new(Vec::<String>::new());
    recalculate_table_with_ctx(table, siblings, block_head, None, Some(&pending));
}

pub fn recalculate_table_with_ctx(
    table: &mut TableModel,
    siblings: &[TableModel],
    block_head: Option<&BlockHead>,
    balance_cache: Option<&std::collections::HashMap<String, String>>,
    pending_lookups: Option<&std::cell::RefCell<Vec<String>>>,
) {
    let mut dependencies: HashMap<(u32, u32), Vec<(u32, u32)>> = HashMap::new();
    let mut formulas: HashMap<(u32, u32), Expr> = HashMap::new();

    let keys: Vec<(u32, u32)> = table.cells.keys().cloned().collect();

    for &(col, row) in &keys {
        let source = &table.cells[&(col, row)].source;
        if source.starts_with('=') {
            match parse_formula(source) {
                Ok(expr) => {
                    let deps = extract_local_deps(&expr);
                    let dep_keys: Vec<(u32, u32)> = deps.iter().map(|d| (d.col, d.row)).collect();
                    dependencies.insert((col, row), dep_keys);
                    formulas.insert((col, row), expr);
                }
                Err(e) => {
                    if let Some(cell) = table.cells.get_mut(&(col, row)) {
                        cell.computed = CellValue::Error(e);
                    }
                }
            }
        } else {
            let source = table.cells[&(col, row)].source.clone();
            let val = parse_literal(&source);
            if let Some(cell) = table.cells.get_mut(&(col, row)) {
                cell.computed = val;
            }
        }
    }

    let formula_keys: Vec<(u32, u32)> = formulas.keys().cloned().collect();
    match topo_sort(&formula_keys, &dependencies) {
        Ok(order) => {
            for key in order {
                if let Some(expr) = formulas.get(&key) {
                    let mut ctx = EvalContext::with_siblings(table, siblings);
                    ctx.block_head = block_head;
                    ctx.balance_cache = balance_cache;
                    ctx.pending_lookups = pending_lookups;
                    let val = evaluate_at(expr, &ctx, Some(key));
                    if let Some(cell) = table.cells.get_mut(&key) {
                        cell.computed = val;
                    }
                }
            }
        }
        Err(cycle_cells) => {
            for key in cycle_cells {
                if let Some(cell) = table.cells.get_mut(&key) {
                    cell.computed = CellValue::Error("#CYCLE!".to_string());
                }
            }
        }
    }
}

fn parse_literal(s: &str) -> CellValue {
    let s = s.trim();
    if s.is_empty() {
        CellValue::Empty
    } else if let Ok(n) = s.parse::<f64>() {
        CellValue::Number(n)
    } else {
        CellValue::Text(s.to_string())
    }
}

type CellKey = (u32, u32);

fn topo_sort(
    nodes: &[CellKey],
    deps: &HashMap<CellKey, Vec<CellKey>>,
) -> Result<Vec<CellKey>, Vec<CellKey>> {
    let node_set: HashSet<(u32, u32)> = nodes.iter().cloned().collect();
    let mut in_degree: HashMap<(u32, u32), usize> = HashMap::new();
    let mut forward: HashMap<(u32, u32), Vec<(u32, u32)>> = HashMap::new();

    for &node in nodes {
        in_degree.entry(node).or_insert(0);
        if let Some(node_deps) = deps.get(&node) {
            for dep in node_deps {
                if node_set.contains(dep) {
                    forward.entry(*dep).or_default().push(node);
                    *in_degree.entry(node).or_insert(0) += 1;
                }
            }
        }
    }

    let mut queue: VecDeque<(u32, u32)> = VecDeque::new();
    for (&node, &deg) in &in_degree {
        if deg == 0 {
            queue.push_back(node);
        }
    }

    let mut order = Vec::new();
    while let Some(node) = queue.pop_front() {
        order.push(node);
        if let Some(fwd) = forward.get(&node) {
            for &next in fwd {
                if let Some(d) = in_degree.get_mut(&next) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push_back(next);
                    }
                }
            }
        }
    }

    if order.len() == nodes.len() {
        Ok(order)
    } else {
        let cycle: Vec<(u32, u32)> = nodes
            .iter()
            .filter(|n| !order.contains(n))
            .cloned()
            .collect();
        Err(cycle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cell::CellValue;
    use crate::model::table::TableModel;

    fn make_table(rows: u32, cols: u32) -> TableModel {
        TableModel::new(1, "T".to_string(), rows, cols)
    }

    #[test]
    fn test_topo_sort_linear() {
        let nodes = vec![(0, 0), (0, 1), (0, 2)];
        let mut deps = HashMap::new();
        deps.insert((0, 1), vec![(0, 0)]);
        deps.insert((0, 2), vec![(0, 1)]);
        let order = topo_sort(&nodes, &deps).unwrap();
        let pos0 = order.iter().position(|&k| k == (0, 0)).unwrap();
        let pos1 = order.iter().position(|&k| k == (0, 1)).unwrap();
        let pos2 = order.iter().position(|&k| k == (0, 2)).unwrap();
        assert!(pos0 < pos1);
        assert!(pos1 < pos2);
    }

    #[test]
    fn test_topo_sort_cycle() {
        let nodes = vec![(0, 0), (0, 1)];
        let mut deps = HashMap::new();
        deps.insert((0, 0), vec![(0, 1)]);
        deps.insert((0, 1), vec![(0, 0)]);
        let result = topo_sort(&nodes, &deps);
        assert!(result.is_err());
        let cycle = result.unwrap_err();
        assert_eq!(cycle.len(), 2);
    }

    #[test]
    fn test_topo_sort_no_deps() {
        let nodes = vec![(0, 0), (1, 0), (2, 0)];
        let deps = HashMap::new();
        let order = topo_sort(&nodes, &deps).unwrap();
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn test_topo_sort_diamond() {
        // A -> B, A -> C, B -> D, C -> D
        let nodes = vec![(0, 0), (1, 0), (2, 0), (3, 0)];
        let mut deps = HashMap::new();
        deps.insert((1, 0), vec![(0, 0)]);
        deps.insert((2, 0), vec![(0, 0)]);
        deps.insert((3, 0), vec![(1, 0), (2, 0)]);
        let order = topo_sort(&nodes, &deps).unwrap();
        let pos_a = order.iter().position(|&k| k == (0, 0)).unwrap();
        let pos_b = order.iter().position(|&k| k == (1, 0)).unwrap();
        let pos_c = order.iter().position(|&k| k == (2, 0)).unwrap();
        let pos_d = order.iter().position(|&k| k == (3, 0)).unwrap();
        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_d);
        assert!(pos_c < pos_d);
    }

    #[test]
    fn test_parse_literal_empty() {
        assert_eq!(parse_literal(""), CellValue::Empty);
        assert_eq!(parse_literal("  "), CellValue::Empty);
    }

    #[test]
    fn test_parse_literal_number() {
        assert_eq!(parse_literal("42"), CellValue::Number(42.0));
        assert_eq!(parse_literal("2.5"), CellValue::Number(2.5));
        assert_eq!(parse_literal(" -1 "), CellValue::Number(-1.0));
    }

    #[test]
    fn test_parse_literal_text() {
        assert_eq!(parse_literal("hello"), CellValue::Text("hello".to_string()));
        assert_eq!(
            parse_literal("abc123"),
            CellValue::Text("abc123".to_string())
        );
    }

    #[test]
    fn test_recalculate_with_named_ref() {
        let mut t = TableModel::new(1, "T".to_string(), 3, 3);
        t.header_rows = 1;
        t.set_cell_source(0, 0, "X".to_string());
        t.set_cell_source(1, 0, "Y".to_string());
        t.set_cell_source(2, 0, "Sum".to_string());
        t.set_cell_source(0, 1, "10".to_string());
        t.set_cell_source(1, 1, "20".to_string());
        t.set_cell_source(2, 1, "=X+Y".to_string());
        recalculate_table(&mut t);
        assert_eq!(t.cells[&(2, 1)].computed, CellValue::Number(30.0));
    }

    #[test]
    fn test_recalculate_preserves_non_formula() {
        let mut t = make_table(2, 1);
        t.set_cell_source(0, 0, "hello".to_string());
        t.set_cell_source(0, 1, "42".to_string());
        recalculate_table(&mut t);
        assert_eq!(
            t.cells[&(0, 0)].computed,
            CellValue::Text("hello".to_string())
        );
        assert_eq!(t.cells[&(0, 1)].computed, CellValue::Number(42.0));
    }

    #[test]
    fn test_recalculate_parse_error() {
        let mut t = make_table(1, 1);
        t.set_cell_source(0, 0, "=+".to_string());
        recalculate_table(&mut t);
        assert!(matches!(t.cells[&(0, 0)].computed, CellValue::Error(_)));
    }

    #[test]
    fn test_self_referencing_formula() {
        let mut t = make_table(1, 1);
        t.set_cell_source(0, 0, "=A1".to_string());
        recalculate_table(&mut t);
        assert!(matches!(t.cells[&(0, 0)].computed, CellValue::Error(_)));
    }

    #[test]
    fn test_three_cell_cycle() {
        let mut t = make_table(3, 1);
        t.set_cell_source(0, 0, "=A3".to_string());
        t.set_cell_source(0, 1, "=A1".to_string());
        t.set_cell_source(0, 2, "=A2".to_string());
        recalculate_table(&mut t);
        assert!(matches!(t.cells[&(0, 0)].computed, CellValue::Error(_)));
        assert!(matches!(t.cells[&(0, 1)].computed, CellValue::Error(_)));
        assert!(matches!(t.cells[&(0, 2)].computed, CellValue::Error(_)));
    }

    #[test]
    fn test_external_dep_not_tracked() {
        // Cross-table refs shouldn't cause cycle detection issues
        let mut t = make_table(1, 1);
        t.set_cell_source(0, 0, "=OtherTable::A1".to_string());
        recalculate_table(&mut t);
        // Should get #REF! (no sibling), not a cycle error
        assert_eq!(
            t.cells[&(0, 0)].computed,
            CellValue::Error("#REF!".to_string())
        );
    }
}
