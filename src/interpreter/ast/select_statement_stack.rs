use crate::interpreter::ast::{parser::Parser, SqlStatement, SelectStatementStack, SelectStatementStackElement, SetOperator, SelectStackOperators};
use crate::interpreter::ast::helpers::select_statement;
use crate::interpreter::ast::Parentheses;
use crate::interpreter::tokenizer::token::TokenTypes;

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    let mut select_statement_stack: Vec<SelectStatementStackElement> = vec![];
    let mut set_operator_stack: Vec<SelectStackOperators> = vec![];

    loop {
        let token = parser.current_token()?;
        match token.token_type {
            TokenTypes::Select => {
                let statement = select_statement::get_statement(parser)?;
                select_statement_stack.push(SelectStatementStackElement::SelectStatement(statement));
            }
            TokenTypes::LeftParen => {
                set_operator_stack.push(SelectStackOperators::Parentheses(Parentheses::Left));
                parser.advance()?;
            }
            TokenTypes::RightParen => {
                while let Some(current_set_operator) = set_operator_stack.pop() {
                    if let SelectStackOperators::Parentheses(_) = current_set_operator {
                        break;
                    }
                    else if let SelectStackOperators::SetOperator(set_operator) = current_set_operator {
                        select_statement_stack.push(SelectStatementStackElement::SetOperator(set_operator));
                    }
                    else {
                        return Err("Mismatched parentheses found.".to_string());
                    }
                }
                parser.advance()?;
            }
            TokenTypes::Union | TokenTypes::Except => {
                let set_operator = get_set_operator(parser)?;
                while let Some(current_set_operator) = set_operator_stack.pop() {
                    if let SelectStackOperators::Parentheses(parentheses) = current_set_operator {
                        set_operator_stack.push(SelectStackOperators::Parentheses(parentheses));
                        break;
                    }
                    else if let SelectStackOperators::SetOperator(current_set_operator) = current_set_operator {
                        select_statement_stack.push(SelectStatementStackElement::SetOperator(current_set_operator));
                    }
                }
                set_operator_stack.push(SelectStackOperators::SetOperator(set_operator));
            }
            TokenTypes::Intersect => {
                let set_operator = get_set_operator(parser)?;
                while let Some(current_set_operator) = set_operator_stack.pop() {
                    if let SelectStackOperators::SetOperator(current_set_operator) = current_set_operator {
                        if set_operator.is_greater_precedence(&current_set_operator) {
                            set_operator_stack.push(SelectStackOperators::SetOperator(current_set_operator));
                            break;
                        }
                        else {
                            select_statement_stack.push(SelectStatementStackElement::SetOperator(current_set_operator));
                        }
                    }
                    else {
                        set_operator_stack.push(current_set_operator);
                        break;
                    }
                }
                set_operator_stack.push(SelectStackOperators::SetOperator(set_operator));
            }
            TokenTypes::SemiColon => break,
            _ => return Err(parser.format_error()),
        }
    }

    while let Some(current_set_operator) = set_operator_stack.pop() {
        if let SelectStackOperators::SetOperator(set_operator) = current_set_operator {
            select_statement_stack.push(SelectStatementStackElement::SetOperator(set_operator));
        }
        else {
            return Err("Mismatched parentheses found.".to_string());
        }
    }

    return Ok(SqlStatement::Select(SelectStatementStack {
        elements: select_statement_stack,
    }));
}

