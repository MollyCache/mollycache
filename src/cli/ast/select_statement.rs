use crate::cli::{ast::{interpreter::Interpreter, SqlStatement, SelectStatement, SelectStatementColumns, Operator, common::token_to_value, common::tokens_to_identifier_list, WhereClause, OrderByClause, OrderByDirection, LimitClause}, tokenizer::token::TokenTypes};

pub fn build(interpreter: &mut Interpreter) -> Result<SqlStatement, String> {
    interpreter.advance();
    let columns = get_columns(interpreter)?;
    interpreter.advance(); // interpreter comes out of get_columns() on the From token this advances to table name
    let table_name = get_table_name(interpreter)?;
    let where_clause = get_where_clause(interpreter)?;
    let order_by_clause = get_order_by(interpreter)?;
    let limit_clause = get_limit(interpreter)?;
    
    // Ensure SemiColon
    let token = interpreter.current_token().ok_or_else(|| interpreter.format_error())?;
    if token.token_type != TokenTypes::SemiColon {
        return Err(interpreter.format_error());
    }

    return Ok(SqlStatement::Select(SelectStatement {
        table_name: table_name,
        columns: columns,
        where_clause: where_clause,
        order_by_clause: order_by_clause,
        limit_clause: limit_clause,
    }));
}

fn get_columns(interpreter: &mut Interpreter) -> Result<SelectStatementColumns, String> {
    match interpreter.current_token() {
        Some(token) => {
            if token.token_type == TokenTypes::Asterisk {
                interpreter.advance();
                return Ok(SelectStatementColumns::All);
            }
            else {
                let columns = tokens_to_identifier_list(interpreter)?;
                return Ok(SelectStatementColumns::Specific(columns));
            }
        }
        None => return Err(interpreter.format_error()),
    }
}

fn get_table_name(interpreter: &mut Interpreter) -> Result<String, String> {
    match interpreter.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::Identifier {
                return Err(interpreter.format_error());
            }
            let result = token.value.to_string();
            interpreter.advance();
            return Ok(result);
        },
        None => return Err(interpreter.format_error()),
    };
}

fn get_where_clause(interpreter: &mut Interpreter) -> Result<Option<WhereClause>, String> {
    let token = interpreter.current_token().ok_or_else(|| interpreter.format_error())?;
    if token.token_type != TokenTypes::Where {
        return Ok(None);
    }
    interpreter.advance();

    let column = match interpreter.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::Identifier {
                return Err(interpreter.format_error());
            }
            token.value.to_string()
        }
        None => return Err(interpreter.format_error()),
    };
    interpreter.advance();
    let operator = match interpreter.current_token() {
        Some(token) => {
            match token.token_type {
                TokenTypes::Equals => Operator::Equals,
                TokenTypes::NotEquals => Operator::NotEquals,
                TokenTypes::LessThan => Operator::LessThan,
                TokenTypes::LessEquals => Operator::LessEquals,
                TokenTypes::GreaterThan => Operator::GreaterThan,
                TokenTypes::GreaterEquals => Operator::GreaterEquals,
                _ => return Err(interpreter.format_error()),
            }
        }
        None => return Err(interpreter.format_error()),
    };
    interpreter.advance();
    let value = token_to_value(interpreter)?;

    interpreter.advance();
    return Ok(Some(WhereClause {
        column: column,
        operator: operator,
        value: value,
    }));
}

fn get_order_by(interpreter: &mut Interpreter) -> Result<Option<Vec<OrderByClause>>, String> {
    let token = interpreter.current_token().ok_or_else(|| interpreter.format_error())?;
    if token.token_type != TokenTypes::Order {
        return Ok(None);
    }
    interpreter.advance();

    let token = interpreter.current_token().ok_or_else(|| interpreter.format_error())?;
    if token.token_type != TokenTypes::By {
        return Err(interpreter.format_error());
    }
    interpreter.advance();

    let mut order_by_clauses = vec![];
    loop {
        let column = match interpreter.current_token() {
            Some(token) => {
                if token.token_type != TokenTypes::Identifier {
                    return Err(interpreter.format_error());
                }
                token.value.to_string()
            },
            None => return Err(interpreter.format_error()),
        };

        interpreter.advance();
        let direction = match interpreter.current_token() {
            Some(token) => {
                match token.token_type {
                    TokenTypes::Asc => {
                        interpreter.advance();
                        OrderByDirection::Asc
                    },
                    TokenTypes::Desc => {
                        interpreter.advance();
                        OrderByDirection::Desc
                    },
                    _ => OrderByDirection::Asc,
                }
            },
            None => return Err(interpreter.format_error()),
        };

        order_by_clauses.push(OrderByClause {
            column: column,
            direction: direction,
        });

        let token = interpreter.current_token().ok_or_else(|| interpreter.format_error())?;
        if token.token_type != TokenTypes::Comma {
            break;
        }
        interpreter.advance();
    }
    return Ok(Some(order_by_clauses));
}

fn get_limit(interpreter: &mut Interpreter) -> Result<Option<LimitClause>, String> {
    let token = interpreter.current_token().ok_or_else(|| interpreter.format_error())?;
    if token.token_type != TokenTypes::Limit {
        return Ok(None);
    }
    interpreter.advance();

    let limit = match interpreter.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::IntLiteral {
                return Err(interpreter.format_error());
            }
            token_to_value(interpreter)?
        },
        None => return Err(interpreter.format_error()),
    };
    interpreter.advance();

    let token = interpreter.current_token().ok_or_else(|| interpreter.format_error())?;
    if token.token_type != TokenTypes::Offset {
        return Ok(Some(LimitClause {
            limit: limit,
            offset: None,
        }));
    }
    interpreter.advance();

    let offset = match interpreter.current_token() {
        Some(token) => {
            if token.token_type != TokenTypes::IntLiteral {
                return Err(interpreter.format_error());
            }
            token_to_value(interpreter)?
        },
        None => return Err(interpreter.format_error()),
    };   

    interpreter.advance();
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
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
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
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
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
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
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
        let mut interpreter = Interpreter::new(tokens);
        let result = build(&mut interpreter);
        println!("{:?}", result);
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
}