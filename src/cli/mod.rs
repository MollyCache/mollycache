use std::io;
use crate::db;
use crate::interpreter::run_sql;

pub fn cli(database: &mut db::database::Database) {
    clear_screen();
    println!("Welcome to the MollyDB CLI");
    let mut line_count = 1;

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

        let result = run_sql(database, input);
        if let Ok(Some(rows)) = result {
            for row in rows {
                println!("{:?}", row);
            }
        }
        else if let Ok(None) = result {
            println!("Executed Successfully");
        }
        else {
            println!("Error: {}", result.unwrap_err());
        }
    }
}

fn clear_screen() {
    // Clear screen and move cursor to top-left
    print!("\x1B[2J\x1B[1;1H");
    io::Write::flush(&mut io::stdout()).unwrap();
}

