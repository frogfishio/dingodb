use crate::ast::*;
use crate::lexer::{Token, TokenKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token {0:?} at position {1}")]
    Unexpected(TokenKind, usize),
    #[error("Expected {0} but got {1:?} at position {2}")]
    Expected(String, TokenKind, usize),
    #[error("t_sda_selector_not_static: selector not static")]
    SelectorNotStatic,
    #[error("t_sda_duplicate_label_in_selector: duplicate label")]
    DuplicateLabelInSelector,
    #[error("t_sda_reserved_placeholder: reserved placeholder")]
    ReservedPlaceholder,
    #[error("t_sda_invalid_map_key: invalid map key")]
    InvalidMapKey,
    #[error("t_sda_invalid_bagkv_key: invalid bagkv key")]
    InvalidBagkvKey,
    #[error("Invalid bytes literal '{literal}' at position {pos}: {reason}")]
    InvalidBytesLiteral { literal: String, pos: usize, reason: String },
    #[error("Unexpected end of input")]
    UnexpectedEof,
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn peek_pos(&self) -> usize {
        self.tokens[self.pos].pos
    }

    fn peek_next(&self) -> &TokenKind {
        if self.pos + 1 < self.tokens.len() {
            &self.tokens[self.pos + 1].kind
        } else {
            &TokenKind::Eof
        }
    }

    fn advance(&mut self) -> &Token {
        let token = &self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        let token = self.peek().clone();
        let pos = self.peek_pos();
        if token == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::Expected(format!("{expected:?}"), token, pos))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        let token = self.peek().clone();
        let pos = self.peek_pos();
        if let TokenKind::Ident(name) = token {
            self.advance();
            Ok(name)
        } else {
            Err(ParseError::Expected("identifier".to_string(), token, pos))
        }
    }

    fn expected_generator_expr_error(&self) -> ParseError {
        ParseError::Expected(
            "generator expression `name in collection`".to_string(),
            self.peek().clone(),
            self.peek_pos(),
        )
    }

    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut stmts = Vec::new();
        while *self.peek() != TokenKind::Eof {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Program { stmts })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        if *self.peek() == TokenKind::Let {
            self.advance();
            if *self.peek() == TokenKind::Placeholder {
                return Err(ParseError::ReservedPlaceholder);
            }
            let name = self.expect_ident()?;
            self.expect(TokenKind::Eq)?;
            let expr = self.parse_expr()?;
            self.expect(TokenKind::Semi)?;
            Ok(Stmt::Let(name, expr))
        } else {
            let expr = self.parse_expr()?;
            self.expect(TokenKind::Semi)?;
            Ok(Stmt::Expr(expr))
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_pipe()
    }

    fn parse_pipe(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_or()?;
        while *self.peek() == TokenKind::Pipe {
            self.advance();
            let rhs = self.parse_or()?;
            lhs = Expr::Pipe(Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_and()?;
        while *self.peek() == TokenKind::Or {
            self.advance();
            let rhs = self.parse_and()?;
            lhs = Expr::BinOp(BinOpKind::Or, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_not()?;
        while *self.peek() == TokenKind::And {
            self.advance();
            let rhs = self.parse_not()?;
            lhs = Expr::BinOp(BinOpKind::And, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_not(&mut self) -> Result<Expr, ParseError> {
        if *self.peek() == TokenKind::Not {
            self.advance();
            let expr = self.parse_not()?;
            Ok(Expr::UnOp(UnOpKind::Not, Box::new(expr)))
        } else {
            self.parse_cmp()
        }
    }

    fn parse_cmp(&mut self) -> Result<Expr, ParseError> {
        let lhs = self.parse_setish()?;
        let op = match self.peek() {
            TokenKind::Eq => Some(BinOpKind::Eq),
            TokenKind::Neq => Some(BinOpKind::Neq),
            TokenKind::Lt => Some(BinOpKind::Lt),
            TokenKind::Le => Some(BinOpKind::Le),
            TokenKind::Gt => Some(BinOpKind::Gt),
            TokenKind::Ge => Some(BinOpKind::Ge),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let rhs = self.parse_setish()?;
            Ok(Expr::BinOp(op, Box::new(lhs), Box::new(rhs)))
        } else {
            Ok(lhs)
        }
    }

    fn parse_setish(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_add()?;
        loop {
            let op = match self.peek() {
                TokenKind::Union => Some(BinOpKind::Union),
                TokenKind::Inter => Some(BinOpKind::Inter),
                TokenKind::Diff => Some(BinOpKind::Diff),
                TokenKind::BUnion => Some(BinOpKind::BUnion),
                TokenKind::BDiff => Some(BinOpKind::BDiff),
                TokenKind::In => Some(BinOpKind::In),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let rhs = self.parse_add()?;
                lhs = Expr::BinOp(op, Box::new(lhs), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_add(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                TokenKind::Plus => Some(BinOpKind::Add),
                TokenKind::Minus => Some(BinOpKind::Sub),
                TokenKind::Concat => Some(BinOpKind::Concat),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let rhs = self.parse_mul()?;
                lhs = Expr::BinOp(op, Box::new(lhs), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_mul(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                TokenKind::Star => Some(BinOpKind::Mul),
                TokenKind::Slash => Some(BinOpKind::Div),
                _ => None,
            };
            if let Some(op) = op {
                self.advance();
                let rhs = self.parse_unary()?;
                lhs = Expr::BinOp(op, Box::new(lhs), Box::new(rhs));
            } else {
                break;
            }
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if *self.peek() == TokenKind::Minus {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expr::UnOp(UnOpKind::Neg, Box::new(expr)))
        } else {
            self.parse_postfix()
        }
    }

    fn is_selector_access(&self) -> bool {
        if self.pos + 2 < self.tokens.len() {
            let next = &self.tokens[self.pos + 1].kind;
            let after = &self.tokens[self.pos + 2].kind;
            matches!(next, TokenKind::Ident(_) | TokenKind::Str(_)) && *after == TokenKind::Gt
        } else {
            false
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                TokenKind::Lt if self.is_selector_access() => {
                    self.advance();
                    let selector = match self.peek().clone() {
                        TokenKind::Ident(name) => {
                            self.advance();
                            name
                        }
                        TokenKind::Str(s) => {
                            self.advance();
                            s
                        }
                        token => {
                            let pos = self.peek_pos();
                            return Err(ParseError::Expected("selector".to_string(), token, pos));
                        }
                    };
                    self.expect(TokenKind::Gt)?;
                    let mode = match self.peek() {
                        TokenKind::QMark => {
                            self.advance();
                            SelectMode::Optional
                        }
                        TokenKind::Bang => {
                            self.advance();
                            SelectMode::Required
                        }
                        _ => SelectMode::Plain,
                    };
                    expr = Expr::Select(Box::new(expr), selector, mode);
                }
                TokenKind::SelL => {
                    self.advance();
                    let selector = match self.peek().clone() {
                        TokenKind::Ident(name) => {
                            self.advance();
                            name
                        }
                        TokenKind::Str(s) => {
                            self.advance();
                            s
                        }
                        token => {
                            let pos = self.peek_pos();
                            return Err(ParseError::Expected("selector".to_string(), token, pos));
                        }
                    };
                    self.expect(TokenKind::SelR)?;
                    let mode = match self.peek() {
                        TokenKind::QMark => {
                            self.advance();
                            SelectMode::Optional
                        }
                        TokenKind::Bang => {
                            self.advance();
                            SelectMode::Required
                        }
                        _ => SelectMode::Plain,
                    };
                    expr = Expr::Select(Box::new(expr), selector, mode);
                }
                TokenKind::LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    if *self.peek() != TokenKind::RParen {
                        args.push(self.parse_expr()?);
                        while *self.peek() == TokenKind::Comma {
                            self.advance();
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    expr = Expr::Call(Box::new(expr), args);
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().clone() {
            TokenKind::Null => {
                self.advance();
                Ok(Expr::Null)
            }
            TokenKind::True => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            TokenKind::Num(n) => {
                self.advance();
                Ok(Expr::Num(n))
            }
            TokenKind::Str(s) => {
                self.advance();
                Ok(Expr::Str(s))
            }
            TokenKind::Bytes => {
                let pos = self.peek_pos();
                self.advance();
                self.expect(TokenKind::LParen)?;
                let literal = match self.peek().clone() {
                    TokenKind::Str(s) => {
                        self.advance();
                        s
                    }
                    token => {
                        let pos = self.peek_pos();
                        return Err(ParseError::Expected("string literal".to_string(), token, pos));
                    }
                };
                self.expect(TokenKind::RParen)?;
                let bytes = parse_bytes_literal(&literal).map_err(|reason| ParseError::InvalidBytesLiteral {
                    literal,
                    pos,
                    reason,
                })?;
                Ok(Expr::Bytes(bytes))
            }
            TokenKind::Placeholder => {
                if *self.peek_next() == TokenKind::FatArrow {
                    return Err(ParseError::ReservedPlaceholder);
                }
                self.advance();
                Ok(Expr::Placeholder)
            }
            TokenKind::Ident(name) => {
                if *self.peek_next() == TokenKind::FatArrow {
                    self.advance();
                    self.advance();
                    let body = self.parse_expr()?;
                    Ok(Expr::Lambda(name, Box::new(body)))
                } else {
                    self.advance();
                    Ok(Expr::Ident(name))
                }
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::Seq => {
                self.advance();
                self.expect(TokenKind::LBrack)?;
                let items = self.parse_expr_list(TokenKind::RBrack)?;
                Ok(Expr::Seq(items))
            }
            TokenKind::Set => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let items = self.parse_expr_list(TokenKind::RBrace)?;
                Ok(Expr::Set(items))
            }
            TokenKind::Bag => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let items = self.parse_expr_list(TokenKind::RBrace)?;
                Ok(Expr::Bag(items))
            }
            TokenKind::Map => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut entries = Vec::new();
                if *self.peek() != TokenKind::RBrace {
                    entries.push(self.parse_map_entry()?);
                    while *self.peek() == TokenKind::Comma {
                        self.advance();
                        if *self.peek() == TokenKind::RBrace {
                            break;
                        }
                        entries.push(self.parse_map_entry()?);
                    }
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Expr::Map(entries))
            }
            TokenKind::Prod => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut fields = Vec::new();
                if *self.peek() != TokenKind::RBrace {
                    fields.push(self.parse_prod_field()?);
                    while *self.peek() == TokenKind::Comma {
                        self.advance();
                        if *self.peek() == TokenKind::RBrace {
                            break;
                        }
                        fields.push(self.parse_prod_field()?);
                    }
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Expr::Prod(fields))
            }
            TokenKind::BagKV => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut entries = Vec::new();
                if *self.peek() != TokenKind::RBrace {
                    entries.push(self.parse_bagkv_entry()?);
                    while *self.peek() == TokenKind::Comma {
                        self.advance();
                        if *self.peek() == TokenKind::RBrace {
                            break;
                        }
                        entries.push(self.parse_bagkv_entry()?);
                    }
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Expr::BagKV(entries))
            }
            TokenKind::Some => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Some_(Box::new(expr)))
            }
            TokenKind::None => {
                self.advance();
                Ok(Expr::None_)
            }
            TokenKind::Ok => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Ok_(Box::new(expr)))
            }
            TokenKind::Fail => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let code = self.parse_expr()?;
                self.expect(TokenKind::Comma)?;
                let msg = self.parse_expr()?;
                self.expect(TokenKind::RParen)?;
                Ok(Expr::Fail_(Box::new(code), Box::new(msg)))
            }
            TokenKind::LBrace => {
                self.advance();
                if let Some(err) = self.detect_static_selector_error() {
                    return Err(err);
                }
                self.parse_comprehension()
            }
            token => {
                let pos = self.peek_pos();
                Err(ParseError::Unexpected(token, pos))
            }
        }
    }

    fn parse_comprehension(&mut self) -> Result<Expr, ParseError> {
        if *self.peek() == TokenKind::Yield {
            self.advance();
            let yield_expr = self.parse_expr()?;
            self.expect(TokenKind::Bar)?;
            let (binding, collection) = self.parse_generator_expr()?;
            let pred = if *self.peek() == TokenKind::Bar {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };
            self.expect(TokenKind::RBrace)?;
            return Ok(Expr::Comprehension {
                yield_expr: Some(Box::new(yield_expr)),
                binding,
                collection: Box::new(collection),
                pred,
            });
        }

        let first_expr = self.parse_expr()?;
        if let Some((binding, collection)) = Self::decompose_generator_expr(first_expr.clone()) {
            let pred = if *self.peek() == TokenKind::Bar {
                self.advance();
                Some(Box::new(self.parse_expr()?))
            } else {
                None
            };
            self.expect(TokenKind::RBrace)?;
            return Ok(Expr::Comprehension {
                yield_expr: None,
                binding,
                collection: Box::new(collection),
                pred,
            });
        }

        if matches!(first_expr, Expr::BinOp(BinOpKind::In, _, _)) {
            return Err(self.expected_generator_expr_error());
        }

        self.expect(TokenKind::Bar)?;
        let (binding, collection) = self.parse_generator_expr()?;
        let pred = if *self.peek() == TokenKind::Bar {
            self.advance();
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        self.expect(TokenKind::RBrace)?;
        Ok(Expr::Comprehension {
            yield_expr: Some(Box::new(first_expr)),
            binding,
            collection: Box::new(collection),
            pred,
        })
    }

    fn parse_generator_expr(&mut self) -> Result<(String, Expr), ParseError> {
        let expr = self.parse_expr()?;
        Self::decompose_generator_expr(expr).ok_or_else(|| self.expected_generator_expr_error())
    }

    fn decompose_generator_expr(expr: Expr) -> Option<(String, Expr)> {
        match expr {
            Expr::BinOp(BinOpKind::In, lhs, rhs) => match *lhs {
                Expr::Ident(name) => Some((name, *rhs)),
                _ => None,
            },
            _ => None,
        }
    }

    fn parse_expr_list(&mut self, close: TokenKind) -> Result<Vec<Expr>, ParseError> {
        let mut items = Vec::new();
        if *self.peek() != close {
            items.push(self.parse_expr()?);
            while *self.peek() == TokenKind::Comma {
                self.advance();
                if *self.peek() == close {
                    break;
                }
                items.push(self.parse_expr()?);
            }
        }
        self.expect(close)?;
        Ok(items)
    }

    fn parse_map_entry(&mut self) -> Result<(String, Expr), ParseError> {
        let key = match self.peek().clone() {
            TokenKind::Str(s) => {
                self.advance();
                s
            }
            _ => return Err(ParseError::InvalidMapKey),
        };
        self.expect(TokenKind::Arrow)?;
        let value = self.parse_expr()?;
        Ok((key, value))
    }

    fn parse_bagkv_entry(&mut self) -> Result<(String, Expr), ParseError> {
        let key = match self.peek().clone() {
            TokenKind::Str(s) => {
                self.advance();
                s
            }
            TokenKind::Ident(name) => {
                self.advance();
                name
            }
            _ => return Err(ParseError::InvalidBagkvKey),
        };
        self.expect(TokenKind::Arrow)?;
        let value = self.parse_expr()?;
        Ok((key, value))
    }

    fn parse_prod_field(&mut self) -> Result<(String, Expr), ParseError> {
        let name = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;
        let value = self.parse_expr()?;
        Ok((name, value))
    }

    fn detect_static_selector_error(&self) -> Option<ParseError> {
        let mut labels = Vec::new();
        let mut idx = self.pos;
        while idx < self.tokens.len() {
            match &self.tokens[idx].kind {
                TokenKind::Ident(name) => labels.push(name.clone()),
                TokenKind::Str(s) => labels.push(s.clone()),
                TokenKind::RBrace => break,
                _ => return None,
            }
            idx += 1;
        }

        if idx >= self.tokens.len() || self.tokens[idx].kind != TokenKind::RBrace || labels.is_empty() {
            return None;
        }

        let mut seen = std::collections::BTreeSet::new();
        for label in labels {
            if !seen.insert(label) {
                return Some(ParseError::DuplicateLabelInSelector);
            }
        }

        Some(ParseError::SelectorNotStatic)
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Program, ParseError> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

fn parse_bytes_literal(src: &str) -> Result<Vec<u8>, String> {
    if src.len() % 2 != 0 {
        return Err("expected even-length base16 string".to_string());
    }

    let mut out = Vec::with_capacity(src.len() / 2);
    let mut chars = src.chars();
    while let (Some(hi), Some(lo)) = (chars.next(), chars.next()) {
        let hi = hi
            .to_digit(16)
            .ok_or_else(|| "expected base16 digits only".to_string())?;
        let lo = lo
            .to_digit(16)
            .ok_or_else(|| "expected base16 digits only".to_string())?;
        out.push(((hi << 4) | lo) as u8);
    }
    Ok(out)
}