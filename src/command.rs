use nix::{sys::wait::waitpid,unistd::Pid};
use std::os::fd::{RawFd};
use crate::builtins;
use crate::exec;
use crate::path_search;
use std::path::Path;

// Adapted from:
// https://www.cs.purdue.edu/homes/grr/SystemsProgrammingBook/Book/Chapter5-WritingYourOwnShell.pdf
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
    pub cmd_line: String,
    pub simple_commands: Vec<SimpleCommand>,
    pub background: bool,
}

impl Command {
    pub fn new() -> Self {
        Self {
            cmd_line: String::new(),
            simple_commands: Vec::new(),
            background: false,
        }
    }
    pub fn insert_simple_command(&mut self, simple_command: SimpleCommand) {
        // TODO: checking
        self.simple_commands.push(simple_command);
    }
    pub fn execute(&self, shell: &mut crate::shell::Shell) -> bool{
        if self.simple_commands.is_empty() { return false; }
        let mut inputfd: RawFd = 0;
        let mut children: Vec<Pid> = Vec::new();
        let len = self.simple_commands.len();

        // check every command and input file first, so nothing runs if one of them is bad
        let mut paths: Vec<String> = Vec::new();
        for cmd in &self.simple_commands {
            let name = &cmd.arguments[0];
            if builtins::is_builtin(name) {
                // built-ins don't live on disk, so there is no path to search for
                paths.push(String::new());
            } else {
                match path_search::find_command(name) {
                    Some(path) => paths.push(path),
                    None => {
                        println!("{}: command not found", name);
                        return false;
                    }
                }
            }
            if let Some(file) = &cmd.in_file {
                let p = Path::new(file);
                if !p.exists() {
                    println!("{}: No such file or directory", file);
                    return false;
                }
                if !p.is_file() {
                    println!("{}: Not a regular file", file);
                    return false;
                }
            }
        }

        for (i, cmd) in self.simple_commands.iter().enumerate() {
            let last = i == len - 1;
            let name = &cmd.arguments[0];
            inputfd = if builtins::is_builtin(name) {
                exec::execute_builtin(
                    name,
                    &cmd.arguments[1..],
                    inputfd,
                    last,
                    &mut children,
                    cmd.in_file.as_deref(),
                    cmd.out_file.as_deref(),
                    &shell.history,
                    &shell.job_list,
                )
            } else {
                exec::execute_command(
                    &paths[i],
                    &cmd.arguments[1..],
                    inputfd,
                    last,
                    &mut children,
                    cmd.in_file.as_deref(),
                    cmd.out_file.as_deref(),
                )
            };
        }
        if self.background {
            if let Some(last_pid) = children.last() {
                shell.jobs = shell.jobs + 1;
                println!("[{}] {}", shell.jobs, last_pid);
                let job = crate::shell::Job::new(shell.jobs, *last_pid, self.cmd_line.clone());
                shell.job_list.push(job);
            }
        } 
        else {
            for pid in children.iter() {
                waitpid(*pid, None).ok();
            }
        }
        true
    }
    
    pub fn clear(&mut self) {
        self.simple_commands.clear();
        self.background = false;
    }
}
