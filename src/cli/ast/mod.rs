use crate::cli::{self, tokenizer::scanner::Token, table::Value};

mod create_statement;
mod insert_statement;
mod interpreter;
mod select_statement;

#[derive(Debug, PartialEq)]
pub enum SqlStatement {
    CreateTable(CreateTableStatement),
    Insert(InsertStatement),
    Select(SelectStatement),
}

#[derive(Debug, PartialEq)]
pub struct CreateTableStatement {
    pub table_name: String,
    pub columns: Vec<cli::table::ColumnDefinition>,
}

#[derive(Debug, PartialEq)]
pub struct InsertStatement {
    pub table_name: String,
    pub columns: Vec<String>,
    pub values: Vec<Value>,
}

#[derive(Debug, PartialEq)]
pub struct SelectStatement {
    pub table_name: String,
    pub columns: Vec<String>,
    pub where_clause: Option<WhereClause>,
}

#[derive(Debug, PartialEq)]
pub struct WhereClause {
    pub column: String,
    pub value: cli::table::Value,
}

pub fn generate(tokens: Vec<cli::tokenizer::scanner::Token>) -> Vec<Result<SqlStatement, String>> {
    let mut results: Vec<Result<SqlStatement, String>> = vec![];
    let mut interpreter = interpreter::Interpreter::new(tokens);
    loop {
        let next_statement = interpreter.next_statement();
        if let Some(next_statement) = next_statement {
            results.push(next_statement);
        } else {
            break;
        }
    }
    return results;
}
