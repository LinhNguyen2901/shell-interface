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
mod builtins;
mod jobs;


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
    let user = env::var("USER").expect("USER environment variable must be set");
    let machine = env::var("MACHINE").expect("MACHINE environment variable must be set");

    let mut history: Vec<String> = Vec::new();
    let mut job_list: Vec<jobs::Job> = Vec::new();

    loop {
        let pwd = env::var("PWD").expect("PWD environment variable must be set");
        print!("{user}@{machine}:{pwd}> ");
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

        let cmd = parser::parse_tokens(token_refs);
        

        // add to run builtins
        if cmd.simple_commands.is_empty() {
            continue;
        }
 
        let cmd_line = input.trim().to_string();
        let name = cmd.simple_commands[0].arguments[0].clone();
        let args = &cmd.simple_commands[0].arguments[1..];
 
        if builtins::is_builtin(&name) {
            if name == "exit" {
                builtins::exit(&history, &job_list);
                break;
            } else if name == "cd" {
                if builtins::cd(args) {
                    history.push(cmd_line);
                }
            } else if name == "jobs" {
                builtins::jobs(&job_list);
                history.push(cmd_line);
            }
        } else if cmd.execute() {
            history.push(cmd_line);
        match parser::parse_tokens(token_refs){
            Ok(cmd) => cmd.execute(),
            Err(err) => println!("{}", err),
        }
        
    }
}