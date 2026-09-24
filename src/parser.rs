use nix::{sys::wait::waitpid,unistd::{fork, ForkResult, write, close, execvp, dup2, pipe, Pid}};
use std::os::fd::{AsRawFd, RawFd, OwnedFd};
use std::ffi::CString;

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
            let mut pipefd: (RawFd, RawFd) = (-1, -1);
            if !last {
                let (i, o) = pipe().expect("Failed to create pipe.");
                pipefd = (i.as_raw_fd(), o.as_raw_fd())
            }
            match unsafe{fork()} {
                Ok(ForkResult::Parent { child, .. }) => {
                    children.push(child);
                    if inputfd != 0 {
                        close(inputfd).ok();
                    }
                    if !last {
                        close(pipefd.1).ok();
                        
                    }
                }
                Ok(ForkResult::Child) => {
                    write(std::io::stdout(), "I'm a new child process\n".as_bytes()).ok();
                    if inputfd != 0 {
                        dup2(inputfd, 0).expect("Failed to redirect stdin");
                        close(inputfd).ok();
                    }
                    if !last {
                        close(pipefd.0).ok();
                        dup2(pipefd.1, 1).expect("Failed to redirect stdout");
                        close(pipefd.1).ok();
                    }
                    let program_cstr = CString::new(cmd.arguments[0].as_str()).unwrap();
                    let args_cstrs: Vec<CString> = cmd.arguments.iter().map(|s| CString::new(s.as_str()).unwrap()).collect();
                    execvp(&program_cstr, &args_cstrs).expect("Exec failed");
                    unsafe { libc::_exit(0) };
                }
                Err(_) => println!("Fork failed"),
            }
            for pid in children {
                let _ = waitpid(pid, None);
            }

            // if cmd.arguments.first().map(|s| s.as_str()) == Some("echo") {
            //     if let Some(arg) = cmd.arguments.get(1) {
            //         println!("{}", arg);
            //     }
            // }
            // else {
            //     println!("Command {:?} not found.", cmd);
            // }
        }
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