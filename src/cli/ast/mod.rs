use crate::cli::{self, table::{Value, ColumnDefinition}};

mod common;
mod create_statement;
mod insert_statement;
mod parser;
mod select_statement;

#[derive(Debug, PartialEq)]
pub enum SqlStatement {
    CreateTable(CreateTableStatement),
    InsertInto(InsertIntoStatement),
    Select(SelectStatement),
}

#[derive(Debug, PartialEq)]
pub struct CreateTableStatement {
    pub table_name: String,
    pub columns: Vec<ColumnDefinition>,
}

#[derive(Debug, PartialEq)]
pub struct InsertIntoStatement {
    pub table_name: String,
    pub columns: Option<Vec<String>>,
    pub values: Vec<Vec<Value>>,
}

#[derive(Debug, PartialEq)]
pub struct SelectStatement {
    pub table_name: String,
    pub columns: SelectStatementColumns,
    pub where_clause: Option<WhereClause>,
    pub order_by_clause: Option<Vec<OrderByClause>>,
    pub limit_clause: Option<LimitClause>,
}

#[derive(Debug, PartialEq)]
pub enum SelectStatementColumns {
    All,
    Specific(Vec<String>),
}

#[derive(Debug, PartialEq)]
pub enum Operator {
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessEquals,
    GreaterEquals,
}

#[derive(Debug, PartialEq)]
pub struct WhereClause {
    pub column: String,
    pub operator: Operator,
    pub value: Value,
}

#[derive(Debug, PartialEq)]
pub enum OrderByDirection {
    Asc,
    Desc,
}

#[derive(Debug, PartialEq)]
pub struct OrderByClause {
    pub column: String,
    pub direction: OrderByDirection,
}

#[derive(Debug, PartialEq)]
pub struct LimitClause {
    pub limit: Value,
    pub offset: Option<Value>,
}

pub trait StatementBuilder {
    fn build_create(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String>;
    fn build_insert(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String>;
    fn build_select(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String>;
}

pub struct DefaultStatementBuilder;

impl StatementBuilder for DefaultStatementBuilder {
    fn build_create(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String> {
        create_statement::build(parser)
    }
    
    fn build_insert(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String> {
        insert_statement::build(parser)
    }
    
    fn build_select(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String> {
        select_statement::build(parser)
    }
}

pub fn generate(tokens: Vec<cli::tokenizer::scanner::Token>) -> Vec<Result<SqlStatement, String>> {
    let mut results: Vec<Result<SqlStatement, String>> = vec![];
    let mut parser = parser::Parser::new(tokens);
    let builder : &dyn StatementBuilder = &DefaultStatementBuilder;
    loop {
        let next_statement = parser.next_statement(builder);
        if let Some(next_statement) = next_statement {
            results.push(next_statement);
        } else {
            break;
        }
    }
    return results;
}
