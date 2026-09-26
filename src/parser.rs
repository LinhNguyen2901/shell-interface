use nix::{sys::wait::waitpid,unistd::Pid};
use std::os::fd::{RawFd};
use crate::exec;
use crate::path_search;

// Adapted from https://www.cs.purdue.edu/homes/grr/SystemsProgrammingBook/Book/Chapter5-WritingYourOwnShell.pdf
#[derive(Debug)]
pub struct SimpleCommand {
    pub arguments: Vec<String>,
    pub out_file: Option<String>,
    pub in_file: Option<String>,
    pub err_file: Option<String>,
}
impl SimpleCommand {
    pub fn new() -> Self {
        Self {
            arguments: Vec::new(),
            out_file: None,
            in_file: None,
            err_file: None,
        }
    }
    pub fn new_with_command(cmd: &str) -> Self {
        Self {
            arguments: vec![cmd.to_string()],
            out_file: None,
            in_file: None,
            err_file: None,
        }
    }
    pub fn insert_argument(&mut self, argument: String) {
        // TODO: checking
        self.arguments.push(argument);
    }
}
#[derive(Debug)]
pub struct Command {
    pub simple_commands: Vec<SimpleCommand>,
    pub background: bool,
}

impl Command {
    pub fn new() -> Self {
        Self {
            simple_commands: Vec::new(),
            background: false,
        }
    }
    pub fn insert_simple_command(&mut self, simple_command: SimpleCommand) {
        // TODO: checking
        self.simple_commands.push(simple_command);
    }
    pub fn execute(&self) -> bool {
        if self.simple_commands.is_empty() { return false; }
        let mut inputfd: RawFd = 0;
        let mut children: Vec<Pid> = Vec::new();
        let len = self.simple_commands.len();
        let mut valid = true; 

        for (i, cmd) in self.simple_commands.iter().enumerate() {
            let last = i == len - 1;
            let name = &cmd.arguments[0];
            match path_search::find_command(name) {
                Some(path) => exec::execute_command(&path, &cmd.arguments[1..], &mut inputfd, last, &mut children, cmd.in_file.as_deref(), cmd.out_file.as_deref()),
                None => {
                    println!("{}: command not found", name);
                    valid = false;
                }
            }
        }
        if !self.background {
            for pid in children.iter() {
                waitpid(*pid, None).ok();
            }
        }
        valid
    }
    
    pub fn clear(&mut self) {
        self.simple_commands.clear();
        self.background = false;
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Vocab {
    NONE,
    CMD,
    WORD,
    PIPE,
    GREAT,
    LESS
}

pub fn parse_tokens(tokens: Vec<&str>) -> Command {
    let mut last_token = Vocab::PIPE;
    let mut output = Command::new();

    for tok in tokens {
        if tok == "|" {
            last_token = Vocab::PIPE;
        }
        else if tok == ">" {
            last_token = Vocab::GREAT;
        }
        else if tok == "<" {
            last_token = Vocab::LESS;
        }
        else if tok == "&" {
            output.background = true;
            last_token = Vocab::NONE;
        }
        else {
            match last_token{
                Vocab::CMD | Vocab::WORD => {
                    if let Some(last_cmd) = output.simple_commands.last_mut() {
                        last_cmd.insert_argument(tok.to_string());
                    } 
                    last_token = Vocab::WORD;
                },
                Vocab::PIPE => {output.insert_simple_command(SimpleCommand::new_with_command(tok)); last_token = Vocab::CMD;},
                Vocab::GREAT => {if let Some(last_cmd) = output.simple_commands.last_mut() {
                        last_cmd.out_file = Some(tok.to_string());
                    } 
                    last_token = Vocab::NONE;}
                Vocab::LESS => {if let Some(last_cmd) = output.simple_commands.last_mut() {
                        last_cmd.in_file = Some(tok.to_string());
                    } 
                    last_token = Vocab::NONE;}
                Vocab::NONE => println!("Error")
            }
        }
    }
    output
}