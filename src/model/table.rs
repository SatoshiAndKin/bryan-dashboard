use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

use super::cell::{col_index_to_label, parse_cell_ref, CellFormat, CellModel, CellRef};
use crate::formula::rewrite::{
    rewrite_refs_after_col_delete, rewrite_refs_after_move, rewrite_refs_after_row_delete,
    shift_refs_in_source,
};

pub type TableId = u64;

fn default_one() -> u32 {
    1
}

fn serialize_cells<S: Serializer>(
    cells: &HashMap<(u32, u32), CellModel>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let entries: Vec<(u32, u32, &CellModel)> = cells.iter().map(|(&(c, r), m)| (c, r, m)).collect();
    entries.serialize(serializer)
}

fn deserialize_cells<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<HashMap<(u32, u32), CellModel>, D::Error> {
    let entries: Vec<(u32, u32, CellModel)> = Vec::deserialize(deserializer)?;
    Ok(entries.into_iter().map(|(c, r, m)| ((c, r), m)).collect())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableModel {
    pub id: TableId,
    pub name: String,
    pub rows: u32,
    pub cols: u32,
    #[serde(
        serialize_with = "serialize_cells",
        deserialize_with = "deserialize_cells"
    )]
    pub cells: HashMap<(u32, u32), CellModel>,
    pub col_widths: HashMap<u32, f32>,
    pub row_heights: HashMap<u32, f32>,
    /// Canvas position (x, y) for Numbers-style multi-table sheets
    #[serde(default)]
    pub canvas_x: f32,
    #[serde(default)]
    pub canvas_y: f32,
    /// Number of header rows at the top (their cell values name the columns)
    #[serde(default = "default_one")]
    pub header_rows: u32,
    /// Number of header columns on the left (their cell values name the rows)
    #[serde(default = "default_one")]
    pub header_cols: u32,
    /// Number of footer rows at the bottom (e.g. TOTAL rows)
    #[serde(default)]
    pub footer_rows: u32,
    /// Custom column names (override, from header row content)
    #[serde(default)]
    pub col_names: HashMap<u32, String>,
    /// Custom row names (override, from header col content)
    #[serde(default)]
    pub row_names: HashMap<u32, String>,
}

impl TableModel {
    pub fn new(id: TableId, name: String, rows: u32, cols: u32) -> Self {
        Self {
            id,
            name,
            rows,
            cols,
            cells: HashMap::new(),
            col_widths: HashMap::new(),
            row_heights: HashMap::new(),
            canvas_x: 0.0,
            canvas_y: 0.0,
            header_rows: 1,
            header_cols: 1,
            footer_rows: 0,
            col_names: HashMap::new(),
            row_names: HashMap::new(),
        }
    }

    pub fn is_header_row(&self, row: u32) -> bool {
        row < self.header_rows
    }

    pub fn is_header_col(&self, col: u32) -> bool {
        col < self.header_cols
    }

    pub fn is_footer_row(&self, row: u32) -> bool {
        self.footer_rows > 0 && row >= self.rows.saturating_sub(self.footer_rows)
    }

    pub fn is_header_cell(&self, col: u32, row: u32) -> bool {
        self.is_header_row(row) || self.is_header_col(col) || self.is_footer_row(row)
    }

    pub fn get_cell(&self, col: u32, row: u32) -> Option<&CellModel> {
        self.cells.get(&(col, row))
    }

    pub fn get_cell_mut(&mut self, col: u32, row: u32) -> &mut CellModel {
        self.cells.entry((col, row)).or_default()
    }

    pub fn set_cell_source(&mut self, col: u32, row: u32, source: String) {
        let cell = self.get_cell_mut(col, row);
        cell.source = source;
    }

    pub fn set_cell_format(&mut self, col: u32, row: u32, format: CellFormat) {
        let cell = self.get_cell_mut(col, row);
        cell.format = format;
    }

    pub fn get_cell_format(&self, col: u32, row: u32) -> CellFormat {
        self.cells
            .get(&(col, row))
            .map(|c| c.format.clone())
            .unwrap_or_default()
    }

    pub fn col_width(&self, col: u32) -> f32 {
        self.col_widths.get(&col).copied().unwrap_or(100.0)
    }

    pub fn row_height(&self, row: u32) -> f32 {
        self.row_heights.get(&row).copied().unwrap_or(28.0)
    }

