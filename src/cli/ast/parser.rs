use crate::cli::{
    ast::SqlStatement, ast::create_statement, ast::insert_statement, ast::select_statement,
    tokenizer::scanner::Token, tokenizer::token::TokenTypes,
};

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        return Self { tokens, current: 0 };
    }

    pub fn current_token(&self) -> Option<&Token<'a>> {
        if self.current >= self.tokens.len() {
            return None;
        }
        return Some(&self.tokens[self.current]);
    }

    pub fn advance(&mut self) {
        self.current += 1;
    }

    pub fn format_error(&self) -> String {
        if let Some(token) = self.current_token() {
            return format!(
                "Error at line {:?}, column {:?}: Unexpected type: {:?}",
                token.line_num, token.col_num, token.token_type
            );
        } else {
            return "Error at end of input.".to_string();
        }
    }

    pub fn next_statement(&mut self) -> Option<Result<SqlStatement, String>> {
        if self.tokens.len() == 0 {
            return Some(Err("No tokens to parse".to_string()));
        }
        return match self.current_token()?.token_type {
            TokenTypes::Create => Some(create_statement::build(self)),
            TokenTypes::Insert => Some(insert_statement::build(self)),
            TokenTypes::Select => Some(select_statement::build(self)),
            TokenTypes::EOF => None,
            _ => {
                self.advance();
                Some(Err(self.format_error()))
            }
        };
    }
}
