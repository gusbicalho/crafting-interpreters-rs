use self::state::Position;

use super::{
    source,
    tokens::{Token, TokenInfo, TokenMeta},
};

mod state;
use state::ScanState;

#[derive(Debug, PartialEq, Eq)]
pub struct ScanError {
    #[allow(dead_code)]
    msg: String,
}

impl ScanError {
    pub fn new(_state: &ScanState, msg: String) -> Self {
        Self { msg }
    }
}

pub fn scan(source: source::Source) -> (Vec<ScanError>, Vec<Token>) {
    let mut tokens = vec![];
    let mut errors: Vec<ScanError> = vec![];
    let mut state = ScanState::new(source.text());
    while !state.is_at_end() {
        state.reset_segment();
        match scan_next(&mut state) {
            Err(err) => {
                errors.push(err);
            }
            Ok(Some(token)) => {
                tokens.push(token);
            }
            Ok(None) => {}
        }
    }
    tokens.push(Token::new(String::new(), TokenInfo::EOF, {
        let Position { line, column } = state.current_position();
        TokenMeta::new(*line, *column)
    }));
    (errors, tokens)
}

fn scan_next(state: &mut ScanState) -> Result<Option<Token>, ScanError> {
    if let Some(c) = state.pop_char() {
        match c {
            '(' => Ok(Some(make_token(state, |_| TokenInfo::LeftParen))),
            ')' => Ok(Some(make_token(state, |_| TokenInfo::RightParen))),
            '{' => Ok(Some(make_token(state, |_| TokenInfo::LeftBrace))),
            '}' => Ok(Some(make_token(state, |_| TokenInfo::RightBrace))),
            ':' => Ok(Some(make_token(state, |_| TokenInfo::Colon))),
            ',' => Ok(Some(make_token(state, |_| TokenInfo::Comma))),
            '.' => Ok(Some(make_token(state, |_| TokenInfo::Dot))),
            '-' => Ok(Some(make_token(state, |_| TokenInfo::Minus))),
            '+' => Ok(Some(make_token(state, |_| TokenInfo::Plus))),
            '?' => Ok(Some(make_token(state, |_| TokenInfo::QuestionMark))),
            ';' => Ok(Some(make_token(state, |_| TokenInfo::Semicolon))),
            '*' => Ok(Some(make_token(state, |_| TokenInfo::Star))),
            '!' => Ok(Some({
                if state.match_char('=') {
                    make_token(state, |_| TokenInfo::BangEqual)
                } else {
                    make_token(state, |_| TokenInfo::Bang)
                }
            })),
            '=' => Ok(Some({
                if state.match_char('=') {
                    make_token(state, |_| TokenInfo::EqualEqual)
                } else {
                    make_token(state, |_| TokenInfo::Equal)
                }
            })),
            '<' => Ok(Some({
                if state.match_char('=') {
                    make_token(state, |_| TokenInfo::LessEqual)
                } else {
                    make_token(state, |_| TokenInfo::Less)
                }
            })),
            '>' => Ok(Some({
                if state.match_char('=') {
                    make_token(state, |_| TokenInfo::GreaterEqual)
                } else {
                    make_token(state, |_| TokenInfo::Greater)
                }
            })),
            '/' => {
                if state.match_char('/') {
                    skip_line_comment(state);
                    Ok(None)
                } else {
                    Ok(Some(make_token(state, |_| TokenInfo::Slash)))
                }
            }
            '"' => scan_string_literal(state).map(Some),
            other if other.is_whitespace() => Ok(None),
            other if other.is_ascii_digit() => scan_number_literal(state).map(Some),
            other if is_identifier_first(other) => scan_identifier(state).map(Some),
            other => Err(ScanError::new(
                state,
                format!("Unexpected character {}", other),
            )),
        }
    } else {
        Ok(None)
    }
}

fn make_token<INFO>(state: &mut ScanState, info: INFO) -> Token
where
    INFO: FnOnce(&str) -> TokenInfo,
{
    let (lexeme, start_pos, _) = state.take_segment();
    let info = info(&lexeme[..]);
    Token::new(
        lexeme,
        info,
        TokenMeta::new(start_pos.line, start_pos.column),
    )
}

fn scan_number_literal(state: &mut ScanState) -> Result<Token, ScanError> {
    fn advance_while_digits(state: &mut ScanState) {
        while state.match_pred(|c| c.is_ascii_digit()).is_some() {}
    }
    advance_while_digits(state);
    match state.peek_chars::<2>() {
        Some([dot, digit]) if dot == '.' && digit.is_ascii_digit() => {
            state.pop_char();
            advance_while_digits(state);
        }
        _ => {}
    }
    Ok(make_token(state, |lexeme| {
        TokenInfo::NumberLiteral(lexeme.parse().unwrap())
    }))
}

