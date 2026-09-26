use std::io;
use std::io::Write;
use std::env;
use nix::{sys::wait::{waitpid, WaitPidFlag, WaitStatus},unistd::Pid};

mod lexer;
mod command;
mod env_expand;
mod exec;
mod parser;
mod tilde;
mod path_search;

pub struct Shell {
    pub background_pid: Vec<Pid>,
    pub user: String,
    pub machine: String,
    pub pwd: String
}
impl Shell {
    pub fn new() -> Self {
        Self {
            background_pid: Vec::new(),
            user: env::var("USER").expect("USER environment variable must be set"),
            machine: env::var("MACHINE").expect("MACHINE environment variable must be set"),
            pwd: env::var("PWD").expect("PWD environment variable must be set"),
        }
    }
    pub fn update_env(&mut self){
        self.user = env::var("USER").expect("USER environment variable must be set");
        self.machine = env::var("MACHINE").expect("MACHINE environment variable must be set");
        self.pwd = env::var("PWD").expect("PWD environment variable must be set");
    }
    pub fn prompt(&self) -> String {
        format!("{}@{}:{}> ", self.user, self.machine, self.pwd)
    }
}
fn main(){
    let mut shell = Shell::new();
    loop {
        shell.update_env();
        print!("{}", shell.prompt());
        io::stdout().flush().unwrap();

        for pid in shell.background_pid.iter() {
            waitpid(*pid, Some(WaitPidFlag::WNOHANG)).ok();
        }

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

        match parser::parse_tokens(token_refs){
            Ok(cmd) => cmd.execute(),
            Err(err) => println!("{}", err),
        }
        
    }
}