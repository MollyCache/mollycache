
#[cfg(test)]
use crate::cli::tokenizer::token::TokenTypes;
#[cfg(test)]
use crate::cli::tokenizer::scanner::Token;

#[cfg(test)]
pub fn token(tt: TokenTypes, val: &'static str) -> Token<'static> {
    Token {
        token_type: tt,
        value: val,
        col_num: 0,
        line_num: 1,
    }
}

#[cfg(test)]
pub fn token_with_location(tt: TokenTypes, val: &'static str, col: usize, line: usize) -> Token<'static> {
    Token {
        token_type: tt,
        value: val,
        col_num: col,
        line_num: line,
    }
}
