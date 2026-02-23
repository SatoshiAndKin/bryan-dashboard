#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Ident(String),
    StringLit(String),
    Plus,
    Minus,
    Star,
    Slash,
    LParen,
    RParen,
    Comma,
    Colon,
    DoubleColon,
    Dollar,
    /// Comparison operators for IF conditions
    Gt,
    Lt,
    Gte,
    Lte,
    Eq,
    Neq,
    /// Ampersand for string concatenation: "hello" & " " & "world"
    Ampersand,
    Eof,
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => {
                    tokens.push(Token::Eof);
                    return Ok(tokens);
                }
                Some(c) => match c {
                    '+' => {
                        self.advance();
                        tokens.push(Token::Plus);
                    }
                    '-' => {
                        self.advance();
                        tokens.push(Token::Minus);
                    }
                    '*' => {
                        self.advance();
                        tokens.push(Token::Star);
                    }
                    '/' => {
                        self.advance();
                        tokens.push(Token::Slash);
                    }
                    '(' => {
                        self.advance();
                        tokens.push(Token::LParen);
                    }
                    ')' => {
                        self.advance();
                        tokens.push(Token::RParen);
                    }
                    ',' => {
                        self.advance();
                        tokens.push(Token::Comma);
                    }
                    ':' => {
                        self.advance();
                        if self.peek() == Some(':') {
                            self.advance();
                            tokens.push(Token::DoubleColon);
                        } else {
                            tokens.push(Token::Colon);
                        }
                    }
                    '$' => {
                        self.advance();
                        tokens.push(Token::Dollar);
                    }
                    '"' => {
                        tokens.push(self.read_string()?);
                    }
                    '>' => {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            tokens.push(Token::Gte);
                        } else {
                            tokens.push(Token::Gt);
                        }
                    }
                    '<' => {
                        self.advance();
                        if self.peek() == Some('=') {
                            self.advance();
                            tokens.push(Token::Lte);
                        } else if self.peek() == Some('>') {
                            self.advance();
                            tokens.push(Token::Neq);
                        } else {
                            tokens.push(Token::Lt);
                        }
                    }
                    '=' => {
                        self.advance();
                        tokens.push(Token::Eq);
                    }
                    '&' => {
                        self.advance();
                        tokens.push(Token::Ampersand);
                    }
                    _ if c.is_ascii_digit() || c == '.' => {
                        tokens.push(self.read_number()?);
                    }
                    _ if c.is_ascii_alphabetic() || c == '_' => {
                        tokens.push(self.read_ident());
                    }
                    _ => return Err(format!("Unexpected character: {}", c)),
                },
            }
        }
    }

    fn read_string(&mut self) -> Result<Token, String> {
        self.advance(); // consume opening "
        let mut s = String::new();
        loop {
            match self.advance() {
                Some('"') => return Ok(Token::StringLit(s)),
                Some(c) => s.push(c),
                None => return Err("Unterminated string literal".to_string()),
            }
        }
    }

    fn read_number(&mut self) -> Result<Token, String> {
        let mut s = String::new();
        let mut has_dot = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
            } else if c == '.' && !has_dot {
                has_dot = true;
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        s.parse::<f64>()
            .map(Token::Number)
            .map_err(|e| format!("Invalid number: {}", e))
    }

    fn read_ident(&mut self) -> Token {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else if c == ' ' {
                // Look ahead: if after spaces there's an ident char followed eventually by ::,
                // include the space in the ident (for table names like "Table 1").
                // Simpler heuristic: if next non-space is alphanumeric and eventually :: follows,
                // include. But that's complex. Instead: include spaces if followed by alnum
                // and later a :: appears.
                // Simple approach: include space + alnum if we can see :: ahead
                if self.has_double_colon_ahead() {
                    s.push(c);
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Token::Ident(s)
    }

    fn has_double_colon_ahead(&self) -> bool {
        let mut i = self.pos;
        // Skip spaces and alnum
        while i < self.chars.len() {
            let c = self.chars[i];
            if c.is_ascii_alphanumeric() || c == '_' || c == ' ' {
                i += 1;
            } else {
                break;
            }
        }
        // Check for ::
        i + 1 < self.chars.len() && self.chars[i] == ':' && self.chars[i + 1] == ':'
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input);
        lexer.tokenize().unwrap()
    }

    #[test]
    fn test_number() {
        let tokens = tokenize("42");
        assert_eq!(tokens, vec![Token::Number(42.0), Token::Eof]);
    }

    #[test]
    fn test_decimal() {
        let tokens = tokenize("2.5");
        assert_eq!(tokens, vec![Token::Number(2.5), Token::Eof]);
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize("+-*/");
        assert_eq!(
            tokens,
            vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_parens_and_comma() {
        let tokens = tokenize("(A1, B2)");
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::Ident("A1".to_string()),
                Token::Comma,
                Token::Ident("B2".to_string()),
                Token::RParen,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_colon_vs_double_colon() {
        let tokens = tokenize("A1:B2");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("A1".to_string()),
                Token::Colon,
                Token::Ident("B2".to_string()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_double_colon() {
        let tokens = tokenize("Table 1::A1");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("Table 1".to_string()),
                Token::DoubleColon,
                Token::Ident("A1".to_string()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_dollar_sign() {
        let tokens = tokenize("$A$1");
        assert_eq!(
            tokens,
            vec![
                Token::Dollar,
                Token::Ident("A".to_string()),
                Token::Dollar,
                Token::Number(1.0),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_whitespace_skipping() {
        let tokens = tokenize("  1  +  2  ");
        assert_eq!(
            tokens,
            vec![
                Token::Number(1.0),
                Token::Plus,
                Token::Number(2.0),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_function_call() {
        let tokens = tokenize("SUM(A1:A5)");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("SUM".to_string()),
                Token::LParen,
                Token::Ident("A1".to_string()),
                Token::Colon,
                Token::Ident("A5".to_string()),
                Token::RParen,
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_cross_sheet_ref() {
        let tokens = tokenize("Sheet 1::Table 1::A1");
        assert_eq!(
            tokens,
            vec![
                Token::Ident("Sheet 1".to_string()),
                Token::DoubleColon,
                Token::Ident("Table 1".to_string()),
                Token::DoubleColon,
                Token::Ident("A1".to_string()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_unexpected_char_error() {
        let mut lexer = Lexer::new("@");
        assert!(lexer.tokenize().is_err());
    }

    #[test]
    fn test_string_literal() {
        let tokens = tokenize(r#""hello world""#);
        assert_eq!(
            tokens,
            vec![Token::StringLit("hello world".to_string()), Token::Eof]
        );
    }

    #[test]
    fn test_comparison_operators() {
        let tokens = tokenize("5>3");
        assert_eq!(
            tokens,
            vec![
                Token::Number(5.0),
                Token::Gt,
                Token::Number(3.0),
                Token::Eof
            ]
        );
        let tokens = tokenize("5>=3");
        assert_eq!(
            tokens,
            vec![
                Token::Number(5.0),
                Token::Gte,
                Token::Number(3.0),
                Token::Eof
            ]
        );
        let tokens = tokenize("5<>3");
        assert_eq!(
            tokens,
            vec![
                Token::Number(5.0),
                Token::Neq,
                Token::Number(3.0),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_ampersand() {
        let tokens = tokenize(r#""a" & "b""#);
        assert_eq!(
            tokens,
            vec![
                Token::StringLit("a".to_string()),
                Token::Ampersand,
                Token::StringLit("b".to_string()),
                Token::Eof
            ]
        );
    }

    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new(r#""unterminated"#);
        assert!(lexer.tokenize().is_err());
    }
}