    pub fn pixel_width(&self) -> f32 {
        let mut w: f32 = 0.0;
        for c in 0..self.cols {
            w += self.col_width(c);
        }
        w + 2.0 // border
    }

    pub fn pixel_height(&self) -> f32 {
        let header_bar = 30.0;
        let mut h: f32 = header_bar;
        for r in 0..self.rows {
            h += self.row_height(r);
        }
        h + 2.0 // border
    }

    pub fn add_row(&mut self) {
        self.rows += 1;
    }

    pub fn add_col(&mut self) {
        self.cols += 1;
    }

    /// Delete a row, shifting all rows below it up. Rewrites cell refs.
    pub fn delete_row(&mut self, row: u32) {
        if self.rows <= 1 {
            return;
        }
        // Remove all cells in this row
        let keys: Vec<(u32, u32)> = self.cells.keys().cloned().collect();
        for (c, r) in keys {
            if r == row {
                self.cells.remove(&(c, r));
            }
        }
        // Shift cells below the deleted row up
        let mut shifted: HashMap<(u32, u32), CellModel> = HashMap::new();
        let remaining: Vec<((u32, u32), CellModel)> = self.cells.drain().collect();
        for ((c, r), cell) in remaining {
            if r > row {
                shifted.insert((c, r - 1), cell);
            } else {
                shifted.insert((c, r), cell);
            }
        }
        self.cells = shifted;
        self.rows -= 1;

        // Shift row heights
        let mut new_heights: HashMap<u32, f32> = HashMap::new();
        for (&r, &h) in &self.row_heights {
            if r < row {
                new_heights.insert(r, h);
            } else if r > row {
                new_heights.insert(r - 1, h);
            }
        }
        self.row_heights = new_heights;

        // Shift row names
        let mut new_names: HashMap<u32, String> = HashMap::new();
        for (&r, name) in &self.row_names {
            if r < row {
                new_names.insert(r, name.clone());
            } else if r > row {
                new_names.insert(r - 1, name.clone());
            }
        }
        self.row_names = new_names;

        // Adjust header/footer counts
        if row < self.header_rows && self.header_rows > 0 {
            self.header_rows -= 1;
        }
        if self.footer_rows > 0 {
            // The footer boundary was at (old_rows - footer_rows).
            // After deletion: new_rows = old_rows - 1 (already decremented above).
            // If deleted row was in the footer zone (>= old_rows - footer_rows),
            // decrement footer_rows. old_rows was self.rows + 1 at this point.
            let old_rows = self.rows + 1;
            if row >= old_rows - self.footer_rows {
                self.footer_rows -= 1;
            }
        }

        // Rewrite refs: shift all row refs >= row+1 down by 1
        rewrite_refs_after_row_delete(&mut self.cells, row);
    }

    /// Delete a column, shifting all columns to the right left. Rewrites cell refs.
    pub fn delete_col(&mut self, col: u32) {
        if self.cols <= 1 {
            return;
        }
        // Remove all cells in this column
        let keys: Vec<(u32, u32)> = self.cells.keys().cloned().collect();
        for (c, r) in keys {
            if c == col {
                self.cells.remove(&(c, r));
            }
        }
        // Shift cells right of the deleted column left
        let mut shifted: HashMap<(u32, u32), CellModel> = HashMap::new();
        let remaining: Vec<((u32, u32), CellModel)> = self.cells.drain().collect();
        for ((c, r), cell) in remaining {
            if c > col {
                shifted.insert((c - 1, r), cell);
            } else {
                shifted.insert((c, r), cell);
            }
        }
        self.cells = shifted;
        self.cols -= 1;

        // Shift col widths
        let mut new_widths: HashMap<u32, f32> = HashMap::new();
        for (&c, &w) in &self.col_widths {
            if c < col {
                new_widths.insert(c, w);
            } else if c > col {
                new_widths.insert(c - 1, w);
            }
        }
        self.col_widths = new_widths;

        // Shift col names
        let mut new_names: HashMap<u32, String> = HashMap::new();
        for (&c, name) in &self.col_names {
            if c < col {
                new_names.insert(c, name.clone());
            } else if c > col {
                new_names.insert(c - 1, name.clone());
            }
        }
        self.col_names = new_names;

        // Adjust header col count
        if col < self.header_cols && self.header_cols > 0 {
            self.header_cols -= 1;
        }

        // Rewrite refs: shift all col refs >= col+1 left by 1
        rewrite_refs_after_col_delete(&mut self.cells, col);
    }

