use crate::cli::ast::{parser::Parser, WhereClause, Operator, helpers::common::{expect_token_type, token_to_value}};
use crate::cli::tokenizer::token::TokenTypes;

pub fn get_where_clause(parser: &mut Parser) -> Result<Option<WhereClause>, String> {
    if expect_token_type(parser, TokenTypes::Where).is_err() {
        return Ok(None);
    }
    parser.advance()?;

    let token = parser.current_token()?;
    expect_token_type(parser, TokenTypes::Identifier)?;
    let column = token.value.to_string();
    parser.advance()?;

    let token = parser.current_token()?;
    let operator  = match token.token_type {
        TokenTypes::Equals => Operator::Equals,
        TokenTypes::NotEquals => Operator::NotEquals,
        TokenTypes::LessThan => Operator::LessThan,
        TokenTypes::LessEquals => Operator::LessEquals,
        TokenTypes::GreaterThan => Operator::GreaterThan,
        TokenTypes::GreaterEquals => Operator::GreaterEquals,
        _ => return Err(parser.format_error()),
    };
    parser.advance()?;

    let value = token_to_value(parser)?;
    parser.advance()?;

    return Ok(Some(WhereClause {
        column: column,
        operator: operator,
        value: value,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::tokenizer::scanner::Token;
    use crate::db::table::Value;

    fn token(tt: TokenTypes, val: &'static str) -> Token<'static> {
        Token {
            token_type: tt,
            value: val,
            col_num: 0,
            line_num: 1,
        }
    }

    #[test]
    fn where_clause_with_all_tokens_is_generated_correctly() {
        // WHERE id = 1 LIMIT...
        let tokens = vec![
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Limit, "LIMIT"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_where_clause(&mut parser);
        assert!(result.is_ok());
        let where_clause = result.unwrap();
        let expected = Some(WhereClause {
            column: "id".to_string(),
            operator: Operator::Equals,
            value: Value::Integer(1),
        });
        assert_eq!(expected, where_clause);
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::Limit);
    }

    #[test]
    fn not_where_clause_returns_none() {
        // SELECT * ...;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Asterisk, "*"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_where_clause(&mut parser);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::Select);
    }
}
