mod interpreter;
mod cli;
mod db;

fn main() {
    let mut database = db::database::Database::new();
    cli::cli(&mut database);
}
