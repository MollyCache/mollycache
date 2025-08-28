use crate::cli::{self, tokenize::TokenTypes};

mod create_statement;
mod insert_statement;
mod select_statement;
mod interpreter;

pub enum SqlStatement {
    CreateTable(CreateTableStatement),
    Insert(InsertStatement),
    Select(SelectStatement),
}

pub struct CreateTableStatement {
    pub table_name: String,
    pub columns: Vec<cli::table::ColumnDefinition>
}

pub struct InsertStatement {
    pub table_name: String,
    pub columns: Vec<String>,
    pub values: Vec<cli::table::Value>
}

pub struct SelectStatement {
    pub table_name: String,
    pub columns: Vec<String>,
    pub where_clause: Option<WhereClause>
}

pub struct WhereClause {
    pub column: String,
    pub value: cli::table::Value
}

pub fn generate(tokens: Vec<cli::tokenize::Token>) -> Vec<Result<SqlStatement, String>> {
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