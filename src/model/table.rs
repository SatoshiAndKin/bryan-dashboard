use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

use super::cell::{col_index_to_label, CellModel, CellRef};
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

    pub fn col_width(&self, col: u32) -> f32 {
        self.col_widths.get(&col).copied().unwrap_or(100.0)
    }

    pub fn row_height(&self, row: u32) -> f32 {
        self.row_heights.get(&row).copied().unwrap_or(28.0)
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
        };
        self.cells
            .get(&(cell_ref.col, cell_ref.row))
            .unwrap_or(&DEFAULT)
    }
}
