use crate::interpreter::ast::helpers::where_clause::get_where_clause;
use crate::interpreter::ast::{
    ColumnValue, SqlStatement, UpdateStatement,
    helpers::common::get_table_name,
    helpers::token::{expect_token_type, token_to_value},
    helpers::{limit_clause::get_limit, order_by_clause::get_order_by},
    parser::Parser,
};
use crate::interpreter::tokenizer::token::TokenTypes;

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    let table_name = get_table_name(parser)?;
    // Ensure Set
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
            }
            _ => break,
        }
    }
    return Ok(update_values);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::table::core::value::Value;
    use crate::interpreter::ast::LimitClause;
    use crate::interpreter::ast::Operator;
    use crate::interpreter::ast::OrderByClause;
    use crate::interpreter::ast::OrderByDirection;
    use crate::interpreter::ast::SelectableColumn;
    use crate::interpreter::ast::SelectableStackElement;
    use crate::interpreter::ast::test_utils::token;

    #[test]
    fn update_statement_with_all_tokens_is_generated_correctly() {
        // UPDATE users SET column = value;
        let tokens = vec![
            token(TokenTypes::Update, "UPDATE"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Set, "SET"),
            token(TokenTypes::Identifier, "column"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::StringLiteral, "value"),
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
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("id".to_string()),
                    SelectableStackElement::Value(Value::Integer(2)),
                    SelectableStackElement::Operator(Operator::GreaterThan),
                ],
                column_name: "id > 2".to_string(),
            }),
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
            token(TokenTypes::StringLiteral, "False"),
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
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("id".to_string()),
                    SelectableStackElement::Value(Value::Integer(3)),
                    SelectableStackElement::Operator(Operator::Equals),
                ],
                column_name: "id = 3".to_string(),
            }),
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
            where_clause: Some(SelectableColumn {
                selectables: vec![
                    SelectableStackElement::Column("id".to_string()),
                    SelectableStackElement::Value(Value::Integer(1)),
                    SelectableStackElement::Operator(Operator::Equals),
                ],
                column_name: "id = 2".to_string(),
            }),
            order_by_clause: Some(OrderByClause {
                columns: vec![SelectableColumn {
                    selectables: vec![SelectableStackElement::Column("id".to_string())],
                    column_name: "id".to_string(),
                }],
                directions: vec![OrderByDirection::Asc],
            }),
            limit_clause: Some(LimitClause {
                limit: 10,
                offset: Some(5),
            }),
        });
        assert_eq!(expected, statement);
    }
}
