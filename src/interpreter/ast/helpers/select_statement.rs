use crate::{interpreter::{
    ast::{
        parser::Parser, SelectStatement, SelectableStack, WhereStackElement,
        helpers::{
            common::{tokens_to_identifier_list, get_table_name, expect_token_type},
            order_by_clause::get_order_by, where_stack::get_where_clause, limit_clause::get_limit
        }
    }, 
    tokenizer::token::TokenTypes
}};

pub fn get_statement(parser: &mut Parser) -> Result<SelectStatement, String> {
    parser.advance()?;
    let columns = get_columns(parser)?;
    expect_token_type(parser, TokenTypes::From)?;
    parser.advance()?;
    let table_name = get_table_name(parser)?;
    let where_clause: Option<Vec<WhereStackElement>> = get_where_clause(parser)?;
    let order_by_clause = get_order_by(parser)?;
    let limit_clause = get_limit(parser)?;
    
    return Ok(SelectStatement {
            table_name: table_name,
            columns: columns,
            where_clause: where_clause,
            order_by_clause: order_by_clause,
            limit_clause: limit_clause,
    });
}

fn get_columns(parser: &mut Parser) -> Result<SelectableStack, String> {
    // TODO: this
    Ok(SelectableStack { selectables: vec![] })
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
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::All],
            },
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
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column("id".to_string())],
            },
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
            columns: SelectableStack {
                selectables: vec![
                    SelectableStackElement::Column("id".to_string()),
                    SelectableStackElement::Column("name".to_string()),
                ],
            },
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
            columns: SelectableStack {
                selectables: vec![SelectableStackElement::Column("id".to_string())]
            },
            where_clause: Some(vec![
                WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("id".to_string()),
                    operator: Operator::Equals,
                    r_side: Operand::Value(Value::Integer(1)),
                }),
            ]),
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
        };
        assert_eq!(expected, statement);
    }
}