use std::io;
mod ast;
mod table;
mod tokenizer;

pub fn cli() {
    println!("Welcome to the MollyDB CLI");
    let mut line_count = 1;
    let mut lines_printed = 0; // Start at 1 to account for welcome message

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
            let mut total_lines_to_clear = lines_printed + line_count - 1;
            if total_lines_to_clear > 1 {
                total_lines_to_clear += line_count-2;
            }
            for _ in 0..total_lines_to_clear {
                print!("\x1B[1A\x1B[2K");
            }
            print!("\r\x1B[2K");
            line_count = 1;
            lines_printed = 0;
            continue;
        }

        let tokens = tokenizer::tokenize(input);
        println!("{:?}", tokens);
        lines_printed += 1;
        let ast = ast::generate(tokens);
        for result in ast {
            println!("{:?}", result);
            lines_printed += 1;
        }
    }
}
