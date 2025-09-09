use crate::interpreter::tokenizer::{scanner::Token, token::TokenTypes};
use crate::db::table::{ColumnDefinition, Value};

mod create_statement;
mod insert_statement;
mod parser;
mod select_statement_stack;
mod update_statement;
mod delete_statement;
mod helpers;
mod drop_statement;
mod alter_table_statement;
#[cfg(test)]
mod test_utils;

#[derive(Debug, PartialEq)]
pub struct DatabaseSqlStatement {
    pub sql_statement: SqlStatement,
    pub line_num: usize,
    pub statement_text: String,
}

#[derive(Debug, PartialEq)]
pub enum SqlStatement {
    CreateTable(CreateTableStatement),
    InsertInto(InsertIntoStatement),
    Select(SelectStatementStack),
    UpdateStatement(UpdateStatement),
    DeleteStatement(DeleteStatement),
    DropTable(DropTableStatement),
    AlterTable(AlterTableStatement),
}

#[derive(Debug, PartialEq)]
pub struct CreateTableStatement {
    pub table_name: String,
    pub existence_check: Option<ExistenceCheck>,
    pub columns: Vec<ColumnDefinition>,
}

#[derive(Debug, PartialEq)]
pub struct DropTableStatement {
    pub table_name: String,
    pub existence_check: Option<ExistenceCheck>,
}

#[derive(Debug, PartialEq)]
pub enum ExistenceCheck  { // Eventually expand to temp tables
    IfNotExists,
    IfExists,
}

#[derive(Debug, PartialEq)]
pub struct InsertIntoStatement {
    pub table_name: String,
    pub columns: Option<Vec<String>>,
    pub values: Vec<Vec<Value>>,
}

#[derive(Debug, PartialEq)]
pub struct SelectStatementStack {
    pub columns: SelectStatementColumns,
    pub elements: Vec<SelectStatementStackElement>,
    pub order_by_clause: Option<Vec<OrderByClause>>,
    pub limit_clause: Option<LimitClause>,
}

#[derive(Debug, PartialEq)]
pub enum SelectStatementStackElement {
    SelectStatement(SelectStatement),
    SetOperator(SetOperator),
}

#[derive(Debug, PartialEq)]
pub enum SelectStackOperators {
    SetOperator(SetOperator),
    Parentheses(Parentheses),
}

#[derive(Debug, PartialEq)]
pub enum SetOperator {
    Union,
    UnionAll,
    Intersect,
    Except,
}

