use crate::{cli::{
    ast::{
        common::{expect_token_type, tokens_to_identifier_list, get_table_name}, 
        parser::Parser, SelectStatement, SelectStatementColumns, SqlStatement, WhereClause,
        helpers::{order_by_clause::get_order_by, where_clause::get_where_clause, limit_clause::get_limit}
    }, 
    tokenizer::token::TokenTypes
}};

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    let columns = get_columns(parser)?;
    let table_name = get_table_name(parser)?;
    parser.advance()?;
    let where_clause: Option<WhereClause> = get_where_clause(parser)?;
    let order_by_clause = get_order_by(parser)?;
    let limit_clause = get_limit(parser)?;
    
    // Ensure SemiColon
    expect_token_type(parser, TokenTypes::SemiColon)?;
    return Ok(SqlStatement::Select(SelectStatement {
        table_name: table_name,
        columns: columns,
        where_clause: where_clause,
        order_by_clause: order_by_clause,
        limit_clause: limit_clause,
    }));
}

fn get_columns(parser: &mut Parser) -> Result<SelectStatementColumns, String> {
    let token = parser.current_token()?;
    
    if token.token_type == TokenTypes::Asterisk {
        parser.advance()?;
        Ok(SelectStatementColumns::All)
    } else {
        let columns = tokens_to_identifier_list(parser)?;
        Ok(SelectStatementColumns::Specific(columns))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ast::Operator;
    use crate::db::table::Value;
    use crate::cli::ast::OrderByClause;
    use crate::cli::ast::OrderByDirection;
    use crate::cli::ast::LimitClause;
    use crate::cli::ast::test_utils::token;

    #[test]
    fn select_statement_with_all_tokens_is_generated_correctly() {
        // SELECT * FROM users;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Asterisk, "*"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement, SqlStatement::Select(SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::All,
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        }));
    }

    #[test]
    fn select_statement_with_a_single_column_is_generated_correctly() {
        // SELECT id FROM guests;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement, SqlStatement::Select(SelectStatement {
            table_name: "guests".to_string(),
            columns: SelectStatementColumns::Specific(vec![
                "id".to_string(),
            ]),
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        }));
    }

    #[test]
    fn select_statement_with_multiple_columns_is_generated_correctly() {
        // SELECT id, name FROM users;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement, SqlStatement::Select(SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::Specific(vec![
                "id".to_string(),
                "name".to_string(),
            ]),
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        }));
    }

    #[test]
    fn select_statement_with_all_clauses_is_generated_correctly() {
        // SELECT id FROM guests WHERE id = 1 ORDER BY id ASC, name DESC, age ASC LIMIT 10 OFFSET 5;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Order, "ORDER"),
            token(TokenTypes::By, "BY"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Asc, "ASC"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "name"),
            token(TokenTypes::Desc, "DESC"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::Identifier, "age"),
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
        assert_eq!(statement, SqlStatement::Select(SelectStatement {
            table_name: "guests".to_string(),
            columns: SelectStatementColumns::Specific(vec![
                "id".to_string(),
            ]),
            where_clause: Some(WhereClause {
                column: "id".to_string(),
                operator: Operator::Equals,
                value: Value::Integer(1),
            }),
            order_by_clause: Some(vec![
                OrderByClause {
                    column: "id".to_string(),
                    direction: OrderByDirection::Asc,
                },
                OrderByClause {
                    column: "name".to_string(),
                    direction: OrderByDirection::Desc,
                },
                OrderByClause {
                    column: "age".to_string(),
                    direction: OrderByDirection::Asc,
                }
            ]),
            limit_clause: Some(LimitClause {
                limit: Value::Integer(10),
                offset: Some(Value::Integer(5)),
            }),
        }));
    }

    #[test]
    fn select_statement_with_limit_clause_no_offset_is_generated_correctly() {
        // SELECT id FROM guests WHERE id > 1 LIMIT 10;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::GreaterThan, ">"),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Limit, "LIMIT"),
            token(TokenTypes::IntLiteral, "10"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement, SqlStatement::Select(SelectStatement {
            table_name: "guests".to_string(),
            columns: SelectStatementColumns::Specific(vec![
                "id".to_string(),
            ]),
            where_clause: Some(WhereClause {
                column: "id".to_string(),
                operator: Operator::GreaterThan,
                value: Value::Integer(1),
            }),
            order_by_clause: None,
            limit_clause: Some(LimitClause {
                limit: Value::Integer(10),
                offset: None,
            }),
        }));
    }

    #[test]
    fn select_statement_with_limit_clause_with_negative_offset_is_generated_correctly() {
        // SELECT id FROM guests LIMIT 10 OFFSET -5;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::Limit, "LIMIT"),
            token(TokenTypes::IntLiteral, "10"),
            token(TokenTypes::Offset, "OFFSET"),
            token(TokenTypes::IntLiteral, "-5"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Error at line 1, column 0: Unexpected value: -5");
    }
}