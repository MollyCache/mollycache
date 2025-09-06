use mollydb::*;

fn main() {
    let mut database = db::database::Database::new();
    cli::cli(&mut database);
}
