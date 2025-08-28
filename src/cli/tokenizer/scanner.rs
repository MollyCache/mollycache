use crate::cli::tokenizer::token::TokenTypes;

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub token_type: TokenTypes,
    pub value: &'a str,
    pub col_num: usize,
    pub line_num: usize,
}

pub struct Scanner<'a> {
    input: &'a str,
    current: usize,
    line_num: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(input: &'a str) -> Self {
        return Self {
            input,
            current: 0,
            line_num: 1,
        };
    }

    fn handle_skips(&mut self) -> bool {
        if self.current_char() == ' ' {
            self.current += 1;
            return true;
        } else if self.current_char() == '\n' {
            self.current += 1;
            self.line_num += 1;
            return true;
        }
        return false;
    }

    fn current_char(&self) -> char {
        return self.input[self.current..].chars().next().unwrap_or('\0');
    }

    fn build_token(&mut self, start: usize, token_type: TokenTypes) -> Token<'a> {
        self.current += 1;
        return Token {
            token_type: token_type,
            value: &self.input[start..self.current],
            col_num: start,
            line_num: self.line_num,
        };
    }

    fn read_string(&mut self) -> TokenTypes {
        self.current += 1;
        while self.current_char() != '"' {
            self.current += 1;
            if self.current >= self.input.len() {
                self.current = self.input.len() - 1;
                return TokenTypes::Error;
            }
        }
        return TokenTypes::String;
    }

    fn read_identifier(&mut self, start: usize) -> TokenTypes {
        self.current += 1;
        while self.current_char().is_alphanumeric() || self.current_char() == '_' {
            self.current += 1;
        }
        self.current -= 1;
        return match &self.input[start..=self.current] {
            slice if slice.eq_ignore_ascii_case("CREATE") => TokenTypes::Create,
            slice if slice.eq_ignore_ascii_case("SELECT") => TokenTypes::Select,
            slice if slice.eq_ignore_ascii_case("INSERT") => TokenTypes::Insert,
            slice if slice.eq_ignore_ascii_case("TABLE") => TokenTypes::Table,
            slice if slice.eq_ignore_ascii_case("FROM") => TokenTypes::From,
            slice if slice.eq_ignore_ascii_case("INTO") => TokenTypes::Into,
            slice if slice.eq_ignore_ascii_case("VALUES") => TokenTypes::Values,
            slice if slice.eq_ignore_ascii_case("WHERE") => TokenTypes::Where,
            _ => TokenTypes::Identifier,
        };
    }

    fn read_digit(&mut self) {
        while self.current_char().is_ascii_digit() {
            self.current += 1;
        }
        self.current -= 1;
    }

    pub fn next_token(&mut self) -> Option<Token<'a>> {
        while self.handle_skips() {}

        if self.current >= self.input.len() {
            return None;
        }
        let start = self.current;
        return match self.current_char() {
            '"' => {
                let token_type = self.read_string();
                Some(self.build_token(start, token_type))
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let token_type = self.read_identifier(start);
                Some(self.build_token(start, token_type))
            }
            c if c.is_ascii_digit() => {
                self.read_digit();
                Some(self.build_token(start, TokenTypes::Number))
            }
            '*' => Some(self.build_token(start, TokenTypes::Asterix)),
            ';' => Some(self.build_token(start, TokenTypes::SemiColon)),
            '(' => Some(self.build_token(start, TokenTypes::LeftParen)),
            ')' => Some(self.build_token(start, TokenTypes::RightParen)),
            ',' => Some(self.build_token(start, TokenTypes::Comma)),
            '=' => Some(self.build_token(start, TokenTypes::Equals)),
            _ => Some(self.build_token(start, TokenTypes::Error)),
        };
    }
}
