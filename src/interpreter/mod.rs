use crate::db;
use crate::db::table::Value;
pub mod ast;
mod tokenizer;

pub fn run_sql(database: &mut db::database::Database, sql: &str) -> Vec<Result<Option<Vec<Vec<Value>>>, String>> {
    let tokens = tokenizer::tokenize(sql);
    // println!("{:?}", tokens);
    let ast = ast::generate(tokens);

    let mut sql_results = vec![];
    for sql_statement in ast {
        // println!("{:?}", sql_statement);
        match sql_statement {
            Ok(statement) => {
                let result = database.execute(statement.sql_statement);
                match result {
                    Ok(values) => {
                        if let Some(rows) = values {
                            sql_results.push(Ok(Some(rows)));
                        }
                        else {
                            sql_results.push(Ok(None));
                        }
                    }
                    Err(error) => {
                        sql_results.push(Err(format!("Execution Error with statement starting on line {} \n Error: {}", statement.line_num, error)));
                    }
                }
            },
            Err(parser_error) => {
                sql_results.push(Err(format!("Parsing Error: {}", parser_error)));
            },
        }
    }
    return sql_results;
}
