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
mod shellception;


fn main() {
    shellception::increase_shell_level();
    let mut shell = Shell::new();
    loop {
        // check finished jobs before the prompt so "done" isn't printed after it
        shell.reap_background_processes();

        shell.update_env();
        print!("{}", shell.prompt());
        io::stdout().flush().unwrap();
        io::stderr().flush().unwrap();

        let input = lexer::get_input();
        // empty string means end of input (Ctrl+D or end of a file), so quit like exit
        if input.is_empty() {
            println!();
            let _ = builtins::exit(&shell.history, &shell.job_list);
            break;
        }

        let tokens = lexer::get_tokens(&input);
        let mut expanded_tokens: Vec<String> = Vec::new();
        for t in &tokens {
            let expanded = env_expand::get_env(&tilde::expand_tilde(t));
            expanded_tokens.push(expanded);
        }

        let token_refs: Vec<&str> = expanded_tokens.iter().map(|s| s.as_str()).collect();
        if token_refs.is_empty() {
            continue;
        }

        match parser::parse_tokens(token_refs){
            Ok(cmd) => {
                // add to run builtins
                if cmd.simple_commands.is_empty() { continue; }
                if cmd.simple_commands.len() == 1 && builtins::is_builtin(&cmd.simple_commands[0].arguments[0]){
                    let cmd_line = input.trim().to_string();
                    let name = cmd.simple_commands[0].arguments[0].clone();
                    let args = &cmd.simple_commands[0].arguments[1..];
            
                    if builtins::is_builtin(&name) {
                        match exec::execute_builtin(&mut shell, &name, &args) {
                            Ok(_) => shell.history.push(cmd_line),
                            Err(e) => eprintln!("{}", e),
                        }
                    }
                }
                else {
                    match cmd.execute(&mut shell) {
                        Ok(_) => shell.history.push(input.trim().to_string()),
                        Err(e) => eprintln!("{}", e)
                    }
                }
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}