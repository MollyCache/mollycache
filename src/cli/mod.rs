use std::io;
use crate::db;
pub mod ast;
mod tokenizer;

pub fn cli() {
    clear_screen();
    println!("Welcome to the MollyDB CLI");
    let mut line_count = 1;
    let mut database = db::database::Database::new();

    loop {
        print!("({:03}) > ", line_count);
        line_count += 1;

        io::Write::flush(&mut io::stdout()).unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("exit") {
            break;
        }
        if input.eq_ignore_ascii_case("clear") {
            clear_screen();
            println!("Welcome to the MollyDB CLI");
            line_count = 1;
            continue;
        }

        let tokens = tokenizer::tokenize(input);
        println!("{:?}", tokens);
        let ast = ast::generate(tokens);
        for sql_statement in ast {
            println!("{:?}", sql_statement);
            match sql_statement {
                Ok(statement) => {
                    let result = database.execute(statement);
                    if let Ok(result) = result {
                        if let Some(rows) = result {
                            for row in rows {
                                println!("{:?}", row);
                            }
                        }
                        else {
                            println!("Executed Successfully");
                        }
                    }
                    else {
                        println!("Error: {}", result.unwrap_err());
                    }
                },
                Err(error) => {
                    println!("Error: {}", error);
                },
            }
        }
    }
}
fn clear_screen() {
    // Clear screen and move cursor to top-left
    print!("\x1B[2J\x1B[1;1H");
    io::Write::flush(&mut io::stdout()).unwrap();
}

