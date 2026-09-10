use std::io;
use std::io::Write;

fn main(){
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = get_input();
        print!("whole input: {}\n", input);

        let tokens = get_tokens(&input); 
        for i in 0..tokens.len() {
            print!("token {i}: {:?}\n", &tokens.get(i).unwrap())
        }
    }
}

pub fn get_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    return input;
}

// Referenced from https://stackoverflow.com/questions/32257273/split-a-string-keeping-the-separators
pub fn get_tokens(input:&str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut last = 0;
    for (index, matched) in input.match_indices(|c: char| !(c.is_alphanumeric() || c == '\\')){
        if last != index {
            result.push(&input[last..index]);
        }
        if !matched.chars().all(char::is_whitespace) {
            result.push(matched);
        }
        last = index + matched.len()
    }
    if last < input.len() {
        result.push(&input[last..]);
    }
    return result;
}