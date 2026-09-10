use std::io;
use std::io::Write;
use std::env;

mod lexer;
mod parser;

fn main(){
    let user = env::var("USER").expect("USER environment variable must be set");
    let machine = env::var("MACHINE").expect("MACHINE environment variable must be set");
    let pwd = env::var("PWD").expect("PWD environment variable must be set");

    loop {
        print!("{user}@{machine}:{pwd}> ");
        io::stdout().flush().unwrap();

        let input = lexer::get_input();

        let tokens = lexer::get_tokens(&input); 
        for i in 0..tokens.len() {
            print!("token {i}: {:?}\n", &tokens.get(i).unwrap())
        }

        let cmd = parser::parse_tokens(tokens);
        cmd.execute();
    }
}