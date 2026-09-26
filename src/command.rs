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
    pub fn execute(&self) {
        if self.simple_commands.is_empty() { return; }
        let mut inputfd: RawFd = 0;
        let mut children: Vec<Pid> = Vec::new();
        let len = self.simple_commands.len();

        for (i, cmd) in self.simple_commands.iter().enumerate() {
            let last = i == len - 1;
            let name = &cmd.arguments[0];
            match path_search::find_command(name) {
                Some(path) => inputfd = exec::execute_command(&path, &cmd.arguments[1..], inputfd, last, &mut children),
                None => println!("{}: command not found", name),
            }
        }
        if !self.background {
            for pid in children.iter() {
                waitpid(*pid, None).ok();
            }
        }
    }
    
    pub fn clear(&mut self) {
        self.simple_commands.clear();
        self.background = false;
    }
}
