use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellRef {
    pub col: u32,
    pub row: u32,
}

impl CellRef {
    pub fn new(col: u32, row: u32) -> Self {
        Self { col, row }
    }

    pub fn col_label(&self) -> String {
        col_index_to_label(self.col)
    }

    pub fn label(&self) -> String {
        format!("{}{}", self.col_label(), self.row + 1)
    }
}

impl fmt::Display for CellRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label())
    }
}

pub fn col_index_to_label(mut col: u32) -> String {
    let mut label = String::new();
    loop {
        label.insert(0, (b'A' + (col % 26) as u8) as char);
        if col < 26 {
            break;
        }
        col = col / 26 - 1;
    }
    label
}

pub fn parse_col_label(s: &str) -> Option<u32> {
    let mut col: u32 = 0;
    for (i, c) in s.chars().enumerate() {
        if !c.is_ascii_uppercase() {
            return None;
        }
        if i > 0 {
            col = (col + 1) * 26;
        }
        col += (c as u32) - ('A' as u32);
    }
    Some(col)
}

pub fn parse_cell_ref(s: &str) -> Option<CellRef> {
    let s = s.trim();
    let split = s.find(|c: char| c.is_ascii_digit())?;
    if split == 0 {
        return None;
    }
    let col_part = &s[..split];
    let row_part = &s[split..];
    let col = parse_col_label(col_part)?;
    let row: u32 = row_part.parse().ok()?;
    if row == 0 {
        return None;
    }
    Some(CellRef::new(col, row - 1))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum CellValue {
    #[default]
    Empty,
    Number(f64),
    Text(String),
    Error(String),
}

impl fmt::Display for CellValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellValue::Empty => write!(f, ""),
            CellValue::Number(n) => {
                if *n == n.floor() && n.abs() < 1e15 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{:.6}", n)
                }
            }
            CellValue::Text(s) => write!(f, "{}", s),
            CellValue::Error(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellModel {
    pub source: String,
    #[serde(skip)]
    pub computed: CellValue,
}

impl Default for CellModel {
    fn default() -> Self {
        Self {
            source: String::new(),
            computed: CellValue::Empty,
        }
    }
}
