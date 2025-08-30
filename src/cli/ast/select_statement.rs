use crate::cli::{ast::{parser::Parser, SqlStatement, SelectStatement, SelectStatementColumns, Operator, common::token_to_value, common::tokens_to_identifier_list, WhereClause, OrderByClause, OrderByDirection, LimitClause}, tokenizer::token::TokenTypes};

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance();
    let columns = get_columns(parser)?;
    parser.advance(); // parser comes out of get_columns() on the From token this advances to table name
    let table_name = get_table_name(parser)?;
    let where_clause = get_where_clause(parser)?;
    let order_by_clause = get_order_by(parser)?;
    let limit_clause = get_limit(parser)?;
    
    // Ensure SemiColon
    let token = parser.current_token().ok_or_else(|| parser.format_error())?;
    if token.token_type != TokenTypes::SemiColon {
        return Err(parser.format_error());
    }
    parser.advance(); // Move past the semicolon
    return Ok(SqlStatement::Select(SelectStatement {
        table_name: table_name,
        columns: columns,
        where_clause: where_clause,
        order_by_clause: order_by_clause,
        limit_clause: limit_clause,
    }));
}

fn get_columns(parser: &mut Parser) -> Result<SelectStatementColumns, String> {
    match parser.current_token() {
        Some(token) => {
            if token.token_type == TokenTypes::Asterisk {
                parser.advance();
                return Ok(SelectStatementColumns::All);
            }
            else {
                let columns = tokens_to_identifier_list(parser)?;
                return Ok(SelectStatementColumns::Specific(columns));
            }
        }
        None => return Err(parser.format_error()),
    }
}

fn get_table_name(parser: &mut Parser) -> Result<String, String> {
    match parser.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::Identifier {
                return Err(parser.format_error());
            }
            let result = token.value.to_string();
            parser.advance();
            return Ok(result);
        },
        None => return Err(parser.format_error()),
    };
}

fn get_where_clause(parser: &mut Parser) -> Result<Option<WhereClause>, String> {
    let token = parser.current_token().ok_or_else(|| parser.format_error())?;
    if token.token_type != TokenTypes::Where {
        return Ok(None);
    }
    parser.advance();

    let column = match parser.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::Identifier {
                return Err(parser.format_error());
            }
            token.value.to_string()
        }
        None => return Err(parser.format_error()),
    };
    parser.advance();
    let operator = match parser.current_token() {
        Some(token) => {
            match token.token_type {
                TokenTypes::Equals => Operator::Equals,
                TokenTypes::NotEquals => Operator::NotEquals,
                TokenTypes::LessThan => Operator::LessThan,
                TokenTypes::LessEquals => Operator::LessEquals,
                TokenTypes::GreaterThan => Operator::GreaterThan,
                TokenTypes::GreaterEquals => Operator::GreaterEquals,
                _ => return Err(parser.format_error()),
            }
        }
        None => return Err(parser.format_error()),
    };
    parser.advance();
    let value = token_to_value(parser)?;

    parser.advance();
    return Ok(Some(WhereClause {
        column: column,
        operator: operator,
        value: value,
    }));
}

fn get_order_by(parser: &mut Parser) -> Result<Option<Vec<OrderByClause>>, String> {
    let token = parser.current_token().ok_or_else(|| parser.format_error())?;
    if token.token_type != TokenTypes::Order {
        return Ok(None);
    }
    parser.advance();

    let token = parser.current_token().ok_or_else(|| parser.format_error())?;
    if token.token_type != TokenTypes::By {
        return Err(parser.format_error());
    }
    parser.advance();

    let mut order_by_clauses = vec![];
    loop {
        let column = match parser.current_token() {
            Some(token) => {
                if token.token_type != TokenTypes::Identifier {
                    return Err(parser.format_error());
                }
                token.value.to_string()
            },
            None => return Err(parser.format_error()),
        };

        parser.advance();
        let direction = match parser.current_token() {
            Some(token) => {
                match token.token_type {
                    TokenTypes::Asc => {
                        parser.advance();
                        OrderByDirection::Asc
                    },
                    TokenTypes::Desc => {
                        parser.advance();
                        OrderByDirection::Desc
                    },
                    _ => OrderByDirection::Asc,
                }
            },
            None => return Err(parser.format_error()),
        };

        order_by_clauses.push(OrderByClause {
            column: column,
            direction: direction,
        });

        let token = parser.current_token().ok_or_else(|| parser.format_error())?;
        if token.token_type != TokenTypes::Comma {
            break;
        }
        parser.advance();
    }
    return Ok(Some(order_by_clauses));
}

fn get_limit(parser: &mut Parser) -> Result<Option<LimitClause>, String> {
    let token = parser.current_token().ok_or_else(|| parser.format_error())?;
    if token.token_type != TokenTypes::Limit {
        return Ok(None);
    }
    parser.advance();

    let limit = match parser.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::IntLiteral {
                return Err(parser.format_error());
            }
            token_to_value(parser)?
        },
        None => return Err(parser.format_error()),
    };
    parser.advance();

    let token = parser.current_token().ok_or_else(|| parser.format_error())?;
    if token.token_type != TokenTypes::Offset {
        return Ok(Some(LimitClause {
            limit: limit,
            offset: None,
        }));
    }
    parser.advance();

    let offset = match parser.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::IntLiteral {
                return Err(parser.format_error());
            }
            token_to_value(parser)?
        },
        None => return Err(parser.format_error()),
    };   

    parser.advance();
    return Ok(Some(LimitClause {
        limit: limit,
        offset: Some(offset),
    }));
    
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::tokenizer::scanner::Token;
    use crate::cli::table::Value;

    fn token(tt: TokenTypes, val: &'static str) -> Token<'static> {
        Token {
            token_type: tt,
            value: val,
            col_num: 0,
            line_num: 1,
        }
    }

    #[test]
    fn select_statement_with_all_tokens_is_generated_correctly() {
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
}