impl SetOperator {
    pub fn is_greater_precedence(&self, other: &SetOperator) -> bool {
        match (self, other) {
            (SetOperator::Intersect, SetOperator::Intersect) => false,
            (SetOperator::Intersect, _) => true,
            (_, _) => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SelectStatement {
    pub table_name: String,
    pub mode: SelectMode,
    pub columns: SelectStatementColumns,
    pub where_clause: Option<Vec<WhereStackElement>>,
    pub order_by_clause: Option<Vec<OrderByClause>>,
    pub limit_clause: Option<LimitClause>,
}

#[derive(Debug, PartialEq)]
pub struct DeleteStatement {
    pub table_name: String,
    pub where_clause: Option<Vec<WhereStackElement>>,
    pub order_by_clause: Option<Vec<OrderByClause>>,
    pub limit_clause: Option<LimitClause>,
}

#[derive(Debug, PartialEq)]
pub struct UpdateStatement {
    pub table_name: String,
    pub update_values: Vec<ColumnValue>,
    pub where_clause: Option<Vec<WhereStackElement>>,
    pub order_by_clause: Option<Vec<OrderByClause>>,
    pub limit_clause: Option<LimitClause>,
}

#[derive(Debug, PartialEq)]
pub struct AlterTableStatement {
    pub table_name: String,
    pub action: AlterTableAction,
}

#[derive(Debug, PartialEq)]
pub enum AlterTableAction {
    RenameTable { new_table_name: String },
    RenameColumn { old_column_name: String, new_column_name: String },
    AddColumn { column_def: ColumnDefinition },
    DropColumn { column_name: String },
}

#[derive(Debug, PartialEq)]
pub struct ColumnValue {
    pub column: String,
    pub value: Value,
}

#[derive(Debug, PartialEq)]
pub enum SelectMode {
    All,
    Distinct,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SelectStatementColumns {
    All,
    Specific(Vec<String>),
}

impl SelectStatementColumns {
    pub fn columns(&self) -> Result<Vec<&String>, String> {
        return match self {
            SelectStatementColumns::All => Err("Cannot get columns from all columns".to_string()),
            SelectStatementColumns::Specific(columns) => Ok(columns.iter().map(|column| column).collect()),
        }
    }
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Clone))]
pub enum Operator {
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessEquals,
    GreaterEquals,
    In,
    NotIn,
    Is,
    IsNot,
}

#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Clone))]
pub struct WhereCondition {
    pub l_side: Operand,
    pub operator: Operator,
    pub r_side: Operand,
}


#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Clone))]
pub enum Operand {
    Value(Value),
    ValueList(Vec<Value>),
    Identifier(String),
}


#[derive(Debug, PartialEq)]
pub enum WhereStackElement {
    Condition(WhereCondition),
    LogicalOperator(LogicalOperator),
    Parentheses(Parentheses),
}

#[derive(Debug, PartialEq)]
pub enum WhereStackOperators {
    LogicalOperator(LogicalOperator),
    Parentheses(Parentheses),
}

impl WhereStackOperators {
    pub fn into_where_stack_element(self) -> WhereStackElement {
        match self {
            WhereStackOperators::LogicalOperator(logical_operator) => WhereStackElement::LogicalOperator(logical_operator),
            WhereStackOperators::Parentheses(parentheses) => WhereStackElement::Parentheses(parentheses),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum LogicalOperator {
    Not,
    And,
    Or,
}

impl LogicalOperator {
    pub fn is_greater_precedence(&self, other: &LogicalOperator) -> bool {
        match (self, other) {
            (LogicalOperator::Not, LogicalOperator::Not) => false,
            (LogicalOperator::Not, _) => true,
            (LogicalOperator::And, LogicalOperator::Not) => false,
            (LogicalOperator::And, LogicalOperator::And) => false,
            (LogicalOperator::And, LogicalOperator::Or) => true,
            (LogicalOperator::Or, LogicalOperator::Not) => false,
            (LogicalOperator::Or, LogicalOperator::And) => false,
            (LogicalOperator::Or, LogicalOperator::Or) => false,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Parentheses {
    Left,
    Right,
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
    fn build_update(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String>;
    fn build_delete(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String>;
    fn build_drop(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String>;
    fn build_alter(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String>;
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
        select_statement_stack::build(parser)
    }

    fn build_update(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String> {
        update_statement::build(parser)
    }

    fn build_delete(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String> {
        delete_statement::build(parser)
    }

    fn build_drop(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String> {
        drop_statement::build(parser)
    }

    fn build_alter(&self, parser: &mut parser::Parser) -> Result<SqlStatement, String> {
        alter_table_statement::build(parser)
    }
}

pub fn generate(tokens: Vec<Token>) -> Vec<Result<DatabaseSqlStatement, String>> {
    let mut results: Vec<Result<DatabaseSqlStatement, String>> = vec![];
    let mut parser = parser::Parser::new(tokens);
    let builder : &dyn StatementBuilder = &DefaultStatementBuilder;
    loop {
        let line_num = match parser.line_num() {
            Ok(line_num) => line_num,
            Err(err) => {
                results.push(Err(err));
                break;
            }
        };
        let next_statement = parser.next_statement(builder);
        if let Some(next_statement) = next_statement {
            match next_statement {
                Err(error) => {
                    results.push(Err(error));
                    // If we encountered a parsing error, skip until we find a semicolon or EOF
                    loop {
                        if let Ok(token) = parser.current_token() {
                            if token.token_type == TokenTypes::EOF {
                                break;
                            }
                            else if token.token_type == TokenTypes::SemiColon {
                                let _ = parser.advance_past_semicolon();
                                break;
                            }
                            else {
                                if parser.advance().is_err() {
                                    return results;
                                }
                            }
                        }
                        else {
                            break;
                        }
                    }
                }
                Ok(sql_statement) => {
                    let parser_advance_result = parser.advance_past_semicolon();
                    if parser_advance_result.is_err() {
                        results.push(Err(parser_advance_result.err().unwrap()));
                        return results;
                    }
                    results.push(
                        Ok(DatabaseSqlStatement {
                            sql_statement: sql_statement,
                            line_num: line_num,
                            statement_text: "".to_string(),
                        })
                    );
                }
            }
        } else {
            break;
        }
    }
    return results;
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_utils::token;

    #[test]
    fn ast_handles_invalid_statements_gracefully() {
        let tokens = vec![
            token(TokenTypes::Select, "INSERT"),
            token(TokenTypes::Into, "INTO"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Alice"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let result = generate(tokens);
        assert!(result[0].is_err());
        let expected = vec![Err("Error at line 1, column 0: Unexpected value: INTO".to_string())];
        assert_eq!(expected, result);
    }

    #[test]
    fn ast_handles_multiple_statements() {
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Asterisk, "*"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::Insert, "INSERT"),
            token(TokenTypes::Into, "INTO"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Values, "VALUES"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Alice"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let result = generate(tokens);
        assert!(result[0].is_ok());
        assert!(result[1].is_ok());
        let expected = vec![
            Ok(DatabaseSqlStatement {
                sql_statement: SqlStatement::Select(SelectStatementStack {
                    columns: SelectStatementColumns::All,
                    elements: vec![SelectStatementStackElement::SelectStatement(SelectStatement {
                        table_name: "users".to_string(),
                        mode: SelectMode::All,
                        columns: SelectStatementColumns::All,
                        where_clause: None,
                        order_by_clause: None,
                        limit_clause: None,
                    })],
                    order_by_clause: None,
                    limit_clause: None,
                }),
                line_num: 1,
                statement_text: "".to_string(),
            }),
            Ok(DatabaseSqlStatement {
                    sql_statement: SqlStatement::InsertInto(InsertIntoStatement {
                        table_name: "users".to_string(),
                        columns: None,
                        values: vec![
                            vec![Value::Integer(1), Value::Text("Alice".to_string())],
                        ],
                }),
                    line_num: 1,
                    statement_text: "".to_string(),
            }),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn ast_handles_invalid_statement_then_valid_statement() {
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::Insert, "INSERT"),
            token(TokenTypes::Into, "INTO"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Values, "VALUES"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Alice"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let result = generate(tokens);
        assert!(result[0].is_err());
        assert!(result[1].is_ok());
        let expected = vec![
            Err("Error at line 1, column 0: Unexpected value: ;".to_string()),
            Ok(DatabaseSqlStatement {
                sql_statement: SqlStatement::InsertInto(InsertIntoStatement {
                    table_name: "users".to_string(),
                    columns: None,
                    values: vec![
                        vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    ],
                }),
                line_num: 1,
                statement_text: "".to_string(),
            }),
        ];
        assert_eq!(expected, result);
    }

    #[test]
    fn ast_handles_multiple_valid_statements() {
        let tokens = vec![
            token(TokenTypes::Select, "SELECT"),
            token(TokenTypes::Asterisk, "*"),
            token(TokenTypes::From, "FROM"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::Insert, "INSERT"),
            token(TokenTypes::Into, "INTO"),
            token(TokenTypes::Identifier, "users"),
            token(TokenTypes::Values, "VALUES"),
            token(TokenTypes::LeftParen, "("),
            token(TokenTypes::IntLiteral, "1"),
            token(TokenTypes::Comma, ","),
            token(TokenTypes::String, "Alice"),
            token(TokenTypes::RightParen, ")"),
            token(TokenTypes::SemiColon, ";"),
            token(TokenTypes::EOF, ""),
        ];
        let result = generate(tokens);
        assert!(result[0].is_ok());
        assert!(result[1].is_ok());
        let expected = vec![
            Ok(DatabaseSqlStatement {
                sql_statement: SqlStatement::Select(SelectStatementStack {
                    columns: SelectStatementColumns::All,
                    elements: vec![SelectStatementStackElement::SelectStatement(SelectStatement {
                        table_name: "users".to_string(),
                        mode: SelectMode::All,
                        columns: SelectStatementColumns::All,
                        where_clause: None,
                        order_by_clause: None,
                        limit_clause: None,
                    })],
                    order_by_clause: None,
                    limit_clause: None,
                }),
                line_num: 1,
                statement_text: "".to_string(),
            }),
            Ok(DatabaseSqlStatement {
                sql_statement: SqlStatement::InsertInto(InsertIntoStatement {
                    table_name: "users".to_string(),
                    columns: None,
                    values: vec![
                        vec![Value::Integer(1), Value::Text("Alice".to_string())],
                    ],
                }),
                line_num: 1,
                statement_text: "".to_string(),
            }),
        ];
        assert_eq!(expected, result);
    }
}