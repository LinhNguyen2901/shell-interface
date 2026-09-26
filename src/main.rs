use std::io;
use std::io::Write;
use crate::shell::Shell;

mod lexer;
mod command;
mod env_expand;
mod exec;
mod parser;
mod tilde;
mod path_search;
mod builtins;
mod shell;


fn main() {
    let mut shell = Shell::new();
    loop {
        shell.update_env();
        print!("{}", shell.prompt());
        io::stdout().flush().unwrap();

        shell.reap_background_processes();

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
            Ok(cmd) => {
                // add to run builtins
                if cmd.simple_commands.is_empty() {
                    continue;
                }
        
                let cmd_line = input.trim().to_string();
                let name = cmd.simple_commands[0].arguments[0].clone();
                let args = &cmd.simple_commands[0].arguments[1..];
        
                if builtins::is_builtin(&name) {
                    if name == "exit" {
                        builtins::exit(&shell.history, &shell.job_list);
                        break;
                    } else if name == "cd" {
                        if builtins::cd(args) {
                            shell.history.push(cmd_line);
                        }
                    } else if name == "jobs" {
                        builtins::jobs(&shell.job_list);
                        shell.history.push(cmd_line);
                    }
                } else if cmd.execute(&mut shell) {
                    shell.history.push(cmd_line);
                }
            }
            Err(err) => println!("{}", err),
        }
    }
}