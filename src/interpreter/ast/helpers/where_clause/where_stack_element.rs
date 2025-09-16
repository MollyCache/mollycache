use crate::interpreter::ast::helpers::where_clause::where_condition::get_condition;
use crate::interpreter::ast::{
    LogicalOperator, Parentheses, WhereStackElement, WhereStackOperators, parser::Parser,
};
use crate::interpreter::tokenizer::token::TokenTypes;

pub fn get_where_stack_element(
    parser: &mut Parser,
    operator_stack: &Vec<WhereStackOperators>,
) -> Result<Option<WhereStackElement>, String> {
    let token_type = &parser.current_token()?.token_type;
    match token_type {
        TokenTypes::And
        | TokenTypes::Or
        | TokenTypes::Not
        | TokenTypes::LeftParen
        | TokenTypes::RightParen => {
            if token_type == &TokenTypes::RightParen
                && !operator_stack.contains(&WhereStackOperators::Parentheses(Parentheses::Left))
            {
                return Ok(None); // Mismatched parens can be caused by the UNION STATEMENTs.
            }
            let where_stack_element = token_type_to_where_stack_element(token_type);
            parser.advance()?;
            Ok(Some(where_stack_element))
        }
        TokenTypes::Identifier
        | TokenTypes::IntLiteral
        | TokenTypes::RealLiteral
        | TokenTypes::StringLiteral
        | TokenTypes::Blob
        | TokenTypes::Null => {
            return Ok(Some(WhereStackElement::Condition(get_condition(parser)?)));
        }
        _ => return Ok(None),
    }
}

fn token_type_to_where_stack_element(token_type: &TokenTypes) -> WhereStackElement {
    match token_type {
        TokenTypes::And => WhereStackElement::LogicalOperator(LogicalOperator::And),
        TokenTypes::Or => WhereStackElement::LogicalOperator(LogicalOperator::Or),
        TokenTypes::Not => WhereStackElement::LogicalOperator(LogicalOperator::Not),
        TokenTypes::LeftParen => WhereStackElement::Parentheses(Parentheses::Left),
        TokenTypes::RightParen => WhereStackElement::Parentheses(Parentheses::Right),
        _ => unreachable!("Invalid token type for where stack element"),
    }
}
