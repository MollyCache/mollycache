use crate::db;
use crate::db::table::Value;
pub mod ast;
mod tokenizer;

pub fn run_sql(database: &mut db::database::Database, sql: &str) -> Result<Option<Vec<Vec<Value>>>, String> {
    let tokens = tokenizer::tokenize(sql);
    // println!("{:?}", tokens);
    let ast = ast::generate(tokens);

    for sql_statement in ast {
        // println!("{:?}", sql_statement);
        match sql_statement {
            Ok(statement) => {
                let result = database.execute(statement);
                if let Ok(values) = result {
                    if let Some(rows) = values {
                        return Ok(Some(rows));
                    }
                    else {
                        return Ok(None);
                    }
                }
                else {
                    return Err(result.unwrap_err());
                }
            },
            Err(error) => {
                return Err(error);
            },
        }
    }
    return Ok(None);
}
