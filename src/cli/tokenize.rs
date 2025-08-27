#[derive(Debug)]
#[derive(PartialEq)]
pub enum TokenTypes {
    // Keywords:
    Create, Select, Insert, Table, From, Into, Values,
    // Single-char tokens:
    Asterix, SemiColon, LeftParen, RightParen, Comma, 
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
    
    fn skip_whitespace(&mut self){
        while self.current_char() == ' ' {
            self.current += 1;
        }
    }

    fn handle_newlines(&mut self){
        while self.current_char() == '\n' {
            self.current += 1;
            self.line_num += 1;
        }
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

    fn read_string(&mut self){
        self.current += 1;
        while self.current_char() != '"'{
            self.current += 1;
        }
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
        self.skip_whitespace();
        self.handle_newlines();

        if self.current >= self.input.len() {
            return None;
        }
        let start = self.current;
        return match self.current_char() {
            '"' => {
                self.read_string();
                Some(self.build_token(start, TokenTypes::String))
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
            _ => Some(self.build_token(start, TokenTypes::Error))
        };
    }
}