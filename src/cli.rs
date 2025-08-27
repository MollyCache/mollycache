use std::io;

pub fn cli() {
    println!("Welcome to the MollyDB CLI");
    
    loop {
        print!("> ");
        
        io::Write::flush(&mut io::stdout()).expect("Failed to flush stdout.");
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("quit") {
            println!("Goodbye!");
            break;
        }

        println!("You said: {}", input);
    }
}