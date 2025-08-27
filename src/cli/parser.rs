pub fn parse(input: &str){
    let tokens = tokenize(input, 1);
    println!("{:?}", tokens);
}

#[derive(Debug)]
#[derive(PartialEq)]
enum TokenTypes {
    CREATE,
    SELECT,
    INSERT,
    TABLE,
    FROM,
    INTO,
    VALUES,
    ASTERIX,
    SEMICOLON,
    VALUE,
    NONE,
}

#[derive(Debug)]
#[derive(PartialEq)]
struct Token<'a> {
    token_type: TokenTypes,
    value: &'a str,
    col_num: usize,
    line_num: usize
}

fn tokenize<'a>(line: &'a str, line_num: usize) -> Vec<Token<'a>> {
    let mut tokens : Vec<Token<'a>> = vec![];
    let mut start: usize = 0;
    let mut inside_quotation = false;
    for (i, ch) in line.char_indices(){
        let cur_value = &line[start..=i];
        let cur_type = match cur_value {
            _ if cur_value.eq_ignore_ascii_case("CREATE") => TokenTypes::CREATE,
            _ if cur_value.eq_ignore_ascii_case("SELECT") => TokenTypes::SELECT,
            _ if cur_value.eq_ignore_ascii_case("INSERT") => TokenTypes::INSERT,
            _ if cur_value.eq_ignore_ascii_case("TABLE") => TokenTypes::TABLE,
            _ if cur_value.eq_ignore_ascii_case("FROM") => TokenTypes::FROM,
            _ if cur_value.eq_ignore_ascii_case("INTO") => TokenTypes::INTO,
            _ if cur_value.eq_ignore_ascii_case("VALUES") => TokenTypes::VALUES,
            _ if ch == '*' => TokenTypes::ASTERIX,
            _ if ch == ';' => TokenTypes::SEMICOLON,
            _ if ch == '"' => TokenTypes::VALUE,
            _ => TokenTypes::NONE
        };

        if ch == ' ' && !inside_quotation {
            start = i + 1;
            continue;
        }

        match cur_type {
            TokenTypes::VALUE => {
                if inside_quotation {
                    tokens.push(Token {
                        token_type: cur_type,
                        value: &line[start..=i],
                        col_num: start,
                        line_num: line_num
                    });
                    start = i+1;
                    inside_quotation = false;
                }
                else{
                    inside_quotation = true;
                }                
            }
            TokenTypes::NONE => {}
            _ => {
                tokens.push(Token { 
                    token_type: cur_type, 
                    value: &line[start..=i], 
                    col_num: start, 
                    line_num: line_num 
                });
                start = i+1;
            }
        }

    }
    return tokens;    
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_parses_statement() {
        let line = "SELECT * FROM \"users\";";
        let result = tokenize(line, 1);
        let expected_tokens = vec![Token {
            token_type: TokenTypes::SELECT,
            value: "SELECT",
            col_num: 0,
            line_num: 1 
        }, Token{
            token_type: TokenTypes::ASTERIX,
            value: "*",
            col_num: 7,
            line_num: 1
        }, Token{
            token_type: TokenTypes::FROM,
            value: "FROM",
            col_num: 9,
            line_num: 1
        }, Token{
            token_type: TokenTypes::VALUE,
            value: "\"users\"",
            col_num: 14,
            line_num: 1
        }, Token{
            token_type: TokenTypes::SEMICOLON,
            value: ";",
            col_num: 21,
            line_num: 1
        },];
        assert_eq!(expected_tokens, result);   
    }
}