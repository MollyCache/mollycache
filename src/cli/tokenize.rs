#[derive(Debug)]
#[derive(PartialEq)]
pub enum TokenTypes {
    // Keywords:
    Create, Select, Insert, Table, From, Into, Values, Where,
    // Single-char tokens:
    Asterix, SemiColon, LeftParen, RightParen, Comma, Equals,
    // Literals:
    String, Number,
    // Identifier
    Identifier,
    
    // Other
    Error
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Token<'a> {
    token_type: TokenTypes,
    value: &'a str,
    col_num: usize,
    line_num: usize
}

pub fn tokenize<'a>(line: &'a str) -> Vec<Token<'a>> {
    let mut tokens : Vec<Token<'a>> = vec![];
    let mut tokenizer = Tokenizer::new(line);
    loop {
        let next_token = tokenizer.next_token();
        if let Some(next_token) = next_token {
            tokens.push(next_token);
        } else {
            break;
        }
    }
    return tokens;    
}

struct Tokenizer<'a> {
    input: &'a str,
    current: usize,
    line_num: usize
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        return Self {input, current: 0, line_num: 1};
    }
    
    fn handle_skips(&mut self) -> bool{
        if self.current_char() == ' ' {
            self.current += 1;
            return true;
        }
        else if self.current_char() == '\n' {
            self.current += 1;
            self.line_num += 1;
            return true;
        }
        return false
    }

    fn current_char(&self) -> char{
        return self.input[self.current..].chars().next().unwrap_or('\0')
    }

    fn build_token(&mut self, start: usize, token_type: TokenTypes) -> Token<'a> {
        self.current += 1;
        return Token {
            token_type: token_type,
            value: &self.input[start..self.current],
            col_num: start,
            line_num: self.line_num
        }
    }

    fn read_string(&mut self) -> TokenTypes {
        self.current += 1;
        while self.current_char() != '"'{
            self.current += 1;
            if  self.current >= self.input.len() {
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
            _ => TokenTypes::Identifier
        }
    }

    fn read_digit(&mut self) {
        while self.current_char().is_ascii_digit() {
            self.current += 1;
        }
        self.current -= 1;
    }

    fn next_token(&mut self) -> Option<Token<'a>> {
        while self.handle_skips() {}

        if self.current >= self.input.len() {
            return None;
        }
        let start = self.current;
        return match self.current_char() {
            '"' => {
                let token_type = self.read_string();
                Some(self.build_token(start, token_type))
            },
            c if c.is_ascii_alphabetic() || c == '_' => {
                let token_type = self.read_identifier(start);
                Some(self.build_token(start, token_type))
            },
            c if c.is_ascii_digit() => {
                self.read_digit();
                Some(self.build_token(start, TokenTypes::Number))
            },
            '*' => Some(self.build_token(start, TokenTypes::Asterix)),
            ';' => Some(self.build_token(start, TokenTypes::SemiColon)),
            '(' => Some(self.build_token(start, TokenTypes::LeftParen)),
            ')' => Some(self.build_token(start, TokenTypes::RightParen)),
            ',' => Some(self.build_token(start, TokenTypes::Comma)),
            '=' => Some(self.build_token(start, TokenTypes::Equals)),
            _ => Some(self.build_token(start, TokenTypes::Error))
        };
    }
}


#[cfg(test)]
mod tests {
    use super::*;

     fn token(tt: TokenTypes, val: &'static str, col: usize, line_num: usize) -> Token<'static> {
        Token { token_type: tt, value: val, col_num: col, line_num: line_num }
    }

    #[test]
    fn tokenizer_parses_select_statement_v1() {
        let result = tokenize("SELECT * FROM users WHERE name = \"Fletcher\";");
        let expected = vec![
            token(TokenTypes::Select, "SELECT", 0, 1),
            token(TokenTypes::Asterix, "*", 7, 1),
            token(TokenTypes::From, "FROM", 9, 1),
            token(TokenTypes::Identifier, "users", 14, 1),
            token(TokenTypes::Where, "WHERE", 20, 1),
            token(TokenTypes::Identifier, "name", 26, 1),
            token(TokenTypes::Equals, "=", 31, 1),
            token(TokenTypes::String, "\"Fletcher\"", 33, 1),
            token(TokenTypes::SemiColon, ";", 43, 1)
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn tokenizer_raises_error_when_token_cannot_be_matched() {
        let result = tokenize("Create INSERT TABLE VALUES, (199) \n  \"Fletcher\"\";");
        let expected = vec![
            token(TokenTypes::Create, "Create", 0, 1),
            token(TokenTypes::Insert, "INSERT", 7, 1),
            token(TokenTypes::Table, "TABLE", 14, 1),
            token(TokenTypes::Values, "VALUES", 20, 1),
            token(TokenTypes::Comma, ",", 26, 1),
            token(TokenTypes::LeftParen, "(", 28, 1),
            token(TokenTypes::Number, "199", 29, 1),
            token(TokenTypes::RightParen, ")", 32, 1),
            token(TokenTypes::String, "\"Fletcher\"", 37, 2),
            token(TokenTypes::Error, "\";", 47, 2),
        ];
        assert_eq!(expected, result);
    }
}