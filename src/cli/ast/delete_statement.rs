use crate::cli::ast::{parser::Parser, SqlStatement, DeleteStatement};

pub fn build(_parser: &mut Parser) -> Result<SqlStatement, String> {
    return Ok(SqlStatement::DeleteStatement(DeleteStatement {
        table_name: "".to_string(),
        where_clause: None,
        order_by_clause: None,
        limit_clause: None,
    }));
}