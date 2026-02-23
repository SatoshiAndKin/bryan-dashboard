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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum NumberFormat {
    #[default]
    Auto,
    Currency,
    Percent,
    Fixed(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum TextAlign {
    #[default]
    Auto,
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CellFormat {
    #[serde(default)]
    pub number_format: NumberFormat,
    #[serde(default)]
    pub align: TextAlign,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bg_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fg_color: Option<String>,
}

impl CellFormat {
    pub fn format_value(&self, value: &CellValue) -> String {
        match value {
            CellValue::Number(n) => match self.number_format {
                NumberFormat::Auto => {
                    if *n == n.floor() && n.abs() < 1e15 {
                        format!("{}", *n as i64)
                    } else {
                        format!("{:.6}", n)
                    }
                }
                NumberFormat::Currency => {
                    if *n < 0.0 {
                        format!("-${:.2}", n.abs())
                    } else {
                        format!("${:.2}", n)
                    }
                }
                NumberFormat::Percent => format!("{:.1}%", n * 100.0),
                NumberFormat::Fixed(dp) => format!("{:.prec$}", n, prec = dp as usize),
            },
            other => other.to_string(),
        }
    }

    pub fn css_style(&self) -> String {
        let mut style = String::new();
        match self.align {
            TextAlign::Left => style.push_str("text-align:left;"),
            TextAlign::Center => style.push_str("text-align:center;"),
            TextAlign::Right => style.push_str("text-align:right;"),
            TextAlign::Auto => {}
        }
        if self.bold {
            style.push_str("font-weight:700;");
        }
        if self.italic {
            style.push_str("font-style:italic;");
        }
        if let Some(ref bg) = self.bg_color {
            style.push_str(&format!("background-color:{};", bg));
        }
        if let Some(ref fg) = self.fg_color {
            style.push_str(&format!("color:{};", fg));
        }
        style
    }

    pub fn is_default(&self) -> bool {
        self.number_format == NumberFormat::Auto
            && self.align == TextAlign::Auto
            && !self.bold
            && !self.italic
            && self.bg_color.is_none()
            && self.fg_color.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CellModel {
    pub source: String,
    #[serde(skip)]
    pub computed: CellValue,
    #[serde(default, skip_serializing_if = "CellFormat::is_default")]
    pub format: CellFormat,
}

impl Default for CellModel {
    fn default() -> Self {
        Self {
            source: String::new(),
            computed: CellValue::Empty,
            format: CellFormat::default(),
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
    fn test_cell_format_currency() {
        let fmt = CellFormat {
            number_format: NumberFormat::Currency,
            ..Default::default()
        };
        assert_eq!(fmt.format_value(&CellValue::Number(42.5)), "$42.50");
        assert_eq!(fmt.format_value(&CellValue::Number(-10.0)), "-$10.00");
        assert_eq!(fmt.format_value(&CellValue::Text("hi".into())), "hi");
    }

    #[test]
    fn test_cell_format_percent() {
        let fmt = CellFormat {
            number_format: NumberFormat::Percent,
            ..Default::default()
        };
        assert_eq!(fmt.format_value(&CellValue::Number(0.5)), "50.0%");
        assert_eq!(fmt.format_value(&CellValue::Number(1.0)), "100.0%");
    }

    #[test]
    fn test_cell_format_fixed() {
        let fmt = CellFormat {
            number_format: NumberFormat::Fixed(2),
            ..Default::default()
        };
        assert_eq!(fmt.format_value(&CellValue::Number(1.23456)), "1.23");
        assert_eq!(fmt.format_value(&CellValue::Number(42.0)), "42.00");
    }

    #[test]
    fn test_cell_format_css_style() {
        let fmt = CellFormat {
            align: TextAlign::Right,
            bold: true,
            italic: true,
            bg_color: Some("#ff0000".to_string()),
            fg_color: Some("#00ff00".to_string()),
            ..Default::default()
        };
        let style = fmt.css_style();
        assert!(style.contains("text-align:right"));
        assert!(style.contains("font-weight:700"));
        assert!(style.contains("font-style:italic"));
        assert!(style.contains("background-color:#ff0000"));
        assert!(style.contains("color:#00ff00"));
    }

    #[test]
    fn test_cell_format_is_default() {
        assert!(CellFormat::default().is_default());
        let fmt = CellFormat {
            bold: true,
            ..Default::default()
        };
        assert!(!fmt.is_default());
    }

    #[test]
    fn test_cell_value_display() {
        assert_eq!(CellValue::Empty.to_string(), "");
        assert_eq!(CellValue::Number(42.0).to_string(), "42");
        assert_eq!(CellValue::Number(1.23).to_string(), "1.230000");
        assert_eq!(CellValue::Text("hello".into()).to_string(), "hello");
        assert_eq!(CellValue::Error("#REF!".into()).to_string(), "#REF!");
    }
}
