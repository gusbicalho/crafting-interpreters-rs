use std::fmt::{self, Display, Formatter};

#[derive(PartialEq)]
pub struct Token {
    lexeme: String,
    info: TokenInfo,
    meta: TokenMeta,
}

impl Token {
    pub fn new(lexeme: String, info: TokenInfo, meta: TokenMeta) -> Self {
        Self { lexeme, info, meta }
    }

    pub fn meta(&self) -> &TokenMeta {
        &self.meta
    }

    pub fn lexeme(&self) -> &str {
        self.lexeme.as_ref()
    }

    pub fn info(&self) -> &TokenInfo {
        &self.info
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.lexeme)
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Token<{},{},{:?}>({})",
            self.meta.line, self.meta.column, self.info, self.lexeme
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct TokenMeta {
    line: usize,
    column: usize,
}

impl TokenMeta {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenInfo {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Colon,
    Comma,
    Dot,
    Minus,
    Plus,
    QuestionMark,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals
    Identifier(String),
    StringLiteral(String),
    NumberLiteral(f64),
    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    // EOF
    EOF,
}
