use thiserror::Error;

use crate::number::{ExactNum, ParseNumError};

#[derive(Debug, Clone)]
pub enum TokenKind {
    Let,
    Yield,
    Null,
    True,
    False,
    Bytes,
    Seq,
    Set,
    Bag,
    Map,
    Prod,
    BagKV,
    Some,
    None,
    Ok,
    Fail,
    Union,
    Inter,
    Diff,
    BUnion,
    BDiff,
    In,
    And,
    Or,
    Not,
    Arrow,
    FatArrow,
    Pipe,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Concat,
    LParen,
    RParen,
    LBrack,
    RBrack,
    LBrace,
    RBrace,
    QMark,
    Bang,
    Colon,
    Comma,
    Semi,
    Bar,
    SelL,
    SelR,
    Ident(String),
    Str(String),
    Num(ExactNum),
    Placeholder,
    Eof,
}

impl PartialEq for TokenKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TokenKind::Num(a), TokenKind::Num(b)) => a == b,
            (TokenKind::Ident(a), TokenKind::Ident(b)) => a == b,
            (TokenKind::Str(a), TokenKind::Str(b)) => a == b,
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: usize,
}

#[derive(Debug, Error)]
pub enum LexError {
    #[error("Unexpected character '{0}' at position {1}")]
    UnexpectedChar(char, usize),
    #[error("Unterminated string at position {0}")]
    UnterminatedString(usize),
    #[error("Invalid numeric literal '{literal}' at position {pos}: {source}")]
    InvalidNumber {
        literal: String,
        pos: usize,
        #[source]
        source: ParseNumError,
    },
}

pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    let chars: Vec<char> = src.chars().collect();
    let mut pos = 0;
    let mut tokens = Vec::new();

    while pos < chars.len() {
        let start = pos;
        let ch = chars[pos];

        if ch.is_whitespace() {
            pos += 1;
            continue;
        }

        if ch == ';' && pos + 1 < chars.len() && chars[pos + 1] == ';' {
            while pos < chars.len() && chars[pos] != '\n' {
                pos += 1;
            }
            continue;
        }

        if ch.is_ascii_digit() || (ch == '.' && pos + 1 < chars.len() && chars[pos + 1].is_ascii_digit()) {
            let num_start = pos;
            while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
                pos += 1;
            }
            if pos < chars.len() && (chars[pos] == 'e' || chars[pos] == 'E') {
                pos += 1;
                if pos < chars.len() && (chars[pos] == '+' || chars[pos] == '-') {
                    pos += 1;
                }
                while pos < chars.len() && chars[pos].is_ascii_digit() {
                    pos += 1;
                }
            }
            let num_str: String = chars[num_start..pos].iter().collect();
            let n = ExactNum::parse_literal(&num_str).map_err(|source| LexError::InvalidNumber {
                literal: num_str.clone(),
                pos: start,
                source,
            })?;
            tokens.push(Token {
                kind: TokenKind::Num(n),
                pos: start,
            });
            continue;
        }

        if ch == '"' {
            pos += 1;
            let mut s = String::new();
            while pos < chars.len() && chars[pos] != '"' {
                if chars[pos] == '\\' {
                    pos += 1;
                    if pos >= chars.len() {
                        return Err(LexError::UnterminatedString(start));
                    }
                    match chars[pos] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '"' => s.push('"'),
                        '\\' => s.push('\\'),
                        c => {
                            s.push('\\');
                            s.push(c);
                        }
                    }
                } else {
                    s.push(chars[pos]);
                }
                pos += 1;
            }
            if pos >= chars.len() {
                return Err(LexError::UnterminatedString(start));
            }
            pos += 1;
            tokens.push(Token {
                kind: TokenKind::Str(s),
                pos: start,
            });
            continue;
        }

        if ch.is_alphabetic() || ch == '_' {
            if ch == '_' {
                let next = pos + 1;
                if next >= chars.len() || (!chars[next].is_alphanumeric() && chars[next] != '_') {
                    pos += 1;
                    tokens.push(Token {
                        kind: TokenKind::Placeholder,
                        pos: start,
                    });
                    continue;
                }
            }
            let id_start = pos;
            while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                pos += 1;
            }
            let ident: String = chars[id_start..pos].iter().collect();
            let kind = match ident.to_ascii_lowercase().as_str() {
                "let" => TokenKind::Let,
                "yield" => TokenKind::Yield,
                "null" => TokenKind::Null,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                "bytes" => TokenKind::Bytes,
                "seq" => TokenKind::Seq,
                "set" => TokenKind::Set,
                "bag" => TokenKind::Bag,
                "map" => TokenKind::Map,
                "prod" => TokenKind::Prod,
                "bagkv" => TokenKind::BagKV,
                "some" => TokenKind::Some,
                "none" => TokenKind::None,
                "ok" => TokenKind::Ok,
                "fail" => TokenKind::Fail,
                "union" => TokenKind::Union,
                "inter" => TokenKind::Inter,
                "diff" => TokenKind::Diff,
                "bunion" => TokenKind::BUnion,
                "bdiff" => TokenKind::BDiff,
                "in" => TokenKind::In,
                "and" => TokenKind::And,
                "or" => TokenKind::Or,
                "not" => TokenKind::Not,
                _ => TokenKind::Ident(ident),
            };
            tokens.push(Token { kind, pos: start });
            continue;
        }

        let kind = match ch {
            '→' => {
                pos += 1;
                TokenKind::Arrow
            }
            '↦' => {
                pos += 1;
                TokenKind::FatArrow
            }
            '∈' => {
                pos += 1;
                TokenKind::In
            }
            '∧' => {
                pos += 1;
                TokenKind::And
            }
            '∨' => {
                pos += 1;
                TokenKind::Or
            }
            '¬' => {
                pos += 1;
                TokenKind::Not
            }
            '⟨' => {
                pos += 1;
                TokenKind::SelL
            }
            '⟩' => {
                pos += 1;
                TokenKind::SelR
            }
            '∣' => {
                pos += 1;
                TokenKind::Bar
            }
            '⊎' => {
                pos += 1;
                TokenKind::BUnion
            }
            '⊖' => {
                pos += 1;
                TokenKind::BDiff
            }
            '≠' => {
                pos += 1;
                TokenKind::Neq
            }
            '≤' => {
                pos += 1;
                TokenKind::Le
            }
            '≥' => {
                pos += 1;
                TokenKind::Ge
            }
            '•' => {
                pos += 1;
                TokenKind::Placeholder
            }
            _ => match ch {
                '-' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '>' {
                        pos += 2;
                        TokenKind::Arrow
                    } else {
                        pos += 1;
                        TokenKind::Minus
                    }
                }
                '=' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '>' {
                        pos += 2;
                        TokenKind::FatArrow
                    } else {
                        pos += 1;
                        TokenKind::Eq
                    }
                }
                '!' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        pos += 2;
                        TokenKind::Neq
                    } else {
                        pos += 1;
                        TokenKind::Bang
                    }
                }
                '<' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        pos += 2;
                        TokenKind::Le
                    } else {
                        pos += 1;
                        TokenKind::Lt
                    }
                }
                '>' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '=' {
                        pos += 2;
                        TokenKind::Ge
                    } else {
                        pos += 1;
                        TokenKind::Gt
                    }
                }
                '|' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '>' {
                        pos += 2;
                        TokenKind::Pipe
                    } else {
                        pos += 1;
                        TokenKind::Bar
                    }
                }
                '+' => {
                    if pos + 1 < chars.len() && chars[pos + 1] == '+' {
                        pos += 2;
                        TokenKind::Concat
                    } else {
                        pos += 1;
                        TokenKind::Plus
                    }
                }
                '*' => {
                    pos += 1;
                    TokenKind::Star
                }
                '/' => {
                    pos += 1;
                    TokenKind::Slash
                }
                '(' => {
                    pos += 1;
                    TokenKind::LParen
                }
                ')' => {
                    pos += 1;
                    TokenKind::RParen
                }
                '[' => {
                    pos += 1;
                    TokenKind::LBrack
                }
                ']' => {
                    pos += 1;
                    TokenKind::RBrack
                }
                '{' => {
                    pos += 1;
                    TokenKind::LBrace
                }
                '}' => {
                    pos += 1;
                    TokenKind::RBrace
                }
                '?' => {
                    pos += 1;
                    TokenKind::QMark
                }
                ':' => {
                    pos += 1;
                    TokenKind::Colon
                }
                ',' => {
                    pos += 1;
                    TokenKind::Comma
                }
                ';' => {
                    pos += 1;
                    TokenKind::Semi
                }
                _ => return Err(LexError::UnexpectedChar(ch, start)),
            },
        };
        tokens.push(Token { kind, pos: start });
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        pos: chars.len(),
    });
    Ok(tokens)
}