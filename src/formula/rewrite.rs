use std::collections::HashMap;

use crate::model::cell::{col_index_to_label, CellModel};
use crate::model::table::TableModel;

/// Rewrite all cell references in a formula source string, replacing
/// occurrences of `old_ref` with `new_ref`. Operates on raw source text
/// so we don't need to re-parse and re-serialize the AST.
///
/// Only rewrites references that appear as whole tokens (preceded by
/// non-alphanumeric or start-of-string, followed by non-alphanumeric
/// or end-of-string) to avoid partial matches.
fn rewrite_ref_in_source(source: &str, old_ref: &str, new_ref: &str) -> String {
    if !source.starts_with('=') {
        return source.to_string();
    }

    let old_upper = old_ref.to_ascii_uppercase();
    let src_upper = source.to_ascii_uppercase();
    let mut result = String::with_capacity(source.len());
    let mut i = 0;
    let bytes = source.as_bytes();
    let old_bytes = old_upper.as_bytes();
    let old_len = old_bytes.len();

    while i < bytes.len() {
        if i + old_len <= bytes.len() && src_upper.as_bytes()[i..i + old_len] == old_bytes[..] {
            let before_ok = i == 0 || !(bytes[i - 1] as char).is_ascii_alphanumeric();
            let after_ok =
                i + old_len >= bytes.len() || !(bytes[i + old_len] as char).is_ascii_alphanumeric();
            if before_ok && after_ok {
                result.push_str(new_ref);
                i += old_len;
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn ref_label(col: u32, row: u32) -> String {
    format!("{}{}", col_index_to_label(col), row + 1)
}

/// After moving a cell from `from` to `to`, rewrite every formula in the table
/// that referenced `from` so it now references `to`. Operates in O(n) over
/// cells with non-empty sources.
pub fn rewrite_refs_after_move(table: &mut TableModel, from: (u32, u32), to: (u32, u32)) {
    let old_label = ref_label(from.0, from.1);
    let new_label = ref_label(to.0, to.1);

    let keys: Vec<(u32, u32)> = table.cells.keys().cloned().collect();
    for key in keys {
        // Don't rewrite the moved cell's own formula (it was already relocated)
        if key == to {
            continue;
        }
        let needs_rewrite = {
            if let Some(cell) = table.cells.get(&key) {
                cell.source.starts_with('=')
                    && cell
                        .source
                        .to_ascii_uppercase()
                        .contains(&old_label.to_ascii_uppercase())
            } else {
                false
            }
        };
        if needs_rewrite {
            let old_source = table.cells[&key].source.clone();
            let new_source = rewrite_ref_in_source(&old_source, &old_label, &new_label);
            if new_source != old_source {
                table.cells.get_mut(&key).unwrap().source = new_source;
            }
        }
    }
}

/// Rewrite refs when copying a cell with an offset. Shifts all cell references
/// in the source by the given (delta_col, delta_row).
/// Respects $ pinning: $A1 pins column, A$1 pins row, $A$1 pins both.
pub fn shift_refs_in_source(source: &str, delta_col: i32, delta_row: i32) -> String {
    if !source.starts_with('=') || (delta_col == 0 && delta_row == 0) {
        return source.to_string();
    }

    let chars: Vec<char> = source.chars().collect();
    let mut result = String::with_capacity(source.len());
    let mut i = 0;

    result.push('=');
    i += 1;

    while i < chars.len() {
        // Check for optional $ before column letters
        let pin_col = i < chars.len() && chars[i] == '$';
        let col_start = if pin_col { i + 1 } else { i };

        if col_start < chars.len() && chars[col_start].is_ascii_uppercase() {
            let ref_start = i;
            let mut col_end = col_start;
            while col_end < chars.len() && chars[col_end].is_ascii_uppercase() {
                col_end += 1;
            }
            // Check for optional $ before row digits
            let pin_row = col_end < chars.len() && chars[col_end] == '$';
            let row_start = if pin_row { col_end + 1 } else { col_end };
            let mut row_end = row_start;
            while row_end < chars.len() && chars[row_end].is_ascii_digit() {
                row_end += 1;
            }
            if col_end > col_start && row_end > row_start {
                let col_str: String = chars[col_start..col_end].iter().collect();
                let row_str: String = chars[row_start..row_end].iter().collect();
                if let Some(cell_ref) =
                    crate::model::cell::parse_cell_ref(&format!("{}{}", col_str, row_str))
                {
                    let before_ok = ref_start == 1 || !chars[ref_start - 1].is_ascii_alphanumeric();
                    let after_ok =
                        row_end >= chars.len() || !chars[row_end].is_ascii_alphanumeric();
                    if before_ok && after_ok {
                        let new_col = if pin_col {
                            cell_ref.col
                        } else {
                            (cell_ref.col as i32 + delta_col).max(0) as u32
                        };
                        let new_row = if pin_row {
                            cell_ref.row
                        } else {
                            (cell_ref.row as i32 + delta_row).max(0) as u32
                        };
                        if pin_col {
                            result.push('$');
                        }
                        result.push_str(&col_index_to_label(new_col));
                        if pin_row {
                            result.push('$');
                        }
                        result.push_str(&(new_row + 1).to_string());
                        i = row_end;
                        continue;
                    }
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

/// After deleting a row, rewrite formula sources so refs to rows above the
/// deleted row stay the same, and refs to rows below shift up by 1.
/// Refs to the deleted row itself become #REF!.
pub fn rewrite_refs_after_row_delete(cells: &mut HashMap<(u32, u32), CellModel>, deleted_row: u32) {
    let keys: Vec<(u32, u32)> = cells.keys().cloned().collect();
    for key in keys {
        let source = cells[&key].source.clone();
        if source.starts_with('=') {
            let new_source = shift_row_refs(&source, deleted_row);
            if new_source != source {
                cells.get_mut(&key).unwrap().source = new_source;
            }
        }
    }
}

/// After deleting a column, rewrite formula sources so refs to cols left of the
/// deleted col stay the same, and refs to cols right shift left by 1.
/// Refs to the deleted col itself become #REF!.
pub fn rewrite_refs_after_col_delete(cells: &mut HashMap<(u32, u32), CellModel>, deleted_col: u32) {
    let keys: Vec<(u32, u32)> = cells.keys().cloned().collect();
    for key in keys {
        let source = cells[&key].source.clone();
        if source.starts_with('=') {
            let new_source = shift_col_refs(&source, deleted_col);
            if new_source != source {
                cells.get_mut(&key).unwrap().source = new_source;
            }
        }
    }
}

fn shift_row_refs(source: &str, deleted_row: u32) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut result = String::with_capacity(source.len());
    let mut i = 0;

    result.push('=');
    i += 1;

    while i < chars.len() {
        // Skip $ for pinned col
        let pin_col = i < chars.len() && chars[i] == '$';
        let col_start = if pin_col { i + 1 } else { i };

        if col_start < chars.len() && chars[col_start].is_ascii_uppercase() {
            let ref_start = i;
            let mut col_end = col_start;
            while col_end < chars.len() && chars[col_end].is_ascii_uppercase() {
                col_end += 1;
            }
            let pin_row = col_end < chars.len() && chars[col_end] == '$';
            let row_start = if pin_row { col_end + 1 } else { col_end };
            let mut row_end = row_start;
            while row_end < chars.len() && chars[row_end].is_ascii_digit() {
                row_end += 1;
            }
            if col_end > col_start && row_end > row_start {
                let col_str: String = chars[col_start..col_end].iter().collect();
                let row_str: String = chars[row_start..row_end].iter().collect();
                if let Some(cell_ref) =
                    crate::model::cell::parse_cell_ref(&format!("{}{}", col_str, row_str))
                {
                    let before_ok = ref_start == 1 || !chars[ref_start - 1].is_ascii_alphanumeric();
                    let after_ok =
                        row_end >= chars.len() || !chars[row_end].is_ascii_alphanumeric();
                    if before_ok && after_ok {
                        if cell_ref.row == deleted_row {
                            result.push_str("#REF!");
                            i = row_end;
                            continue;
                        }
                        let new_row = if cell_ref.row > deleted_row {
                            cell_ref.row - 1
                        } else {
                            cell_ref.row
                        };
                        if pin_col {
                            result.push('$');
                        }
                        result.push_str(&col_str);
                        if pin_row {
                            result.push('$');
                        }
                        result.push_str(&(new_row + 1).to_string());
                        i = row_end;
                        continue;
                    }
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

fn shift_col_refs(source: &str, deleted_col: u32) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut result = String::with_capacity(source.len());
    let mut i = 0;

    result.push('=');
    i += 1;

    while i < chars.len() {
        let pin_col = i < chars.len() && chars[i] == '$';
        let col_start = if pin_col { i + 1 } else { i };

        if col_start < chars.len() && chars[col_start].is_ascii_uppercase() {
            let ref_start = i;
            let mut col_end = col_start;
            while col_end < chars.len() && chars[col_end].is_ascii_uppercase() {
                col_end += 1;
            }
            let pin_row = col_end < chars.len() && chars[col_end] == '$';
            let row_start = if pin_row { col_end + 1 } else { col_end };
            let mut row_end = row_start;
            while row_end < chars.len() && chars[row_end].is_ascii_digit() {
                row_end += 1;
            }
            if col_end > col_start && row_end > row_start {
                let col_str: String = chars[col_start..col_end].iter().collect();
                let row_str: String = chars[row_start..row_end].iter().collect();
                if let Some(cell_ref) =
                    crate::model::cell::parse_cell_ref(&format!("{}{}", col_str, row_str))
                {
                    let before_ok = ref_start == 1 || !chars[ref_start - 1].is_ascii_alphanumeric();
                    let after_ok =
                        row_end >= chars.len() || !chars[row_end].is_ascii_alphanumeric();
                    if before_ok && after_ok {
                        if cell_ref.col == deleted_col {
                            result.push_str("#REF!");
                            i = row_end;
                            continue;
                        }
                        let new_col = if cell_ref.col > deleted_col {
                            cell_ref.col - 1
                        } else {
                            cell_ref.col
                        };
                        if pin_col {
                            result.push('$');
                        }
                        result.push_str(&col_index_to_label(new_col));
                        if pin_row {
                            result.push('$');
                        }
                        result.push_str(&row_str);
                        i = row_end;
                        continue;
                    }
                }
            }
        }
        result.push(chars[i]);
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewrite_ref_in_source() {
        assert_eq!(rewrite_ref_in_source("=A1+B2", "A1", "C3"), "=C3+B2");
        assert_eq!(
            rewrite_ref_in_source("=SUM(A1:A5)", "A1", "B1"),
            "=SUM(B1:A5)"
        );
        // Don't touch non-formulas
        assert_eq!(rewrite_ref_in_source("hello", "A1", "B1"), "hello");
        // Don't match partial (AA1 should not match A1)
        assert_eq!(rewrite_ref_in_source("=AA1+A1", "A1", "B1"), "=AA1+B1");
    }

    #[test]
    fn test_shift_refs() {
        assert_eq!(shift_refs_in_source("=A1+B2", 1, 1), "=B2+C3");
        assert_eq!(shift_refs_in_source("=SUM(A1:B5)", 0, 2), "=SUM(A3:B7)");
        assert_eq!(shift_refs_in_source("42", 1, 1), "42");
    }

    #[test]
    fn test_shift_refs_pinned() {
        // $A1 pins column A, row shifts
        assert_eq!(shift_refs_in_source("=$A1+B2", 1, 1), "=$A2+C3");
        // A$1 pins row 1, column shifts
        assert_eq!(shift_refs_in_source("=A$1+B2", 1, 1), "=B$1+C3");
        // $A$1 pins both
        assert_eq!(shift_refs_in_source("=$A$1+B2", 1, 1), "=$A$1+C3");
        // All pinned
        assert_eq!(shift_refs_in_source("=$A$1+$B$2", 5, 5), "=$A$1+$B$2");
    }

    #[test]
    fn test_shift_row_refs_above_unaffected() {
        assert_eq!(shift_row_refs("=A1+A2", 2), "=A1+A2");
    }

    #[test]
    fn test_shift_row_refs_below_shifts() {
        // Delete row 1 (0-indexed), row 3 (=A4 in formula) -> becomes row 2 (=A3)
        assert_eq!(shift_row_refs("=A4", 1), "=A3");
    }

    #[test]
    fn test_shift_row_refs_deleted_becomes_ref_error() {
        assert_eq!(shift_row_refs("=A2", 1), "=#REF!");
    }

    #[test]
    fn test_shift_col_refs_left_unaffected() {
        assert_eq!(shift_col_refs("=A1+B1", 2), "=A1+B1");
    }

    #[test]
    fn test_shift_col_refs_right_shifts() {
        // Delete col 1 (B), D1 -> becomes C1
        assert_eq!(shift_col_refs("=D1", 1), "=C1");
    }

    #[test]
    fn test_shift_col_refs_deleted_becomes_ref_error() {
        assert_eq!(shift_col_refs("=B1", 1), "=#REF!");
    }

    #[test]
    fn test_rewrite_ref_in_source_case_insensitive() {
        assert_eq!(rewrite_ref_in_source("=a1+B2", "A1", "C3"), "=C3+B2");
    }

    #[test]
    fn test_shift_refs_zero_delta() {
        assert_eq!(shift_refs_in_source("=A1+B2", 0, 0), "=A1+B2");
    }

    #[test]
    fn test_shift_refs_negative_clamped() {
        // Shifting A1 by -5 columns should clamp to column 0 (A)
        assert_eq!(shift_refs_in_source("=A1", -5, 0), "=A1");
    }

    #[test]
    fn test_shift_row_refs_pinned_row_deleted() {
        // $A$2 has pinned row; deleting row 1 (0-indexed) should still produce #REF!
        assert_eq!(shift_row_refs("=$A$2", 1), "=#REF!");
    }

    #[test]
    fn test_shift_col_refs_pinned_col_deleted() {
        // $B1 has pinned col B (col 1); deleting col 1 should still produce #REF!
        assert_eq!(shift_col_refs("=$B1", 1), "=#REF!");
    }

    #[test]
    fn test_shift_row_refs_range_formula() {
        // =SUM(A1:A5), delete row 2 (0-indexed row 1)
        // A1 stays, A5 (row 4) becomes A4
        assert_eq!(shift_row_refs("=SUM(A1:A5)", 1), "=SUM(A1:A4)");
    }

    #[test]
    fn test_shift_col_refs_range_formula() {
        // =SUM(A1:E1), delete col B (col 1)
        // A stays, E (col 4) becomes D (col 3)
        assert_eq!(shift_col_refs("=SUM(A1:E1)", 1), "=SUM(A1:D1)");
    }

    #[test]
    fn test_shift_row_refs_multiple_refs() {
        // =A1+A3+A5, delete row 2 (0-indexed row 1)
        // A1 stays, A3 (row 2) becomes A2, A5 (row 4) becomes A4
        assert_eq!(shift_row_refs("=A1+A3+A5", 1), "=A1+A2+A4");
    }

    #[test]
    fn test_shift_col_refs_multiple_refs() {
        // =A1+C1+E1, delete col B (col 1)
        // A1 stays, C1 (col 2) becomes B1, E1 (col 4) becomes D1
        assert_eq!(shift_col_refs("=A1+C1+E1", 1), "=A1+B1+D1");
    }

    #[test]
    fn test_rewrite_ref_multiple_matches() {
        assert_eq!(rewrite_ref_in_source("=A1+A1+A1", "A1", "B2"), "=B2+B2+B2");
    }

    #[test]
    fn test_rewrite_ref_no_match() {
        assert_eq!(rewrite_ref_in_source("=B1+C2", "A1", "Z99"), "=B1+C2");
    }

    #[test]
    fn test_shift_refs_large_negative_clamp() {
        assert_eq!(shift_refs_in_source("=Z99", -100, -100), "=A1");
    }
}
