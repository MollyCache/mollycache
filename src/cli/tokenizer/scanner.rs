use crate::cli::tokenizer::token::{self, TokenTypes};

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
    col_num: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(input: &'a str) -> Self {
        return Self {
            input,
            current: 0,
            line_num: 1,
            col_num: 0,
        };
    }

    fn handle_skips(&mut self) -> bool {
        if self.current_char() == ' ' {
            self.advance();
            return true;
        } else if self.current_char() == '\n' {
            self.advance();
            self.line_num += 1;
            self.col_num = self.current;
            return true;
        }
        return false;
    }

    fn advance(&mut self) {
        self.current += 1;
    }

    fn current_char(&self) -> char {
        return self.input[self.current..].chars().next().unwrap_or('\0');
    }

    fn peek_char(&self) -> char {
        return self.input[self.current + 1..].chars().next().unwrap_or('\0');
    }

    fn build_token(&mut self, start: usize, token_type: TokenTypes) -> Token<'a> {
        self.advance();
        return Token {
            token_type: token_type,
            value: &self.input[start..self.current],
            col_num: start-self.col_num,
            line_num: self.line_num,
        };
    }

    fn build_string_token(&mut self, start: usize, token_type: TokenTypes) -> Token<'a> {
        return match token_type {
            TokenTypes::String => { 
            self.advance();
                Token {
                    token_type: token_type,
                    value: &self.input[start+1..self.current-1],
                    col_num: start-self.col_num,
                    line_num: self.line_num,
                }
            }
            _ => self.build_token(start, token_type)
        };
    }

    fn build_hex_literal_token(&mut self, start: usize, token_type: TokenTypes) -> Token<'a> {
        return match token_type {
            TokenTypes::HexLiteral => {
                self.advance();
                Token {
                    token_type: token_type,
                    value: &self.input[start+2..self.current-1],
                    col_num: start-self.col_num,
                    line_num: self.line_num,
                }
            }
            _ => self.build_token(start, token_type)
        }
    }

    fn read_string(&mut self) -> TokenTypes {
        self.advance();
        while self.current_char() != '"' {
            self.advance();
            if self.current >= self.input.len() {
                self.current = self.input.len() - 1;
                return TokenTypes::Error;
            }
        }
        return TokenTypes::String;
    }

    fn read_identifier(&mut self, start: usize) -> TokenTypes {
        while self.peek_char().is_alphanumeric() || self.peek_char() == '_' {
            self.advance();
        }
        return match &self.input[start..=self.current] {
            slice if slice.eq_ignore_ascii_case("CREATE") => TokenTypes::Create,
            slice if slice.eq_ignore_ascii_case("SELECT") => TokenTypes::Select,
            slice if slice.eq_ignore_ascii_case("INSERT") => TokenTypes::Insert,
            slice if slice.eq_ignore_ascii_case("TABLE") => TokenTypes::Table,
            slice if slice.eq_ignore_ascii_case("FROM") => TokenTypes::From,
            slice if slice.eq_ignore_ascii_case("INTO") => TokenTypes::Into,
            slice if slice.eq_ignore_ascii_case("VALUES") => TokenTypes::Values,
            slice if slice.eq_ignore_ascii_case("WHERE") => TokenTypes::Where,
            slice if slice.eq_ignore_ascii_case("UPDATE") => TokenTypes::Update,
            slice if slice.eq_ignore_ascii_case("DELETE") => TokenTypes::Delete,
            slice if slice.eq_ignore_ascii_case("DROP") => TokenTypes::Drop,
            slice if slice.eq_ignore_ascii_case("INDEX") => TokenTypes::Index,
            slice if slice.eq_ignore_ascii_case("INTEGER") => TokenTypes::Integer,
            slice if slice.eq_ignore_ascii_case("REAL") => TokenTypes::Real,
            slice if slice.eq_ignore_ascii_case("TEXT") => TokenTypes::Text,
            slice if slice.eq_ignore_ascii_case("BLOB") => TokenTypes::Blob,
            slice if slice.eq_ignore_ascii_case("NULL") => TokenTypes::Null,
            slice if slice.eq_ignore_ascii_case("PRIMARY") => TokenTypes::Primary,
            slice if slice.eq_ignore_ascii_case("KEY") => TokenTypes::Key,
            slice if slice.eq_ignore_ascii_case("NOT") => TokenTypes::Not,
            slice if slice.eq_ignore_ascii_case("UNIQUE") => TokenTypes::Unique,
            slice if slice.eq_ignore_ascii_case("DEFAULT") => TokenTypes::Default,
            slice if slice.eq_ignore_ascii_case("AUTOINCREMENT") => TokenTypes::AutoIncrement,
            slice if slice.eq_ignore_ascii_case("ORDER") => TokenTypes::Order,
            slice if slice.eq_ignore_ascii_case("BY") => TokenTypes::By,
            slice if slice.eq_ignore_ascii_case("GROUP") => TokenTypes::Group,
            slice if slice.eq_ignore_ascii_case("HAVING") => TokenTypes::Having,
            slice if slice.eq_ignore_ascii_case("DISTINCT") => TokenTypes::Distinct,
            slice if slice.eq_ignore_ascii_case("ALL") => TokenTypes::All,
            slice if slice.eq_ignore_ascii_case("AS") => TokenTypes::As,
            slice if slice.eq_ignore_ascii_case("INNER") => TokenTypes::Inner,
            slice if slice.eq_ignore_ascii_case("LEFT") => TokenTypes::Left,
            slice if slice.eq_ignore_ascii_case("RIGHT") => TokenTypes::Right,
            slice if slice.eq_ignore_ascii_case("FULL") => TokenTypes::Full,
            slice if slice.eq_ignore_ascii_case("OUTER") => TokenTypes::Outer,
            slice if slice.eq_ignore_ascii_case("JOIN") => TokenTypes::Join,
            slice if slice.eq_ignore_ascii_case("ON") => TokenTypes::On,
            slice if slice.eq_ignore_ascii_case("UNION") => TokenTypes::Union,
            slice if slice.eq_ignore_ascii_case("LIMIT") => TokenTypes::Limit,
            slice if slice.eq_ignore_ascii_case("OFFSET") => TokenTypes::Offset,
            slice if slice.eq_ignore_ascii_case("AND") => TokenTypes::And,
            slice if slice.eq_ignore_ascii_case("OR") => TokenTypes::Or,
            slice if slice.eq_ignore_ascii_case("IN") => TokenTypes::In,
            slice if slice.eq_ignore_ascii_case("EXISTS") => TokenTypes::Exists,
            slice if slice.eq_ignore_ascii_case("CASE") => TokenTypes::Case,
            slice if slice.eq_ignore_ascii_case("WHEN") => TokenTypes::When,
            slice if slice.eq_ignore_ascii_case("THEN") => TokenTypes::Then,
            slice if slice.eq_ignore_ascii_case("ELSE") => TokenTypes::Else,
            slice if slice.eq_ignore_ascii_case("END") => TokenTypes::End,
            slice if slice.eq_ignore_ascii_case("COUNT") => TokenTypes::Count,
            slice if slice.eq_ignore_ascii_case("SUM") => TokenTypes::Sum,
            slice if slice.eq_ignore_ascii_case("AVG") => TokenTypes::Avg,
            slice if slice.eq_ignore_ascii_case("MIN") => TokenTypes::Min,
            slice if slice.eq_ignore_ascii_case("MAX") => TokenTypes::Max,
            slice if slice.eq_ignore_ascii_case("TRUE") => TokenTypes::True,
            slice if slice.eq_ignore_ascii_case("FALSE") => TokenTypes::False,
            _ => TokenTypes::Identifier,
        };
    }

    fn read_digit(&mut self) -> TokenTypes {
        let mut token_type = TokenTypes::IntLiteral;
        while self.peek_char().is_ascii_digit() || self.peek_char() == '.' || self.peek_char() == 'e' || self.peek_char() == '-' {
            if self.peek_char() == '.' || self.peek_char() == 'e' {
                token_type = TokenTypes::RealLiteral;
            }
            self.advance();
        }
        return token_type;
    }

    fn read_hex_literal(&mut self) -> TokenTypes {
        self.advance();
        self.advance();
        while self.current_char().is_ascii_hexdigit() {
            self.advance();
        }
        if self.current_char() == '\'' {
            return TokenTypes::HexLiteral;
        } else {
            return TokenTypes::Error;
        }
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
                Some(self.build_string_token(start, token_type))
            }
            c if c == 'X' && self.peek_char() == '\'' => {
                let token_type = self.read_hex_literal();
                Some(self.build_hex_literal_token(start, token_type))
            }
            c if c.is_ascii_alphabetic() || c == '_' => {
                let token_type = self.read_identifier(start);
                Some(self.build_token(start, token_type))
            }
            c if c.is_ascii_digit() => {
                let token_type = self.read_digit();
                Some(self.build_token(start, token_type))
            }
            '*' => Some(self.build_token(start, TokenTypes::Asterisk)),
            ';' => Some(self.build_token(start, TokenTypes::SemiColon)),
            '(' => Some(self.build_token(start, TokenTypes::LeftParen)),
            ')' => Some(self.build_token(start, TokenTypes::RightParen)),
            ',' => Some(self.build_token(start, TokenTypes::Comma)),
            '.' => Some(self.build_token(start, TokenTypes::Dot)),
            '+' => Some(self.build_token(start, TokenTypes::Plus)),
            '-' => {
                if self.peek_char().is_ascii_digit() {
                    self.advance();
                    let token_type = self.read_digit();
                    Some(self.build_token(start, token_type))
                } else {
                    Some(self.build_token(start, TokenTypes::Minus))
                }
            }
            '/' => Some(self.build_token(start, TokenTypes::Divide)),
            '%' => Some(self.build_token(start, TokenTypes::Modulo)),
            '=' => Some(self.build_token(start, TokenTypes::Equals)),
            '!' => {
                if self.peek_char() == '=' {
                    self.advance();
                    Some(self.build_token(start, TokenTypes::NotEquals))
                } else {
                    Some(self.build_token(start, TokenTypes::Error))
                }
            },
            '<' => {
                if self.peek_char() == '=' {
                    self.advance();
                    Some(self.build_token(start, TokenTypes::LessEquals))
                } else {
                    Some(self.build_token(start, TokenTypes::LessThan))
                }
            },
            '>' => {
                if self.peek_char() == '=' {
                    self.advance();
                    Some(self.build_token(start, TokenTypes::GreaterEquals))
                } else {
                    Some(self.build_token(start, TokenTypes::GreaterThan))
                }
            },
            _ => Some(self.build_token(start, TokenTypes::Error)),
        };
    }
}