use std::io;
mod ast;
mod table;
mod tokenizer;

pub fn cli() {
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
            println!("Goodbye!");
            break;
        }

        let tokens = tokenizer::tokenize(input);
        println!("{:?}", tokens);
        ast::generate(tokens);
    }
}