    /// Get a display name for a column. Uses the last header row's non-formula
    /// cell content, or col_names override, falling back to the column letter.
    pub fn col_display_name(&self, col: u32) -> String {
        if let Some(name) = self.col_names.get(&col) {
            if !name.is_empty() {
                return name.clone();
            }
        }
        if self.header_rows > 0 {
            let header_row = self.header_rows - 1;
            if let Some(cell) = self.cells.get(&(col, header_row)) {
                if !cell.source.is_empty() && !cell.source.starts_with('=') {
                    return cell.source.clone();
                }
            }
        }
        col_index_to_label(col)
    }

    /// Get a display name for a row. Uses the last header col's non-formula
    /// cell content, or row_names override, falling back to the row number.
    pub fn row_display_name(&self, row: u32) -> String {
        if let Some(name) = self.row_names.get(&row) {
            if !name.is_empty() {
                return name.clone();
            }
        }
        if self.header_cols > 0 {
            let header_col = self.header_cols - 1;
            if let Some(cell) = self.cells.get(&(header_col, row)) {
                if !cell.source.is_empty() && !cell.source.starts_with('=') {
                    return cell.source.clone();
                }
            }
        }
        (row + 1).to_string()
    }

    /// Get the pretty name for a column if headers provide one, else None.
    pub fn col_pretty_name(&self, col: u32) -> Option<String> {
        let name = self.col_display_name(col);
        if name != col_index_to_label(col) {
            Some(name)
        } else {
            None
        }
    }

    /// Get the pretty name for a row if headers provide one, else None.
    pub fn row_pretty_name(&self, row: u32) -> Option<String> {
        let name = self.row_display_name(row);
        if name != (row + 1).to_string() {
            Some(name)
        } else {
            None
        }
    }

