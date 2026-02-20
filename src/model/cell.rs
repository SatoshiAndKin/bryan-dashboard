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
    if s.is_empty() {
        return None;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_col_index_to_label() {
        assert_eq!(col_index_to_label(0), "A");
        assert_eq!(col_index_to_label(1), "B");
        assert_eq!(col_index_to_label(25), "Z");
        assert_eq!(col_index_to_label(26), "AA");
        assert_eq!(col_index_to_label(27), "AB");
        assert_eq!(col_index_to_label(51), "AZ");
        assert_eq!(col_index_to_label(52), "BA");
        assert_eq!(col_index_to_label(701), "ZZ");
        assert_eq!(col_index_to_label(702), "AAA");
    }

    #[test]
    fn test_parse_col_label() {
        assert_eq!(parse_col_label("A"), Some(0));
        assert_eq!(parse_col_label("B"), Some(1));
        assert_eq!(parse_col_label("Z"), Some(25));
        assert_eq!(parse_col_label("AA"), Some(26));
        assert_eq!(parse_col_label("AB"), Some(27));
        assert_eq!(parse_col_label("AZ"), Some(51));
        assert_eq!(parse_col_label("BA"), Some(52));
        assert_eq!(parse_col_label("ZZ"), Some(701));
        assert_eq!(parse_col_label("AAA"), Some(702));
        assert_eq!(parse_col_label(""), None);
        assert_eq!(parse_col_label("a"), None); // lowercase
        assert_eq!(parse_col_label("1"), None);
    }

    #[test]
    fn test_col_label_roundtrip() {
        for i in 0..1000 {
            let label = col_index_to_label(i);
            assert_eq!(
                parse_col_label(&label),
                Some(i),
                "Failed roundtrip for {}",
                i
            );
        }
    }

    #[test]
    fn test_parse_cell_ref() {
        let r = parse_cell_ref("A1").unwrap();
        assert_eq!(r.col, 0);
        assert_eq!(r.row, 0);

        let r = parse_cell_ref("B3").unwrap();
        assert_eq!(r.col, 1);
        assert_eq!(r.row, 2);

        let r = parse_cell_ref("AA10").unwrap();
        assert_eq!(r.col, 26);
        assert_eq!(r.row, 9);

        assert!(parse_cell_ref("A0").is_none()); // row 0 invalid
        assert!(parse_cell_ref("1A").is_none()); // starts with digit
        assert!(parse_cell_ref("").is_none());
        assert!(parse_cell_ref("A").is_none()); // no row number
    }

    #[test]
    fn test_cell_ref_label() {
        let r = CellRef::new(0, 0);
        assert_eq!(r.label(), "A1");
        let r = CellRef::new(27, 9);
        assert_eq!(r.label(), "AB10");
    }

    #[test]
    fn test_cell_value_display() {
        assert_eq!(CellValue::Empty.to_string(), "");
        assert_eq!(CellValue::Number(42.0).to_string(), "42");
        assert_eq!(CellValue::Number(3.14).to_string(), "3.140000");
        assert_eq!(CellValue::Text("hello".into()).to_string(), "hello");
        assert_eq!(CellValue::Error("#REF!".into()).to_string(), "#REF!");
    }
}