fn scan_string_literal(state: &mut ScanState) -> Result<Token, ScanError> {
    while state.match_pred(|c| c != '"').is_some() {}
    if !state.match_char('"') {
        Err(ScanError::new(
            state,
            "Unterminated string literal".to_string(),
        ))?;
    }
    Ok(make_token(state, |lexeme| {
        TokenInfo::StringLiteral(lexeme.trim_matches('"').to_string())
    }))
}

fn scan_identifier(state: &mut ScanState) -> Result<Token, ScanError> {
    while state.match_pred(is_identifier_part).is_some() {}
    Ok(make_token(state, |lexeme| match lexeme {
        "and" => TokenInfo::And,
        "class" => TokenInfo::Class,
        "else" => TokenInfo::Else,
        "false" => TokenInfo::False,
        "for" => TokenInfo::For,
        "fun" => TokenInfo::Fun,
        "if" => TokenInfo::If,
        "nil" => TokenInfo::Nil,
        "or" => TokenInfo::Or,
        "return" => TokenInfo::Return,
        "super" => TokenInfo::Super,
        "this" => TokenInfo::This,
        "true" => TokenInfo::True,
        "var" => TokenInfo::Var,
        "while" => TokenInfo::While,
        other => TokenInfo::Identifier(other.to_string()),
    }))
}

fn is_identifier_first(c: char) -> bool {
    c == '_' || c.is_ascii_alphabetic()
}
fn is_identifier_part(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

fn skip_line_comment(state: &mut ScanState) {
    while state.match_pred(|c| c != '\n').is_some() {}
    state.reset_segment();
}

#[cfg(test)]
mod test {
    use crate::pipeline::{
        source,
        tokens::{Token, TokenInfo, TokenMeta},
    };

    use super::{scan, ScanError};

    #[test]
    fn weird_case() {
        let (errors, tokens) = scan(source::from_repl_input("//first comment\n{123.456.789\nand.123.treco&?:// zuera\n \"lol\" )!=!<=<>=>/bla \"erro"));
        assert_eq!(
            errors,
            vec![
                ScanError {
                    msg: "Unexpected character &".to_string()
                },
                ScanError {
                    msg: "Unterminated string literal".to_string()
                }
            ]
        );
        assert_eq!(
            tokens,
            vec![
                Token::new("{".to_string(), TokenInfo::LeftBrace, TokenMeta::new(1, 1)),
                Token::new(
                    "123.456".to_string(),
                    TokenInfo::NumberLiteral(123.456),
                    TokenMeta::new(1, 8)
                ),
                Token::new(".".to_string(), TokenInfo::Dot, TokenMeta::new(1, 9)),
                Token::new(
                    "789".to_string(),
                    TokenInfo::NumberLiteral(789.0),
                    TokenMeta::new(1, 12)
                ),
                Token::new("and".to_string(), TokenInfo::And, TokenMeta::new(2, 3)),
                Token::new(".".to_string(), TokenInfo::Dot, TokenMeta::new(2, 4)),
                Token::new(
                    "123".to_string(),
                    TokenInfo::NumberLiteral(123.0),
                    TokenMeta::new(2, 7)
                ),
                Token::new(".".to_string(), TokenInfo::Dot, TokenMeta::new(2, 8)),
                Token::new(
                    "treco".to_string(),
                    TokenInfo::Identifier("treco".to_string()),
                    TokenMeta::new(2, 13)
                ),
                Token::new(
                    "?".to_string(),
                    TokenInfo::QuestionMark,
                    TokenMeta::new(2, 15)
                ),
                Token::new(":".to_string(), TokenInfo::Colon, TokenMeta::new(2, 16)),
                Token::new(
                    "\"lol\"".to_string(),
                    TokenInfo::StringLiteral("lol".to_string()),
                    TokenMeta::new(3, 6)
                ),
                Token::new(")".to_string(), TokenInfo::RightParen, TokenMeta::new(3, 8)),
                Token::new(
                    "!=".to_string(),
                    TokenInfo::BangEqual,
                    TokenMeta::new(3, 10)
                ),
                Token::new("!".to_string(), TokenInfo::Bang, TokenMeta::new(3, 11)),
                Token::new(
                    "<=".to_string(),
                    TokenInfo::LessEqual,
                    TokenMeta::new(3, 13)
                ),
                Token::new("<".to_string(), TokenInfo::Less, TokenMeta::new(3, 14)),
                Token::new(
                    ">=".to_string(),
                    TokenInfo::GreaterEqual,
                    TokenMeta::new(3, 16)
                ),
                Token::new(">".to_string(), TokenInfo::Greater, TokenMeta::new(3, 17)),
                Token::new("/".to_string(), TokenInfo::Slash, TokenMeta::new(3, 18)),
                Token::new(
                    "bla".to_string(),
                    TokenInfo::Identifier("bla".to_string()),
                    TokenMeta::new(3, 21)
                ),
                Token::new("".to_string(), TokenInfo::EOF, TokenMeta::new(3, 27)),
            ]
        );
    }
}