    /// Replace A1-style cell references in a formula with pretty names when available.
    /// E.g. `=SUM(A1:A5)` -> `=SUM(Price:Price)` if col A has header "Price".
    pub fn prettify_formula(&self, source: &str) -> String {
        if !source.starts_with('=') {
            return source.to_string();
        }
        let chars: Vec<char> = source.chars().collect();
        let mut result = String::with_capacity(source.len());
        let mut i = 0;
        while i < chars.len() {
            if chars[i].is_ascii_uppercase() {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_uppercase() {
                    i += 1;
                }
                let col_end = i;
                if i < chars.len() && chars[i].is_ascii_digit() {
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        i += 1;
                    }
                    let ref_str: String = chars[start..i].iter().collect();
                    if let Some(cr) = parse_cell_ref(&ref_str) {
                        let col_name = self.col_pretty_name(cr.col);
                        let row_name = self.row_pretty_name(cr.row);
                        match (col_name, row_name) {
                            (Some(cn), Some(rn)) => result.push_str(&format!("{}.{}", cn, rn)),
                            (Some(cn), None) => result.push_str(&format!("{}{}", cn, cr.row + 1)),
                            (None, Some(rn)) => {
                                result.push_str(&format!("{}{}", col_index_to_label(cr.col), rn))
                            }
                            (None, None) => result.push_str(&ref_str),
                        }
                    } else {
                        result.push_str(&ref_str);
                    }
                } else {
                    let ident: String = chars[start..col_end].iter().collect();
                    result.push_str(&ident);
                }
            } else {
                result.push(chars[i]);
                i += 1;
            }
        }
        result
    }

    /// Move a cell from `from` to `to`. Rewrites all formulas that referenced
    /// `from` to point to `to`. Clears the source cell.
    pub fn move_cell(&mut self, from: (u32, u32), to: (u32, u32)) {
        if from == to {
            return;
        }
        let source_cell = self.cells.remove(&from).unwrap_or_default();
        self.cells.insert(to, source_cell);
        rewrite_refs_after_move(self, from, to);
    }

    /// Copy a cell's source to a target position, shifting any cell references
    /// in the formula by the positional delta.
    pub fn copy_cell(&mut self, from: (u32, u32), to: (u32, u32)) {
        if from == to {
            return;
        }
        let source = self
            .cells
            .get(&from)
            .map(|c| c.source.clone())
            .unwrap_or_default();
        let delta_col = to.0 as i32 - from.0 as i32;
        let delta_row = to.1 as i32 - from.1 as i32;
        let new_source = shift_refs_in_source(&source, delta_col, delta_row);
        self.set_cell_source(to.0, to.1, new_source);
    }

    pub fn cell_ref_value(&self, cell_ref: &CellRef) -> &CellModel {
        static DEFAULT: CellModel = CellModel {
            source: String::new(),
            computed: super::cell::CellValue::Empty,
            format: super::cell::CellFormat {
                number_format: super::cell::NumberFormat::Auto,
                align: super::cell::TextAlign::Auto,
                bold: false,
                italic: false,
                bg_color: None,
                fg_color: None,
            },
        };
        self.cells
            .get(&(cell_ref.col, cell_ref.row))
            .unwrap_or(&DEFAULT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::graph::recalculate_table;

    fn make_table(rows: u32, cols: u32) -> TableModel {
        TableModel::new(1, "T".to_string(), rows, cols)
    }

    #[test]
    fn test_delete_row_shifts_data() {
        let mut t = make_table(4, 2);
        t.set_cell_source(0, 0, "A".to_string());
        t.set_cell_source(0, 1, "B".to_string());
        t.set_cell_source(0, 2, "C".to_string());
        t.set_cell_source(0, 3, "D".to_string());
        t.delete_row(1); // delete row containing "B"
        assert_eq!(t.rows, 3);
        assert_eq!(t.cells[&(0, 0)].source, "A");
        assert_eq!(t.cells[&(0, 1)].source, "C");
        assert_eq!(t.cells[&(0, 2)].source, "D");
        assert!(t.cells.get(&(0, 3)).is_none());
    }

    #[test]
    fn test_delete_row_rewrites_refs() {
        let mut t = make_table(4, 2);
        t.set_cell_source(0, 0, "10".to_string());
        t.set_cell_source(0, 1, "20".to_string());
        t.set_cell_source(0, 2, "30".to_string());
        // Formula in col 1 row 3 refs row 3 (=A3 which is 0-indexed row 2)
        t.set_cell_source(1, 3, "=A3".to_string());
        t.delete_row(1); // delete 0-indexed row 1 (value "20")
                         // A3 was 0-indexed row 2, now shifted to row 1. The ref should become A2.
        assert_eq!(t.cells[&(1, 2)].source, "=A2");
    }

    #[test]
    fn test_delete_row_ref_becomes_ref_error() {
        let mut t = make_table(3, 2);
        t.set_cell_source(0, 0, "10".to_string());
        t.set_cell_source(0, 1, "20".to_string());
        t.set_cell_source(1, 2, "=A2".to_string()); // refs row 1
        t.delete_row(1); // delete the row that A2 points to
                         // The formula cell moved from (1,2) to (1,1), and ref A2 (row 1) was deleted
        assert_eq!(t.cells[&(1, 1)].source, "=#REF!");
    }

    #[test]
    fn test_delete_col_shifts_data() {
        let mut t = make_table(2, 4);
        t.set_cell_source(0, 0, "A".to_string());
        t.set_cell_source(1, 0, "B".to_string());
        t.set_cell_source(2, 0, "C".to_string());
        t.set_cell_source(3, 0, "D".to_string());
        t.delete_col(1); // delete col containing "B"
        assert_eq!(t.cols, 3);
        assert_eq!(t.cells[&(0, 0)].source, "A");
        assert_eq!(t.cells[&(1, 0)].source, "C");
        assert_eq!(t.cells[&(2, 0)].source, "D");
    }

    #[test]
    fn test_delete_col_rewrites_refs() {
        let mut t = make_table(2, 4);
        t.set_cell_source(0, 0, "10".to_string());
        t.set_cell_source(1, 0, "20".to_string());
        t.set_cell_source(2, 0, "30".to_string());
        t.set_cell_source(3, 0, "=C1".to_string()); // refs col 2
        t.delete_col(1); // delete col B
                         // C1 was col 2, now shifted to col 1 = B1. Ref should become B1.
        assert_eq!(t.cells[&(2, 0)].source, "=B1");
    }

    #[test]
    fn test_delete_row_adjusts_header_rows() {
        let mut t = make_table(5, 2);
        t.header_rows = 2;
        t.delete_row(0); // delete first header row
        assert_eq!(t.header_rows, 1);
    }

    #[test]
    fn test_delete_row_adjusts_footer_rows() {
        let mut t = make_table(5, 2);
        t.footer_rows = 1;
        t.delete_row(4); // delete last row (the footer)
        assert_eq!(t.footer_rows, 0);
    }

    #[test]
    fn test_delete_row_body_preserves_headers_footers() {
        let mut t = make_table(5, 2);
        t.header_rows = 1;
        t.footer_rows = 1;
        t.delete_row(2); // delete a body row
        assert_eq!(t.header_rows, 1);
        assert_eq!(t.footer_rows, 1);
        assert_eq!(t.rows, 4);
    }

    #[test]
    fn test_delete_col_adjusts_header_cols() {
        let mut t = make_table(2, 5);
        t.header_cols = 2;
        t.delete_col(0);
        assert_eq!(t.header_cols, 1);
    }

    #[test]
    fn test_move_cell_rewrites_refs() {
        let mut t = make_table(3, 3);
        t.set_cell_source(0, 0, "42".to_string());
        t.set_cell_source(1, 0, "=A1".to_string());
        t.move_cell((0, 0), (2, 2));
        // Formula in (1,0) should now reference C3
        assert_eq!(t.cells[&(1, 0)].source, "=C3");
    }

    #[test]
    fn test_copy_cell_shifts_refs() {
        let mut t = make_table(5, 5);
        t.set_cell_source(0, 0, "=B2+C3".to_string());
        t.copy_cell((0, 0), (1, 1));
        assert_eq!(t.cells[&(1, 1)].source, "=C3+D4");
    }

    #[test]
    fn test_formula_evaluation() {
        let mut t = make_table(3, 2);
        t.set_cell_source(0, 0, "10".to_string());
        t.set_cell_source(0, 1, "20".to_string());
        t.set_cell_source(0, 2, "=A1+A2".to_string());
        recalculate_table(&mut t);
        assert_eq!(
            t.cells[&(0, 2)].computed,
            crate::model::cell::CellValue::Number(30.0)
        );
    }

    #[test]
    fn test_formula_sum_range() {
        let mut t = make_table(4, 2);
        t.set_cell_source(0, 0, "1".to_string());
        t.set_cell_source(0, 1, "2".to_string());
        t.set_cell_source(0, 2, "3".to_string());
        t.set_cell_source(0, 3, "=SUM(A1:A3)".to_string());
        recalculate_table(&mut t);
        assert_eq!(
            t.cells[&(0, 3)].computed,
            crate::model::cell::CellValue::Number(6.0)
        );
    }

    #[test]
    fn test_formula_cycle_detection() {
        let mut t = make_table(2, 2);
        t.set_cell_source(0, 0, "=B1".to_string());
        t.set_cell_source(1, 0, "=A1".to_string());
        recalculate_table(&mut t);
        assert!(matches!(
            t.cells[&(0, 0)].computed,
            crate::model::cell::CellValue::Error(_)
        ));
    }

    #[test]
    fn test_formula_div_zero() {
        let mut t = make_table(1, 2);
        t.set_cell_source(0, 0, "=1/0".to_string());
        recalculate_table(&mut t);
        assert_eq!(
            t.cells[&(0, 0)].computed,
            crate::model::cell::CellValue::Error("#DIV/0!".to_string())
        );
    }

    #[test]
    fn test_is_header_footer_cell() {
        let mut t = make_table(10, 5);
        t.header_rows = 2;
        t.header_cols = 1;
        t.footer_rows = 1;
        assert!(t.is_header_cell(0, 0)); // header row
        assert!(t.is_header_cell(3, 1)); // header row
        assert!(t.is_header_cell(0, 5)); // header col
        assert!(t.is_header_cell(2, 9)); // footer row
        assert!(!t.is_header_cell(2, 5)); // body cell
    }

    #[test]
    fn test_col_display_name_from_header() {
        let mut t = make_table(3, 3);
        t.header_rows = 1;
        t.set_cell_source(0, 0, "Name".to_string());
        t.set_cell_source(1, 0, "Age".to_string());
        assert_eq!(t.col_display_name(0), "Name");
        assert_eq!(t.col_display_name(1), "Age");
        assert_eq!(t.col_display_name(2), "C"); // no header content, falls back
    }

    #[test]
    fn test_col_display_name_ignores_formulas() {
        let mut t = make_table(3, 2);
        t.header_rows = 1;
        t.set_cell_source(0, 0, "=1+1".to_string());
        assert_eq!(t.col_display_name(0), "A"); // formula in header, falls back
    }

    #[test]
    fn test_delete_row_then_recalculate() {
        let mut t = make_table(4, 2);
        t.set_cell_source(0, 0, "10".to_string());
        t.set_cell_source(0, 1, "20".to_string());
        t.set_cell_source(0, 2, "30".to_string());
        t.set_cell_source(1, 3, "=A3".to_string()); // refs "30"
        recalculate_table(&mut t);
        assert_eq!(
            t.cells[&(1, 3)].computed,
            crate::model::cell::CellValue::Number(30.0)
        );

        t.delete_row(1); // delete "20"
        recalculate_table(&mut t);
        // After delete: row 0="10", row 1="30", row 2 has formula.
        // Formula was =A3, row 2 (value 30) shifted to row 1, so ref becomes =A2
        assert_eq!(t.cells[&(1, 2)].source, "=A2");
        assert_eq!(
            t.cells[&(1, 2)].computed,
            crate::model::cell::CellValue::Number(30.0)
        );
    }

    #[test]
    fn test_formula_with_header_row() {
        let mut t = make_table(3, 2);
        t.header_rows = 1;
        // Row 0 is header, rows 1-2 are data
        t.set_cell_source(0, 0, "Name".to_string());
        t.set_cell_source(1, 0, "Value".to_string());
        t.set_cell_source(0, 1, "100".to_string());
        t.set_cell_source(0, 2, "=A2".to_string()); // refs row 1 (the "100" cell)
        recalculate_table(&mut t);
        assert_eq!(
            t.cells[&(0, 2)].computed,
            crate::model::cell::CellValue::Number(100.0)
        );
        // Also verify header name shows correctly
        assert_eq!(t.col_display_name(0), "Name");
        assert_eq!(t.col_display_name(1), "Value");
    }

    #[test]
    fn test_block_number_formula() {
        use crate::eth::BlockHead;
        use crate::formula::graph::recalculate_table_full;

        let mut t = make_table(2, 1);
        t.set_cell_source(0, 0, "=BLOCK_NUMBER()".to_string());
        t.set_cell_source(0, 1, "=BLOCK_HASH()".to_string());

        let bh = BlockHead {
            number: 12345,
            hash: "0xdeadbeef".to_string(),
            timestamp: 1000,
            base_fee: Some(100),
            ..Default::default()
        };
        recalculate_table_full(&mut t, &[], Some(&bh));
        assert_eq!(
            t.cells[&(0, 0)].computed,
            crate::model::cell::CellValue::Number(12345.0)
        );
        assert_eq!(
            t.cells[&(0, 1)].computed,
            crate::model::cell::CellValue::Text("0xdeadbeef".to_string())
        );
    }

    #[test]
    fn test_block_formula_no_rpc() {
        let mut t = make_table(1, 1);
        t.set_cell_source(0, 0, "=BLOCK_NUMBER()".to_string());
        recalculate_table(&mut t);
        assert!(matches!(
            t.cells[&(0, 0)].computed,
            crate::model::cell::CellValue::Error(_)
        ));
    }

    #[test]
    fn test_prettify_formula_with_headers() {
        let mut t = make_table(5, 3);
        t.header_rows = 1;
        t.header_cols = 0;
        t.set_cell_source(0, 0, "Price".to_string());
        t.set_cell_source(1, 0, "Qty".to_string());
        t.set_cell_source(2, 0, "Total".to_string());
        recalculate_table(&mut t);
        assert_eq!(t.prettify_formula("=SUM(A2:A5)"), "=SUM(Price2:Price5)");
        assert_eq!(t.prettify_formula("=A2+B3"), "=Price2+Qty3");
        assert_eq!(t.prettify_formula("=C1"), "=Total1");
        assert_eq!(t.prettify_formula("hello"), "hello");
    }

    #[test]
    fn test_prettify_formula_no_headers() {
        let mut t = make_table(3, 2);
        t.header_rows = 0;
        t.header_cols = 0;
        assert_eq!(t.prettify_formula("=A1+B2"), "=A1+B2");
    }
}
