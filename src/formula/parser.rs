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
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_concat()?;
        loop {
            match self.peek() {
                Token::Gt => {
                    self.advance();
                    let right = self.parse_concat()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Gt, Box::new(right));
                }
                Token::Lt => {
                    self.advance();
                    let right = self.parse_concat()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Lt, Box::new(right));
                }
                Token::Gte => {
                    self.advance();
                    let right = self.parse_concat()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Gte, Box::new(right));
                }
                Token::Lte => {
                    self.advance();
                    let right = self.parse_concat()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Lte, Box::new(right));
                }
                Token::Eq => {
                    self.advance();
                    let right = self.parse_concat()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Eq, Box::new(right));
                }
                Token::Neq => {
                    self.advance();
                    let right = self.parse_concat()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Neq, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_concat(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;
        loop {
            if *self.peek() == Token::Ampersand {
                self.advance();
                let right = self.parse_additive()?;
                left = Expr::BinOp(Box::new(left), BinOp::Concat, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
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
            Token::StringLit(s) => {
                self.advance();
                Ok(Expr::StringLit(s))
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
                        // Not a cell ref — treat as a named column/row reference
                        Ok(Expr::NamedRef(name))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::cell::CellRef;

    #[test]
    fn test_parse_number() {
        let e = parse_formula("=42").unwrap();
        assert_eq!(e, Expr::Number(42.0));
    }

    #[test]
    fn test_parse_cell_ref() {
        let e = parse_formula("=A1").unwrap();
        assert_eq!(e, Expr::CellRef(CellRef::new(0, 0)));
    }

    #[test]
    fn test_parse_arithmetic() {
        let e = parse_formula("=1+2*3").unwrap();
        // Should be 1 + (2*3) due to precedence
        assert!(matches!(e, Expr::BinOp(_, BinOp::Add, _)));
    }

    #[test]
    fn test_parse_range() {
        let e = parse_formula("=A1:B5").unwrap();
        assert!(matches!(e, Expr::Range(_, _)));
    }

    #[test]
    fn test_parse_function() {
        let e = parse_formula("=SUM(A1:B5)").unwrap();
        assert!(matches!(e, Expr::FuncCall(ref name, _) if name == "SUM"));
    }

    #[test]
    fn test_parse_cross_table_ref() {
        let e = parse_formula("=Table 1::A1").unwrap();
        match e {
            Expr::CrossTableRef(sheet, table, r) => {
                assert!(sheet.is_none());
                assert_eq!(table, "Table 1");
                assert_eq!(r, CellRef::new(0, 0));
            }
            _ => panic!("Expected CrossTableRef, got {:?}", e),
        }
    }

    #[test]
    fn test_parse_cross_sheet_ref() {
        let e = parse_formula("=Sheet 1::Table 1::A1").unwrap();
        match e {
            Expr::CrossTableRef(sheet, table, r) => {
                assert_eq!(sheet, Some("Sheet 1".to_string()));
                assert_eq!(table, "Table 1");
                assert_eq!(r, CellRef::new(0, 0));
            }
            _ => panic!("Expected CrossTableRef, got {:?}", e),
        }
    }

    #[test]
    fn test_parse_cross_table_range() {
        let e = parse_formula("=Table 1::A1:B5").unwrap();
        match e {
            Expr::CrossTableRange(sheet, table, start, end) => {
                assert!(sheet.is_none());
                assert_eq!(table, "Table 1");
                assert_eq!(start, CellRef::new(0, 0));
                assert_eq!(end, CellRef::new(1, 4));
            }
            _ => panic!("Expected CrossTableRange, got {:?}", e),
        }
    }

    #[test]
    fn test_parse_unary_neg() {
        let e = parse_formula("=-5").unwrap();
        assert!(matches!(e, Expr::UnaryNeg(_)));
    }

    #[test]
    fn test_parse_parens() {
        let e = parse_formula("=(1+2)*3").unwrap();
        assert!(matches!(e, Expr::BinOp(_, BinOp::Mul, _)));
    }

    #[test]
    fn test_parse_error_on_junk() {
        assert!(parse_formula("=1+").is_err());
    }

    #[test]
    fn test_parse_multi_arg_function() {
        let e = parse_formula("=SUM(A1, B2, C3)").unwrap();
        if let Expr::FuncCall(name, args) = e {
            assert_eq!(name, "SUM");
            assert_eq!(args.len(), 3);
        } else {
            panic!("Expected FuncCall");
        }
    }
}
