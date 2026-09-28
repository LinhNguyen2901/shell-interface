use crate::command::{Command, SimpleCommand};
use std::io;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Vocab {
    NONE,
    CMD,
    WORD,
    PIPE,
    GREAT,
    LESS,
    REDIR_WORD
}

pub fn get_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    return input;
}

// Referenced from https://stackoverflow.com/questions/32257273/split-a-string-keeping-the-separators
pub fn get_tokens(input:&str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut last = 0;
    for (index, matched) in input.match_indices(|c: char| c.is_whitespace() || c == '|' || c == '<' || c == '>' || c == '&'){
        if last != index {
            result.push(&input[last..index]);
        }
        if !matched.chars().all(char::is_whitespace) {
            result.push(matched);
        }
        last = index + matched.len()
    }
    if last < input.len() {
        result.push(&input[last..]);
    }
    return result;
}

pub fn parse_tokens(tokens: Vec<&str>) -> Result<Command, String> {
    let mut last_token = Vocab::PIPE;
    let mut output = Command::new();
    output.cmd_line = tokens.join(" ");

    for tok in tokens {
        // println!("{:?}", last_token);
        match tok {
            "|" => match last_token {
                Vocab::CMD | Vocab::WORD | Vocab::REDIR_WORD => {last_token = Vocab::PIPE;}
                _ => return Err("Unexpected |".to_string()),
            }
            ">" => match last_token {
                Vocab::CMD | Vocab::WORD | Vocab::REDIR_WORD => {last_token = Vocab::GREAT;}
                _ => return Err("Unexpected >".to_string()),
            }
            "<" => match last_token {
                Vocab::CMD | Vocab::WORD | Vocab::REDIR_WORD => {last_token = Vocab::LESS;}
                _ => return Err("Unexpected <".to_string()),
            }
            "&" => {
                output.background = true;
                match last_token {
                    Vocab::CMD | Vocab::WORD => {last_token = Vocab::NONE;}
                    _ => return Err("Unexpected &".to_string()),
                }
            }
            word => match last_token{
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
                    last_token = Vocab::REDIR_WORD;}
                Vocab::LESS => {if let Some(last_cmd) = output.simple_commands.last_mut() {
                        last_cmd.in_file = Some(tok.to_string());
                    }
                    last_token = Vocab::REDIR_WORD;}
                Vocab::NONE | Vocab::REDIR_WORD => return Err(format!("Unexpected syntax: {word}")),
            }
        }
    }
    if matches!(last_token, Vocab::PIPE | Vocab::GREAT | Vocab::LESS) {
        return Err("Incomplete command".to_string());
    }
    Ok(output)
}