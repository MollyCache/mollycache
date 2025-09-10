pub mod scanner;
pub mod token;
use crate::interpreter::tokenizer::scanner::Token;

pub fn tokenize<'a>(line: &'a str) -> Vec<Token<'a>> {
    let mut tokens: Vec<Token<'a>> = vec![];
    let mut tokenizer = scanner::Scanner::new(line);
    loop {
        let next_token = tokenizer.next_token();
        if let Some(next_token) = next_token {
            tokens.push(next_token);
        } else {
            tokens.push(Token {
                token_type: token::TokenTypes::EOF,
                value: "",
                col_num: 0,
                line_num: 0,
            });
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
        let result = tokenize("SELECT * FROM users WHERE name = \'Fletcher\';");
        let expected = vec![
            token(TokenTypes::Select, "SELECT", 0, 1),
            token(TokenTypes::Asterisk, "*", 7, 1),
            token(TokenTypes::From, "FROM", 9, 1),
            token(TokenTypes::Identifier, "users", 14, 1),
            token(TokenTypes::Where, "WHERE", 20, 1),
            token(TokenTypes::Identifier, "name", 26, 1),
            token(TokenTypes::Equals, "=", 31, 1),
            token(TokenTypes::String, "Fletcher", 33, 1),
            token(TokenTypes::SemiColon, ";", 43, 1),
            token(TokenTypes::EOF, "", 0, 0),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn tokenizer_raises_error_when_token_cannot_be_matched() {
        let result = tokenize("Create INSERT TABLE VALUES, (199) \n\'Fletcher\'\"");
        let expected = vec![
            token(TokenTypes::Create, "Create", 0, 1),
            token(TokenTypes::Insert, "INSERT", 7, 1),
            token(TokenTypes::Table, "TABLE", 14, 1),
            token(TokenTypes::Values, "VALUES", 20, 1),
            token(TokenTypes::Comma, ",", 26, 1),
            token(TokenTypes::LeftParen, "(", 28, 1),
            token(TokenTypes::IntLiteral, "199", 29, 1),
            token(TokenTypes::RightParen, ")", 32, 1),
            token(TokenTypes::String, "Fletcher", 0, 2),
            token(TokenTypes::Error, "\"", 10, 2),
            token(TokenTypes::EOF, "", 0, 0),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn tokens_are_implemented_correctly() {
        let statement = r#"
        CREATE SELECT INSERT TABLE FROM INTO VALUES WHERE
        UPDATE DELETE DROP INDEX SET
        INTEGER REAL TEXT BLOB NULL
        PRIMARY KEY NOT UNIQUE DEFAULT AUTOINCREMENT
        ORDER BY GROUP HAVING DISTINCT ALL AS ASC DESC
        INNER LEFT RIGHT FULL OUTER JOIN ON UNION
        LIMIT OFFSET
        AND OR IN EXISTS IF
        CASE WHEN THEN ELSE END
        = != < <= > >=
        COUNT SUM AVG MIN MAX
        * ; ( ) , .
        + - / %
        'string' TRUE FALSE
        fletchers_table
        "#;
        let result = tokenize(statement);
        let expected = vec![
            token(TokenTypes::Create, "CREATE", 8, 2),
            token(TokenTypes::Select, "SELECT", 15, 2),
            token(TokenTypes::Insert, "INSERT", 22, 2),
            token(TokenTypes::Table, "TABLE", 29, 2),
            token(TokenTypes::From, "FROM", 35, 2),
            token(TokenTypes::Into, "INTO", 40, 2),
            token(TokenTypes::Values, "VALUES", 45, 2),
            token(TokenTypes::Where, "WHERE", 52, 2),
            token(TokenTypes::Update, "UPDATE", 8, 3),
            token(TokenTypes::Delete, "DELETE", 15, 3),
            token(TokenTypes::Drop, "DROP", 22, 3),
            token(TokenTypes::Index, "INDEX", 27, 3),
            token(TokenTypes::Set, "SET", 33, 3),
            token(TokenTypes::Integer, "INTEGER", 8, 4),
            token(TokenTypes::Real, "REAL", 16, 4),
            token(TokenTypes::Text, "TEXT", 21, 4),
            token(TokenTypes::Blob, "BLOB", 26, 4),
            token(TokenTypes::Null, "NULL", 31, 4),
            token(TokenTypes::Primary, "PRIMARY", 8, 5),
            token(TokenTypes::Key, "KEY", 16, 5),
            token(TokenTypes::Not, "NOT", 20, 5),
            token(TokenTypes::Unique, "UNIQUE", 24, 5),
            token(TokenTypes::Default, "DEFAULT", 31, 5),
            token(TokenTypes::AutoIncrement, "AUTOINCREMENT", 39, 5),
            token(TokenTypes::Order, "ORDER", 8, 6),
            token(TokenTypes::By, "BY", 14, 6),
            token(TokenTypes::Group, "GROUP", 17, 6),
            token(TokenTypes::Having, "HAVING", 23, 6),
            token(TokenTypes::Distinct, "DISTINCT", 30, 6),
            token(TokenTypes::All, "ALL", 39, 6),
            token(TokenTypes::As, "AS", 43, 6),
            token(TokenTypes::Asc, "ASC", 46, 6),
            token(TokenTypes::Desc, "DESC", 50, 6),
            token(TokenTypes::Inner, "INNER", 8, 7),
            token(TokenTypes::Left, "LEFT", 14, 7),
            token(TokenTypes::Right, "RIGHT", 19, 7),
            token(TokenTypes::Full, "FULL", 25, 7),
            token(TokenTypes::Outer, "OUTER", 30, 7),
            token(TokenTypes::Join, "JOIN", 36, 7),
            token(TokenTypes::On, "ON", 41, 7),
            token(TokenTypes::Union, "UNION", 44, 7),
            token(TokenTypes::Limit, "LIMIT", 8, 8),
            token(TokenTypes::Offset, "OFFSET", 14, 8),
            token(TokenTypes::And, "AND", 8, 9),
            token(TokenTypes::Or, "OR", 12, 9),
            token(TokenTypes::In, "IN", 15, 9),
            token(TokenTypes::Exists, "EXISTS", 18, 9),
            token(TokenTypes::If, "IF", 25, 9),
            token(TokenTypes::Case, "CASE", 8, 10),
            token(TokenTypes::When, "WHEN", 13, 10),
            token(TokenTypes::Then, "THEN", 18, 10),
            token(TokenTypes::Else, "ELSE", 23, 10),
            token(TokenTypes::End, "END", 28, 10),
            token(TokenTypes::Equals, "=", 8, 11),
            token(TokenTypes::NotEquals, "!=", 10, 11),
            token(TokenTypes::LessThan, "<", 13, 11),
            token(TokenTypes::LessEquals, "<=", 15, 11),
            token(TokenTypes::GreaterThan, ">", 18, 11),
            token(TokenTypes::GreaterEquals, ">=", 20, 11),
            token(TokenTypes::Count, "COUNT", 8, 12),
            token(TokenTypes::Sum, "SUM", 14, 12),
            token(TokenTypes::Avg, "AVG", 18, 12),
            token(TokenTypes::Min, "MIN", 22, 12),
            token(TokenTypes::Max, "MAX", 26, 12),
            token(TokenTypes::Asterisk, "*", 8, 13),
            token(TokenTypes::SemiColon, ";", 10, 13),
            token(TokenTypes::LeftParen, "(", 12, 13),
            token(TokenTypes::RightParen, ")", 14, 13),
            token(TokenTypes::Comma, ",", 16, 13),
            token(TokenTypes::Dot, ".", 18, 13),
            token(TokenTypes::Plus, "+", 8, 14),
            token(TokenTypes::Minus, "-", 10, 14),
            token(TokenTypes::Divide, "/", 12, 14),
            token(TokenTypes::Modulo, "%", 14, 14),
            token(TokenTypes::String, "string", 8, 15),
            token(TokenTypes::True, "TRUE", 17, 15),
            token(TokenTypes::False, "FALSE", 22, 15),
            token(TokenTypes::Identifier, "fletchers_table", 8, 16),
            token(TokenTypes::EOF, "", 0, 0)
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn tokenizer_parses_int_and_real_literals() {
        let result = tokenize("12 -12 12.12 -12.12 12e-12 -12e12");
        let expected = vec![
            token(TokenTypes::IntLiteral, "12", 0, 1),
            token(TokenTypes::IntLiteral, "-12", 3, 1),
            token(TokenTypes::RealLiteral, "12.12", 7, 1),
            token(TokenTypes::RealLiteral, "-12.12", 13, 1),
            token(TokenTypes::RealLiteral, "12e-12", 20, 1),
            token(TokenTypes::RealLiteral, "-12e12", 27, 1),
            token(TokenTypes::EOF, "", 0, 0),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn tokenizer_parses_hex_literals() {
        let result = tokenize("X\'0A1A3F\' 12 X\'ZZZZZ\'");
        let expected = vec![
            token(TokenTypes::HexLiteral, "0A1A3F" , 0, 1),
            token(TokenTypes::IntLiteral, "12", 10, 1),
            token(TokenTypes::Error, "X\'Z", 13, 1),
            token(TokenTypes::Identifier, "ZZZZ", 16, 1),
            token(TokenTypes::Error, "\'", 20, 1),
            token(TokenTypes::EOF, "", 0, 0),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn tokenizer_parses_string_identifiers() {
        let result = tokenize("'string' \"identifier\"");
        let expected = vec![
            token(TokenTypes::String, "string", 0, 1),
            token(TokenTypes::Identifier, "identifier", 9, 1),
            token(TokenTypes::EOF, "", 0, 0),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn tokenizer_parses_more_keywords() {
        let result = tokenize("ALTER RENAME TO COLUMN ADD DROP");
        let expected = vec![
            token(TokenTypes::Alter, "ALTER", 0, 1),
            token(TokenTypes::Rename, "RENAME", 6, 1),
            token(TokenTypes::To, "TO", 13, 1),
            token(TokenTypes::Column, "COLUMN", 16, 1),
            token(TokenTypes::Add, "ADD", 23, 1),
            token(TokenTypes::Drop, "DROP", 27, 1),
            token(TokenTypes::EOF, "", 0, 0),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn tokenizer_parses_comments() {
        let result = tokenize("SELECT * FROM users WHERE name = 'Fletcher'; -- This is a comment");
        let expected = vec![
            token(TokenTypes::Select, "SELECT", 0, 1),
            token(TokenTypes::Asterisk, "*", 7, 1),
            token(TokenTypes::From, "FROM", 9, 1),
            token(TokenTypes::Identifier, "users", 14, 1),
            token(TokenTypes::Where, "WHERE", 20, 1),
            token(TokenTypes::Identifier, "name", 26, 1),
            token(TokenTypes::Equals, "=", 31, 1),
            token(TokenTypes::String, "Fletcher", 33, 1),
            token(TokenTypes::SemiColon, ";", 43, 1),
            token(TokenTypes::EOF, "", 0, 0),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn tokenizer_parses_comments_with_newlines() {
        let result = tokenize("
        SELECT * FROM users; -- This is a comment
        SELECT * FROM users;");
        let expected = vec![
            token(TokenTypes::Select, "SELECT", 8, 2),
            token(TokenTypes::Asterisk, "*", 15, 2),
            token(TokenTypes::From, "FROM", 17, 2),
            token(TokenTypes::Identifier, "users", 22, 2),
            token(TokenTypes::SemiColon, ";", 27, 2),
            token(TokenTypes::Select, "SELECT", 8, 3),
            token(TokenTypes::Asterisk, "*", 15, 3),
            token(TokenTypes::From, "FROM", 17, 3),
            token(TokenTypes::Identifier, "users", 22, 3),
            token(TokenTypes::SemiColon, ";", 27, 3),
            token(TokenTypes::EOF, "", 0, 0),
        ];
        assert_eq!(expected, result);
    }
}