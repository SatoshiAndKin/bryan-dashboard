use super::ast::{BinOp, Expr};
use super::lexer::Token;
use crate::model::cell::parse_cell_ref;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        let tok = self.advance();
        if &tok == expected {
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", expected, tok))
        }
    }

    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;
        loop {
            match self.peek() {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_multiplicative()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Add, Box::new(right));
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_multiplicative()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Sub, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            match self.peek() {
                Token::Star => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Mul, Box::new(right));
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Div, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if *self.peek() == Token::Minus {
            self.advance();
            let expr = self.parse_primary()?;
            return Ok(Expr::UnaryNeg(Box::new(expr)));
        }
        self.parse_primary()
    }

    /// Parse cross-table or cross-sheet references:
    ///   TABLE::A1           -> CrossTableRef(None, table, ref)
    ///   SHEET::TABLE::A1    -> CrossTableRef(Some(sheet), table, ref)
    ///   TABLE::A1:B5        -> CrossTableRange(None, table, start, end)
    ///   SHEET::TABLE::A1:B5 -> CrossTableRange(Some(sheet), table, start, end)
    fn try_parse_cross_table_ref(&mut self) -> Option<Result<Expr, String>> {
        let save_pos = self.pos;

        // Consume first NAME before ::
        let first_name = match self.consume_name_before_double_colon() {
            Some(n) => n,
            None => {
                self.pos = save_pos;
                return None;
            }
        };

        // We consumed NAME:: — now check what follows
        // If next is another NAME::, then first_name is sheet, second is table
        // If next is a cell ref (IDENT that parses as cell ref), then first_name is table

        let checkpoint = self.pos;
        if let Some(second_name) = self.consume_name_before_double_colon() {
            // SHEET::TABLE:: — now expect cell ref
            return Some(self.parse_cell_ref_or_range(Some(first_name), second_name));
        }
        self.pos = checkpoint;

        // TABLE:: — expect cell ref
        Some(self.parse_cell_ref_or_range(None, first_name))
    }

    /// Try to consume IDENT :: (where IDENT may include spaces per lexer rules).
    /// Returns the name if successful, None otherwise (restoring position).
    fn consume_name_before_double_colon(&mut self) -> Option<String> {
        let save = self.pos;
        let mut parts = Vec::new();
        loop {
            match self.peek() {
                Token::Ident(s) => {
                    parts.push(s.clone());
                    self.advance();
                    if *self.peek() == Token::DoubleColon {
                        break;
                    }
                }
                _ => {
                    self.pos = save;
                    return None;
                }
            }
        }
        if parts.is_empty() || *self.peek() != Token::DoubleColon {
            self.pos = save;
            return None;
        }
        self.advance(); // consume ::
        Some(parts.join(" "))
    }

    /// After consuming [SHEET::]TABLE::, parse the trailing cell ref or range.
    fn parse_cell_ref_or_range(
        &mut self,
        sheet: Option<String>,
        table: String,
    ) -> Result<Expr, String> {
        match self.peek().clone() {
            Token::Ident(ref_name) => {
                self.advance();
                if let Some(cell_ref) = parse_cell_ref(&ref_name) {
                    if *self.peek() == Token::Colon {
                        self.advance();
                        let end_tok = self.advance();
                        if let Token::Ident(end_name) = end_tok {
                            if let Some(end_ref) = parse_cell_ref(&end_name) {
                                return Ok(Expr::CrossTableRange(sheet, table, cell_ref, end_ref));
                            }
                        }
                        return Err("#REF!".to_string());
                    }
                    Ok(Expr::CrossTableRef(sheet, table, cell_ref))
                } else {
                    Err("#REF!".to_string())
                }
            }
            _ => Err("#REF!".to_string()),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(Expr::Number(n))
            }
            Token::Ident(_) => {
                // Try cross-table ref first (TABLE_NAME::A1)
                if let Some(result) = self.try_parse_cross_table_ref() {
                    return result;
                }

                let Token::Ident(name) = self.advance() else {
                    unreachable!()
                };

                // Function call
                if *self.peek() == Token::LParen {
                    self.advance();
                    let args = self.parse_arg_list()?;
                    self.expect(&Token::RParen)?;
                    Ok(Expr::FuncCall(name.to_uppercase(), args))
                } else {
                    // Cell ref, possibly with range
                    if let Some(cell_ref) = parse_cell_ref(&name) {
                        if *self.peek() == Token::Colon {
                            self.advance();
                            let end_tok = self.advance();
                            if let Token::Ident(end_name) = end_tok {
                                if let Some(end_ref) = parse_cell_ref(&end_name) {
                                    return Ok(Expr::Range(cell_ref, end_ref));
                                }
                            }
                            return Err("#REF!".to_string());
                        }
                        Ok(Expr::CellRef(cell_ref))
                    } else {
                        Err("#REF!".to_string())
                    }
                }
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            tok => Err(format!("#PARSE! Unexpected token: {:?}", tok)),
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if *self.peek() == Token::RParen {
            return Ok(args);
        }
        args.push(self.parse_expr()?);
        while *self.peek() == Token::Comma {
            self.advance();
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }
}

pub fn parse_formula(source: &str) -> Result<Expr, String> {
    let input = source.strip_prefix('=').unwrap_or(source);
    let mut lexer = super::lexer::Lexer::new(input);
    let tokens = lexer.tokenize().map_err(|_| "#PARSE!".to_string())?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr()?;
    if *parser.peek() != Token::Eof {
        return Err("#PARSE!".to_string());
    }
    Ok(expr)
}
