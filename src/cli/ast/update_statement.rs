use crate::cli::ast::{
    parser::Parser, SqlStatement, UpdateStatement, ColumnValue, 
    helpers::common::{expect_token_type, token_to_value, get_table_name}
};
use crate::cli::ast::helpers::where_clause::get_where_clause;
use crate::cli::tokenizer::token::TokenTypes;

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    
    let table_name = get_table_name(parser)?;
    // Ensure Set
    parser.advance()?;
    expect_token_type(parser, TokenTypes::Set)?;
    let update_values = get_update_values(parser)?;
    let where_clause = get_where_clause(parser)?;

    // Ensure SemiColon
    expect_token_type(parser, TokenTypes::SemiColon)?;
    return Ok(SqlStatement::UpdateStatement(UpdateStatement {
        table_name: table_name,
        update_values: update_values,
        where_clause: where_clause,
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
    use crate::cli::ast::Operator;
    use crate::cli::ast::WhereTreeNode;
    use crate::cli::ast::WhereTreeElement;
    use crate::cli::ast::WhereTreeEdge;
    use crate::cli::ast::test_utils::token;
    
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
            where_clause: Some(WhereTreeNode {
                left: Box::new(Some(WhereTreeElement::Edge(WhereTreeEdge {
                column: "id".to_string(),
                    operator: Operator::GreaterThan,
                    value: Value::Integer(2),
                }))),
                right: Box::new(None),
                operator: None,
                negation: false,
            }),
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
            where_clause: Some(WhereTreeNode {
                left: Box::new(Some(WhereTreeElement::Edge(WhereTreeEdge {
                    column: "id".to_string(),
                    operator: Operator::Equals,
                    value: Value::Integer(3),
                }))),
                right: Box::new(None),
                operator: None,
                negation: false,
            }),
            });
        assert_eq!(statement, expected);
    }
}