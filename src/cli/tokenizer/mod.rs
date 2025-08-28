pub mod scanner;
pub mod token;
use crate::cli::tokenizer::scanner::Token;

pub fn tokenize<'a>(line: &'a str) -> Vec<Token<'a>> {
    let mut tokens: Vec<Token<'a>> = vec![];
    let mut tokenizer = scanner::Scanner::new(line);
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

#[cfg(test)]
mod tests {
    use super::*;
    use token::TokenTypes;

    fn token(tt: TokenTypes, val: &'static str, col: usize, line_num: usize) -> Token<'static> {
        Token {
            token_type: tt,
            value: val,
            col_num: col,
            line_num: line_num,
        }
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
            token(TokenTypes::SemiColon, ";", 43, 1),
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
