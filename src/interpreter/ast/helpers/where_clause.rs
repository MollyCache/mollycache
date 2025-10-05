use crate::interpreter::ast::helpers::common::get_selectables;
use crate::interpreter::ast::{SelectableColumn, parser::Parser};
use crate::interpreter::tokenizer::token::TokenTypes;

use crate::interpreter::ast::helpers::token::expect_token_type;

pub fn get_where_clause(parser: &mut Parser) -> Result<Option<SelectableColumn>, String> {
    if expect_token_type(parser, TokenTypes::Where).is_err() {
        return Ok(None);
    }
    parser.advance()?;

    let mut condition = get_selectables(parser, false, false, &mut None)?;
    if condition.len() != 1 {
        return Err("Invalid WHERE condition".to_string());
    }

    return Ok(Some(
        condition
            .pop()
            .ok_or("Invalid WHERE condition".to_string())?,
    ));
}

#[cfg(test)]
mod tests {
    // TODO: tests
}
