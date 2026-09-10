// Adapted from https://www.cs.purdue.edu/homes/grr/SystemsProgrammingBook/Book/Chapter5-WritingYourOwnShell.pdf
#[derive(Debug)]
pub struct SimpleCommand {
    pub arguments: Vec<String>,
}
impl SimpleCommand {
    pub fn new() -> Self {
        Self {
            arguments: Vec::new(),
        }
    }
    pub fn new_with_command(cmd: &str) -> Self {
        Self {
            arguments: vec![cmd.to_string()],
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
    pub out_file: Option<String>,
    pub in_file: Option<String>,
    pub err_file: Option<String>,
    pub background: bool,
}

impl Command {
    pub fn new() -> Self {
        Self {
            simple_commands: Vec::new(),
            out_file: None,
            in_file: None,
            err_file: None,
            background: false,
        }
    }
    pub fn insert_simple_command(&mut self, simple_command: SimpleCommand) {
        // TODO: checking
        self.simple_commands.push(simple_command);
    }
    pub fn prompt(&self) {
        
    }
    pub fn print(&self) {
        
    }
    pub fn execute(&self) {
        // Temporary, only echo
        for cmd in &self.simple_commands {
            if cmd.arguments.first().map(|s| s.as_str()) == Some("echo") {
                if let Some(arg) = cmd.arguments.get(1) {
                    println!("{}", arg);
                }
            }
            else {
                println!("Command {:?} not found.", cmd);
            }
        }
    }
    pub fn clear(&mut self) {
        self.simple_commands.clear();
        self.out_file = None;
        self.in_file = None;
        self.err_file = None;
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
                Vocab::GREAT => {output.out_file = Some(tok.to_string()); last_token = Vocab::NONE;}
                Vocab::LESS => {output.in_file = Some(tok.to_string()); last_token = Vocab::NONE;}
                Vocab::NONE => println!("Error")
            }
        }
    }
    output
}