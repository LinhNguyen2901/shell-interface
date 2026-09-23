use std::io;
use std::io::Write;
use std::env;

mod lexer;
mod parser;
mod env_expand;
mod exec;

mod tilde;
mod path_search;

fn main(){
    let user = env::var("USER").expect("USER environment variable must be set");
    let machine = env::var("MACHINE").expect("MACHINE environment variable must be set");
    let pwd = env::var("PWD").expect("PWD environment variable must be set");

    loop {
        print!("{user}@{machine}:{pwd}> ");
        io::stdout().flush().unwrap();

        let input = lexer::get_input();

        let tokens = lexer::get_tokens(&input);
        let mut expanded_tokens: Vec<String> = Vec::new();
        for t in &tokens {
            let expanded = env_expand::get_env(&tilde::expand_tilde(t));
            expanded_tokens.push(expanded);
        }

        let token_refs: Vec<&str> = expanded_tokens.iter().map(|s| s.as_str()).collect();

        for i in 0..token_refs.len() {
            print!("token {i}: {:?}\n", &token_refs.get(i).unwrap())
        }

        let cmd = parser::parse_tokens(token_refs);
        cmd.execute();
    }
}