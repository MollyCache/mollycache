use crate::interpreter::ast::{
    parser::Parser, SqlStatement, UpdateStatement, ColumnValue, 
    helpers::common::{expect_token_type, token_to_value, get_table_name},
    helpers::{order_by_clause::get_order_by, limit_clause::get_limit},
};
use crate::interpreter::ast::helpers::where_stack::get_where_clause;
use crate::interpreter::tokenizer::token::TokenTypes;

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    
    let table_name = get_table_name(parser)?;
    // Ensure Set
    parser.advance()?;
    expect_token_type(parser, TokenTypes::Set)?;
    let update_values = get_update_values(parser)?;
    let where_clause = get_where_clause(parser)?;
    let order_by_clause = get_order_by(parser)?;
    let limit_clause = get_limit(parser)?;

    // Ensure SemiColon
    expect_token_type(parser, TokenTypes::SemiColon)?;
    return Ok(SqlStatement::UpdateStatement(UpdateStatement {
        table_name: table_name,
        update_values: update_values,
        where_clause: where_clause,
        order_by_clause: order_by_clause,
        limit_clause: limit_clause,
    }));
}

// We do not currently support conditional updates such as "UPDATE table SET column = column * 1.1;"
fn get_update_values(parser: &mut Parser) -> Result<Vec<ColumnValue>, String> {
    parser.advance()?;
    let mut update_values = vec![];
    loop {
        let token = parser.current_token()?;
        expect_token_type(parser, TokenTypes::Identifier)?;
        let column = token.value.to_string();
        parser.advance()?;

        expect_token_type(parser, TokenTypes::Equals)?;
        parser.advance()?;

        let value = token_to_value(parser)?;
        update_values.push(ColumnValue {
            column: column,
            value: value,
        });
        
        parser.advance()?;
        let token = parser.current_token()?;

        match token.token_type {
            TokenTypes::Comma => {
                parser.advance()?;
            },
            _ => break,
        }
    }
    return Ok(update_values);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::Value;
    use crate::interpreter::ast::Operator;
    use crate::interpreter::ast::WhereStackElement;
    use crate::interpreter::ast::WhereCondition;
    use crate::interpreter::ast::test_utils::token;
    use crate::interpreter::ast::Operand;
    use crate::interpreter::ast::OrderByClause;
    use crate::interpreter::ast::OrderByDirection;
    use crate::interpreter::ast::LimitClause;
    
    #[test]
    fn update_statement_with_all_tokens_is_generated_correctly() {
        // UPDATE users SET column = value;
        let tokens = vec![
            token(TokenTypes::Update, "UPDATE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Set, "SET"),
            token(TokenTypes::Identifier, "column"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::String, "value"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::UpdateStatement(UpdateStatement {
            table_name: "users".to_string(),
            update_values: vec![ColumnValue {
                column: "column".to_string(),
                value: Value::Text("value".to_string()),
            }],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        });
        assert_eq!(statement, expected);
    }

    #[test]
    fn update_statement_with_where_clause_is_generated_correctly() {
        // UPDATE users SET column = 1 WHERE id > 2;
        let tokens = vec![
            token(TokenTypes::Update, "UPDATE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Set, "SET"),
            token(TokenTypes::Identifier, "column"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::GreaterThan, ">"),
            token(TokenTypes::IntLiteral, "2"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::UpdateStatement(UpdateStatement {
            table_name: "users".to_string(),
            update_values: vec![ColumnValue {
                column: "column".to_string(),
                value: Value::Integer(1),
            }],
            where_clause: Some(vec![
                WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("id".to_string()),
                    operator: Operator::GreaterThan,
                    r_side: Operand::Value(Value::Integer(2)),
                }),
            ]),
            order_by_clause: None,
            limit_clause: None,
        });
        assert_eq!(statement, expected);
    }

    #[test]
    fn update_statement_with_multiple_update_values_is_generated_correctly() {
        // UPDATE users SET column = 1, active = "False" WHERE id = 3;
        let tokens = vec![
            token(TokenTypes::Update, "UPDATE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Set, "SET"),
            token(TokenTypes::Identifier, "column"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "active"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::String, "False"),
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "3"),
            token(TokenTypes::SemiColon, ";"),

        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::UpdateStatement(UpdateStatement {
            table_name: "users".to_string(),
            update_values: vec![
                ColumnValue {
                    column: "column".to_string(),
                    value: Value::Integer(1),
                },
                ColumnValue {
                    column: "active".to_string(),
                    value: Value::Text("False".to_string()),
                },
            ],
            where_clause: Some(vec![
                WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("id".to_string()),
                    operator: Operator::Equals,
                    r_side: Operand::Value(Value::Integer(3)),
                }),
            ]),
            order_by_clause: None,
            limit_clause: None,
        });
        assert_eq!(statement, expected);
    }

    #[test]
    fn update_statement_with_all_clauses_is_generated_correctly() {
        // UPDATE users SET column = 1 WHERE id = 1 ORDER BY id ASC LIMIT 10 OFFSET 5;
        let tokens = vec![
            token(TokenTypes::Update, "UPDATE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Set, "SET"),
            token(TokenTypes::Identifier, "column"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Order, "ORDER"),
            token(TokenTypes::By, "BY"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Asc, "ASC"),
            token(TokenTypes::Limit, "LIMIT"),
            token(TokenTypes::IntLiteral, "10"),
            token(TokenTypes::Offset, "OFFSET"),
            token(TokenTypes::IntLiteral, "5"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::UpdateStatement(UpdateStatement {
            table_name: "users".to_string(),
            update_values: vec![ColumnValue {
                column: "column".to_string(),
                value: Value::Integer(1),
            }],
            where_clause: Some(vec![
                WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("id".to_string()),
                    operator: Operator::Equals,
                    r_side: Operand::Value(Value::Integer(1)),
                }),
            ]),
            order_by_clause: Some(vec![OrderByClause {
                column: "id".to_string(),
                direction: OrderByDirection::Asc,
            }]),
            limit_clause: Some(LimitClause {
                limit: Value::Integer(10),
                offset: Some(Value::Integer(5)),
            }),
        });
        assert_eq!(expected, statement);
    }
}