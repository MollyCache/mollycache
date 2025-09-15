use crate::{interpreter::{
    ast::{
        parser::Parser, SelectStatement, SelectableStack, WhereStackElement, SelectMode,
        helpers::{
            token::{expect_token_type},
            common::{get_table_name, get_selectables},
            order_by_clause::get_order_by, where_clause::get_where_clause, limit_clause::get_limit
        }
    }, 
    tokenizer::token::TokenTypes,
}};

pub fn get_statement(parser: &mut Parser) -> Result<SelectStatement, String> {
    parser.advance()?;
    let mode = match parser.current_token()?.token_type {
        TokenTypes::Distinct => {
            parser.advance()?;
            SelectMode::Distinct
        }
        _ => SelectMode::All
    };
    let (columns, column_names) = get_columns_and_names(parser)?;
    expect_token_type(parser, TokenTypes::From)?; // TODO: this is not true, you can do SELECT 1;
    parser.advance()?;
    let table_name = get_table_name(parser)?;
    let where_clause: Option<Vec<WhereStackElement>> = get_where_clause(parser)?;
    let order_by_clause = get_order_by(parser)?;
    let limit_clause = get_limit(parser)?;
    
    return Ok(SelectStatement {
            table_name: table_name,
            mode: mode,
            columns: columns,
            column_names: column_names,
            where_clause: where_clause,
            order_by_clause: order_by_clause,
            limit_clause: limit_clause,
    });
}

fn get_columns_and_names(parser: &mut Parser) -> Result<(SelectableStack, Vec<String>), String> {
    let mut column_names: Vec<String> = vec![];
    Ok((get_selectables(parser, true, &mut None, &mut Some(&mut column_names))?, column_names))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::Operator;
    use crate::db::table::Value;
    use crate::interpreter::ast::OrderByClause;
    use crate::interpreter::ast::OrderByDirection;
    use crate::interpreter::ast::LimitClause;
    use crate::interpreter::ast::WhereStackElement;
    use crate::interpreter::ast::WhereCondition;
    use crate::interpreter::ast::test_utils::token;
    use crate::interpreter::ast::Operand;
    use crate::interpreter::ast::SelectableStackElement;

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
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement, SelectStatement {
            table_name: "users".to_string(),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All],
            },
            column_names: vec!["*".to_string()],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        });
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
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement, SelectStatement {
            table_name: "guests".to_string(),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column("id".to_string())],
            },
            column_names: vec!["id".to_string()],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        });
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
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement, SelectStatement {
            table_name: "users".to_string(),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![
                    SelectableStackElement::Column("id".to_string()),
                    SelectableStackElement::Column("name".to_string()),
                ],
            },
            column_names: vec!["id".to_string(), "name".to_string()],
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        });
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
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SelectStatement {
            table_name: "guests".to_string(),
            mode: SelectMode::All,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column("id".to_string())]
            },
            column_names: vec!["id".to_string()],
            where_clause: Some(vec![
                WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("id".to_string()),
                    operator: Operator::Equals,
                    r_side: Operand::Value(Value::Integer(1)),
                }),
            ]),
            order_by_clause: Some(OrderByClause {
                columns: SelectableStack {
                    selectables: vec![
                        SelectableStackElement::Column("id".to_string()),
                        SelectableStackElement::Column("name".to_string()),
                        SelectableStackElement::Column("age".to_string()),
                    ]
                },
                column_names: vec!["id".to_string(), "name".to_string(), "age".to_string()],
                directions: vec![OrderByDirection::Asc, OrderByDirection::Desc, OrderByDirection::Asc],
            }),
            limit_clause: Some(LimitClause {
                limit: 10,
                offset: Some(5),
            }),
        };
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_distinct_mode_is_generated_correctly() {
        // SELECT DISTINCT id FROM guests;
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Distinct, "DISTINCT"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "guests"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = get_statement(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        assert_eq!(statement, SelectStatement {
            table_name: "guests".to_string(),
            column_names: vec!["id".to_string()],
            mode: SelectMode::Distinct,
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column("id".to_string())],
            },
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        });
    }
}