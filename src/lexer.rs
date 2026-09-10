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

pub fn get_tokens(input:&str) -> Vec<&str> {
    return input.split(' ').filter(|s| !s.is_empty()).collect();
}