#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Ident(String),
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
