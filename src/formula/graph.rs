use std::collections::{HashMap, HashSet, VecDeque};

use super::ast::Expr;
use super::eval::{evaluate, EvalContext};
use super::parser::parse_formula;
use crate::model::cell::{CellRef, CellValue};
use crate::model::table::TableModel;

fn extract_local_deps(expr: &Expr) -> Vec<CellRef> {
    let mut deps = Vec::new();
    match expr {
        Expr::Number(_) => {}
        Expr::CellRef(r) => deps.push(r.clone()),
        Expr::CrossTableRef(_, _) => {} // external dep, not tracked in local graph
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
        Expr::CrossTableRange(_, _, _) => {} // external dep
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
    }
    deps
}

pub fn recalculate_table(table: &mut TableModel) {
    recalculate_table_with_siblings(table, &[]);
}

pub fn recalculate_table_with_siblings(table: &mut TableModel, siblings: &[TableModel]) {
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
                    let ctx = EvalContext::with_siblings(table, siblings);
                    let val = evaluate(expr, &ctx);
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
