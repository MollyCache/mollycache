use crate::cli::{
    ast::{
        parser::Parser, SqlStatement, DeleteStatement, 
        helpers::{
            common::{expect_token_type, get_table_name},
            order_by_clause::get_order_by, where_clause::get_where_clause, limit_clause::get_limit
        }
    },
    tokenizer::token::TokenTypes
};

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    expect_token_type(parser, TokenTypes::From)?;
    let table_name = get_table_name(parser)?;
    parser.advance()?;
    let where_clause = get_where_clause(parser)?;
    let order_by_clause = get_order_by(parser)?;
    let limit_clause = get_limit(parser)?;

    return Ok(SqlStatement::DeleteStatement(DeleteStatement {
        table_name: table_name,
        where_clause: where_clause,
        order_by_clause: order_by_clause,
        limit_clause: limit_clause,
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ast::test_utils::token;
    use crate::cli::ast::OrderByClause;
    use crate::cli::ast::OrderByDirection;
    use crate::cli::ast::LimitClause;
    use crate::cli::ast::Operator;
    use crate::cli::ast::WhereStackElement;
    use crate::cli::ast::WhereCondition;
    use crate::cli::ast::Operand;
    use crate::db::table::Value;

    #[test]
    fn delete_statement_with_all_tokens_is_generated_correctly() {
        // DELETE FROM users;
        let tokens = vec![
            token(TokenTypes::Delete, "DELETE"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
        ];
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::DeleteStatement(DeleteStatement {
            table_name: "users".to_string(),
            where_clause: None,
            order_by_clause: None,
            limit_clause: None,
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn delete_statement_with_all_clauses_is_generated_correctly() {
        // DELETE FROM users WHERE id = 1 ORDER BY id ASC LIMIT 10 OFFSET 5;
        let tokens = vec![
            token(TokenTypes::Delete, "DELETE"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
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
        let expected = SqlStatement::DeleteStatement(DeleteStatement {  
            table_name: "users".to_string(),
            where_clause: Some(vec![
                WhereStackElement::Condition(WhereCondition {
                    l_side: Operand::Identifier("id".to_string()),
                    operator: Operator::Equals,
                    r_side: Operand::Value(Value::Integer(1)),
                })
            ]),
            order_by_clause: Some(vec![
                OrderByClause {
                    column: "id".to_string(),
                    direction: OrderByDirection::Asc,
                }
            ]),
            limit_clause: Some(LimitClause {
                limit: Value::Integer(10),
                offset: Some(Value::Integer(5)),
            }),
        });
        assert_eq!(expected, statement);
    }
}