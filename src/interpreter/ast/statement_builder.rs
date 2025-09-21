use crate::interpreter::ast::SqlStatement;
use crate::interpreter::ast::parser::Parser;
use crate::interpreter::ast::{
    alter_table_statement, create_statement, delete_statement, drop_statement, insert_statement,
    select_statement_stack, transaction_statements, update_statement,
};

pub trait StatementBuilder {
    fn build_create(&self, parser: &mut Parser) -> Result<SqlStatement, String>;
    fn build_insert(&self, parser: &mut Parser) -> Result<SqlStatement, String>;
    fn build_select(&self, parser: &mut Parser) -> Result<SqlStatement, String>;
    fn build_update(&self, parser: &mut Parser) -> Result<SqlStatement, String>;
    fn build_delete(&self, parser: &mut Parser) -> Result<SqlStatement, String>;
    fn build_drop(&self, parser: &mut Parser) -> Result<SqlStatement, String>;
    fn build_alter(&self, parser: &mut Parser) -> Result<SqlStatement, String>;
    fn build_begin(&self, parser: &mut Parser) -> Result<SqlStatement, String>;
    fn build_commit(&self, parser: &mut Parser) -> Result<SqlStatement, String>;
    fn build_rollback(&self, parser: &mut Parser) -> Result<SqlStatement, String>;
    fn build_savepoint(&self, parser: &mut Parser) -> Result<SqlStatement, String>;
    fn build_release(&self, parser: &mut Parser) -> Result<SqlStatement, String>;
}

pub struct DefaultStatementBuilder;

impl StatementBuilder for DefaultStatementBuilder {
    fn build_create(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        create_statement::build(parser)
    }

    fn build_insert(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        insert_statement::build(parser)
    }

    fn build_select(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        select_statement_stack::build(parser)
    }

    fn build_update(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        update_statement::build(parser)
    }

    fn build_delete(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        delete_statement::build(parser)
    }

    fn build_drop(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        drop_statement::build(parser)
    }

    fn build_alter(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        alter_table_statement::build(parser)
    }

    fn build_begin(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        transaction_statements::build_begin(parser)
    }

    fn build_commit(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        transaction_statements::build_commit(parser)
    }

    fn build_rollback(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        transaction_statements::build_rollback(parser)
    }

    fn build_savepoint(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        transaction_statements::build_savepoint(parser)
    }

    fn build_release(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        transaction_statements::build_release(parser)
    }
}

#[cfg(test)]
pub struct MockStatementBuilder;
#[cfg(test)]
use crate::interpreter::ast::{
    CreateTableStatement, InsertIntoStatement, SelectMode, SelectStatement, SelectStatementColumn, SelectStatementTable,
    SelectStatementStack, SelectStatementStackElement, SelectableStack, SelectableStackElement,
};

#[cfg(test)]
impl StatementBuilder for MockStatementBuilder {
    fn build_create(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        parser.advance()?;
        parser.advance_past_semicolon()?;
        return Ok(SqlStatement::CreateTable(CreateTableStatement {
            table_name: "users".to_string(),
            existence_check: None,
            columns: vec![],
        }));
    }

    fn build_insert(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        parser.advance()?;
        parser.advance_past_semicolon()?;
        return Ok(SqlStatement::InsertInto(InsertIntoStatement {
            table_name: "users".to_string(),
            columns: None,
            values: vec![],
        }));
    }

    fn build_select(&self, parser: &mut Parser) -> Result<SqlStatement, String> {
        parser.advance()?;
        parser.advance_past_semicolon()?;
        return Ok(SqlStatement::Select(SelectStatementStack {
            elements: vec![SelectStatementStackElement::SelectStatement(
                SelectStatement {
                    table_name: SelectStatementTable::new("users".to_string()),
                    mode: SelectMode::All,
                    columns: SelectableStack {
                        selectables: vec![SelectableStackElement::All],
                    },
                    column_names: vec![SelectStatementColumn::new("*".to_string())],
                    where_clause: None,
                    order_by_clause: None,
                    limit_clause: None,
                },
            )],
            order_by_clause: None,
            limit_clause: None,
        }));
    }

    fn build_update(&self, _parser: &mut Parser) -> Result<SqlStatement, String> {
        todo!();
    }

    fn build_delete(&self, _parser: &mut Parser) -> Result<SqlStatement, String> {
        todo!();
    }

    fn build_drop(&self, _parser: &mut Parser) -> Result<SqlStatement, String> {
        todo!();
    }

    fn build_alter(&self, _parser: &mut Parser) -> Result<SqlStatement, String> {
        todo!();
    }

    fn build_begin(&self, _parser: &mut Parser) -> Result<SqlStatement, String> {
        todo!();
    }

    fn build_commit(&self, _parser: &mut Parser) -> Result<SqlStatement, String> {
        todo!();
    }

    fn build_rollback(&self, _parser: &mut Parser) -> Result<SqlStatement, String> {
        todo!();
    }

    fn build_savepoint(&self, _parser: &mut Parser) -> Result<SqlStatement, String> {
        todo!();
    }

    fn build_release(&self, _parser: &mut Parser) -> Result<SqlStatement, String> {
        todo!();
    }
}
