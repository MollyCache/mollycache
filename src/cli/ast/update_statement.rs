use crate::cli::ast::{parser::Parser, SqlStatement};
use crate::cli::ast::UpdateStatement;

pub fn build(parser: &mut Parser) -> Result<SqlStatement, String> {
    parser.advance()?;
    let statement = UpdateStatement { 
        table_name: "".to_string(),
        update_values: vec![],
        where_clause: None,
    };
    return Ok(SqlStatement::UpdateStatement(statement));
}