fn get_set_operator(parser: &mut Parser) -> Result<SetOperator, String> {
    let token = parser.current_token()?;
    let set_operator = match token.token_type {
        TokenTypes::Union => {
            if parser.peek_token()?.token_type == TokenTypes::All {
                parser.advance()?;
                Ok(SetOperator::UnionAll)
            } else {
                Ok(SetOperator::Union)
            }
        },
        TokenTypes::Except => {
            Ok(SetOperator::Except)
        },
        TokenTypes::Intersect => {
            Ok(SetOperator::Intersect)
        },
        _ => Err("Expected token type: Union, Except, or Intersect was not found".to_string()),
    };
    parser.advance()?;
    return set_operator;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::ast::test_utils::token;
    use crate::interpreter::ast::SelectStatement;
    use crate::interpreter::ast::SetOperator;
    use crate::interpreter::ast::SelectStatementColumns;
    use crate::interpreter::ast::WhereStackElement;
    use crate::interpreter::ast::WhereCondition;
    use crate::interpreter::ast::Operand;
    use crate::interpreter::ast::Operator;
    use crate::db::table::Value;
    use crate::interpreter::tokenizer::token::TokenTypes;
    use crate::interpreter::tokenizer::scanner::Token;

    fn simple_select_statement_tokens(id: &'static str) -> Vec<Token<'static>> {
        vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Asterisk, "*"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Where, "WHERE"),
            token(TokenTypes::Identifier, "id"),
            token(TokenTypes::Equals, "="),
            token(TokenTypes::IntLiteral, id),
        ]
    }

    fn expected_simple_select_statement(id: i64) -> SelectStatementStackElement {
        SelectStatementStackElement::SelectStatement(SelectStatement {
            table_name: "users".to_string(),
            columns: SelectStatementColumns::All,
            where_clause: Some(vec![WhereStackElement::Condition(WhereCondition {
                l_side: Operand::Identifier("id".to_string()),
                operator: Operator::Equals,
                r_side: Operand::Value(Value::Integer(id)),
            })]),
            order_by_clause: None,
            limit_clause: None,
        })
    }


    #[test]
    fn simple_select_statement_is_generated_correctly() {
        // SELECT * FROM users WHERE id = 1;
        let mut tokens = simple_select_statement_tokens("1");
        tokens.append(&mut vec![token(TokenTypes::SemiColon, ";")]);
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::Select(SelectStatementStack {
            elements: vec![expected_simple_select_statement(1)],
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_set_operator_is_generated_correctly() {
        // SELECT * FROM users WHERE id = 1 UNION ALL SELECT * FROM users WHERE id = 2;
        let mut tokens = simple_select_statement_tokens("1");
        tokens.append(&mut vec![token(TokenTypes::Union, "UNION"), token(TokenTypes::All, "ALL")]);
        tokens.append(&mut simple_select_statement_tokens("2"));
        tokens.append(&mut vec![token(TokenTypes::SemiColon, ";")]);
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        println!("{:?}", result);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::Select(SelectStatementStack {
            elements: vec![
                expected_simple_select_statement(1),
                expected_simple_select_statement(2),
                SelectStatementStackElement::SetOperator(SetOperator::UnionAll),
            ],
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_multiple_set_operators_is_generated_correctly() {
        // SELECT 1 ... UNION ALL SELECT 2 ... INTERSECT SELECT 3 ... EXCEPT SELECT 4 ...;
        let mut tokens = simple_select_statement_tokens("1");
        tokens.append(&mut vec![token(TokenTypes::Union, "UNION")]);
        tokens.append(&mut simple_select_statement_tokens("2"));
        tokens.append(&mut vec![token(TokenTypes::Intersect, "INTERSECT")]);
        tokens.append(&mut simple_select_statement_tokens("3"));
        tokens.append(&mut vec![token(TokenTypes::Except, "EXCEPT")]);
        tokens.append(&mut simple_select_statement_tokens("4"));
        tokens.append(&mut vec![token(TokenTypes::SemiColon, ";")]);
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        println!("{:?}", result);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::Select(SelectStatementStack {
            elements: vec![
                expected_simple_select_statement(1),
                expected_simple_select_statement(2),
                expected_simple_select_statement(3),
                SelectStatementStackElement::SetOperator(SetOperator::Intersect),
                SelectStatementStackElement::SetOperator(SetOperator::Union),
                expected_simple_select_statement(4),
                SelectStatementStackElement::SetOperator(SetOperator::Except),
            ],
        });
        assert_eq!(expected, statement);
    }

    #[test]
    fn select_statement_with_multiple_set_operators_and_parentheses_is_generated_correctly() {
        // (SELECT 1 ... UNION ALL SELECT 2 ...) INTERSECT (SELECT 3 ... EXCEPT SELECT 4 ...);
        let mut tokens = vec![token(TokenTypes::LeftParen, "(")];
        tokens.append(&mut simple_select_statement_tokens("1"));
        tokens.append(&mut vec![token(TokenTypes::Union, "UNION")]);
        tokens.append(&mut vec![token(TokenTypes::All, "ALL")]);
        tokens.append(&mut simple_select_statement_tokens("2"));
        tokens.append(&mut vec![token(TokenTypes::RightParen, ")")]);
        tokens.append(&mut vec![token(TokenTypes::Intersect, "INTERSECT")]);
        tokens.append(&mut vec![token(TokenTypes::LeftParen, "(")]);
        tokens.append(&mut simple_select_statement_tokens("3"));
        tokens.append(&mut vec![token(TokenTypes::Except, "EXCEPT")]);
        tokens.append(&mut simple_select_statement_tokens("4"));
        tokens.append(&mut vec![token(TokenTypes::RightParen, ")")]);
        tokens.append(&mut vec![token(TokenTypes::SemiColon, ";")]);
        let mut parser = Parser::new(tokens);
        let result = build(&mut parser);
        println!("{:?}", result);
        assert!(result.is_ok());
        let statement = result.unwrap();
        let expected = SqlStatement::Select(SelectStatementStack {
            elements: vec![
                expected_simple_select_statement(1),
                expected_simple_select_statement(2),
                SelectStatementStackElement::SetOperator(SetOperator::UnionAll),
                expected_simple_select_statement(3),
                expected_simple_select_statement(4),
                SelectStatementStackElement::SetOperator(SetOperator::Except),
                SelectStatementStackElement::SetOperator(SetOperator::Intersect),
            ],
        });
        assert_eq!(expected, statement);
    }
}