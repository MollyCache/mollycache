use crate::interpreter::ast::{parser::Parser, SqlStatement, SelectStatementStack, SelectStatementStackElement};
use crate::interpreter::ast::helpers::select_statement;

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    let statement = select_statement::get_statement(parser)?;
    return Ok(SqlStatement::Select(SelectStatementStack {
        elements: vec![SelectStatementStackElement::SelectStatement(statement)],
    }));
}