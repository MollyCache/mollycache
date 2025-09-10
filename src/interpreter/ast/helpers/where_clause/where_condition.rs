use crate::interpreter::ast::{parser::Parser, WhereCondition, Operand, Operator};
use crate::interpreter::tokenizer::token::TokenTypes;
use crate::interpreter::ast::helpers::token::{expect_token_type, token_to_value, tokens_to_value_list};


pub fn get_condition(parser: &mut Parser) -> Result<WhereCondition, String> {
    let l_side = get_operand(parser)?;
    parser.advance()?;

    let token = parser.current_token()?;
    let mut operator = match token.token_type {
        TokenTypes::Equals => Operator::Equals,
        TokenTypes::NotEquals => Operator::NotEquals,
        TokenTypes::LessThan => Operator::LessThan,
        TokenTypes::LessEquals => Operator::LessEquals,
        TokenTypes::GreaterThan => Operator::GreaterThan,
        TokenTypes::GreaterEquals => Operator::GreaterEquals,
        TokenTypes::In => Operator::In,
        TokenTypes::Not => Operator::NotIn,
        TokenTypes::Is => Operator::Is,
        _ => return Err(parser.format_error()),
    };
    parser.advance()?;

    if operator == Operator::NotIn || operator == Operator::In {
        if operator == Operator::NotIn {
            expect_token_type(parser, TokenTypes::In)?;
            parser.advance()?;
        }
        let r_side = get_operand(parser)?;
        parser.advance()?;

        return Ok(WhereCondition {
            l_side: l_side,
            operator: operator,
            r_side: r_side,
        });
    }

    if operator == Operator::Is && parser.current_token()?.token_type == TokenTypes::Not {
        operator = Operator::IsNot;
        parser.advance()?;
    }

    let r_side = get_operand(parser)?;
    parser.advance()?;

    return Ok(WhereCondition {
        l_side: l_side,
        operator,
        r_side: r_side,
    });
}

pub fn get_operand(parser: &mut Parser) -> Result<Operand, String> {
    let token = parser.current_token()?;
    match token.token_type {
        TokenTypes::Identifier => Ok(Operand::Identifier(token.value.to_string())),
        TokenTypes::IntLiteral => Ok(Operand::Value(token_to_value(parser)?)),
        TokenTypes::RealLiteral => Ok(Operand::Value(token_to_value(parser)?)),
        TokenTypes::String => Ok(Operand::Value(token_to_value(parser)?)),
        TokenTypes::Blob => Ok(Operand::Value(token_to_value(parser)?)),
        TokenTypes::Null => Ok(Operand::Value(token_to_value(parser)?)),
        TokenTypes::LeftParen => {
            parser.advance()?;

            let values = tokens_to_value_list(parser)?;
            expect_token_type(parser, TokenTypes::RightParen)?;

            Ok(Operand::ValueList(values))
        },
        _ => return Err(parser.format_error()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::{Operator, WhereCondition, Operand};
    use crate::interpreter::ast::test_utils::token;
    use crate::db::table::Value;

    fn assert_where_condition(result: Result<WhereCondition, String>, expected: WhereCondition, parser: &mut Parser) {
        assert!(result.is_ok());
        let where_clause = result.unwrap();
        assert_eq!(expected, where_clause);
        assert_eq!(parser.current_token().unwrap().token_type, TokenTypes::SemiColon);
    }

    #[test]
    fn parses_simple_equality_condition() {
        // id > 1;...
        let tokens = vec![
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::GreaterThan, ">"),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_condition(&mut parser);
        let expected = WhereCondition {
            l_side: Operand::Identifier("id".to_string()),
            operator: Operator::GreaterThan,
            r_side: Operand::Value(Value::Integer(1)),
        };
        assert_where_condition(result, expected, &mut parser);
    }

    #[test]
    fn parses_in_operator_with_value_list() {
        // id IN (1, 2, 3);...
        let tokens = vec![
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::In, "IN"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::IntLiteral, "2"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::IntLiteral, "3"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_condition(&mut parser);
        let expected = WhereCondition {
            l_side: Operand::Identifier("id".to_string()),
            operator: Operator::In,
            r_side: Operand::ValueList(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)]),
        };
        assert_where_condition(result, expected, &mut parser);
    }

    #[test]
    fn parses_not_in_operator_with_value_list() {
        // id NOT IN (1);...
        let tokens = vec![
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Not, "NOT"),
            token(TokenTypes::In, "IN"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_condition(&mut parser);
        let expected = WhereCondition {
            l_side: Operand::Identifier("id".to_string()),
            operator: Operator::NotIn,
            r_side: Operand::ValueList(vec![Value::Integer(1)]),
        };
        assert_where_condition(result, expected, &mut parser);
    }

    #[test]
    fn parses_column_to_column_comparison() {
        // id = name;...
        let tokens = vec![
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_condition(&mut parser);
        let expected = WhereCondition {
            l_side: Operand::Identifier("id".to_string()),
            operator: Operator::Equals,
            r_side: Operand::Identifier("name".to_string()),
        };
        assert_where_condition(result, expected, &mut parser);
    }

    #[test]
    fn where_stack_handles_reversed_condition() {
        // "fletcher" < id;...
        let tokens = vec![
            token(TokenTypes::String, "fletcher"),
            token(TokenTypes::LessThan, "<"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_condition(&mut parser);
        let expected = WhereCondition {
            l_side: Operand::Value(Value::Text("fletcher".to_string())),
            operator: Operator::LessThan,
            r_side: Operand::Identifier("id".to_string()),
        };
        assert_where_condition(result, expected, &mut parser);
    }

    #[test]
    fn where_stack_handles_conditions_with_two_literals() {
        // 1.1 != 2.2;...
        let tokens = vec![
            token(TokenTypes::RealLiteral, "1.1"),
            token(TokenTypes::NotEquals, "!="),
            token(TokenTypes::RealLiteral, "2.2"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_condition(&mut parser);
        let expected = WhereCondition {
            l_side: Operand::Value(Value::Real(1.1)),
            operator: Operator::NotEquals,
            r_side: Operand::Value(Value::Real(2.2)),
        };
        assert_where_condition(result, expected, &mut parser);
    }

    #[test]
    fn where_condition_handles_invalid_not_in_statement() {
        // X'00' NOT (1);...
        let tokens = vec![
            token(TokenTypes::Blob, "00"),
            token(TokenTypes::Not, "NOT"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_condition(&mut parser);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Error at line 1, column 0: Unexpected value: (");
    }

    #[test]
    fn where_condition_handles_is_operator() {
        // id IS NULL;...
        let tokens = vec![
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Is, "IS"),
            token(TokenTypes::Null, "NULL"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_condition(&mut parser);
        let expected = WhereCondition {
            l_side: Operand::Identifier("id".to_string()),
            operator: Operator::Is,
            r_side: Operand::Value(Value::Null),
        };
        assert_where_condition(result, expected, &mut parser);
    }

    #[test]
    fn where_condition_handles_is_not_operator() {
        // id IS NOT name;...
        let tokens = vec![
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Is, "IS"),
            token(TokenTypes::Not, "NOT"),
            token(TokenTypes::Null, "NULL"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_condition(&mut parser);
        let expected = WhereCondition {
            l_side: Operand::Identifier("id".to_string()),
            operator: Operator::IsNot,
            r_side: Operand::Value(Value::Null),
        };
        assert_where_condition(result, expected, &mut parser);
    }
}