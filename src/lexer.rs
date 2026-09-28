use nix::unistd::read;


pub fn get_input() -> String {
    let mut input = String::new();
    let mut byte = [0u8; 1];
    loop {
        match read(0, &mut byte) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                input.push(byte[0] as char);
                if byte[0] == b'\n' {
                    break;
                }
            }
        }
    }
    return input;
}

// Referenced from:
// https://stackoverflow.com/questions/32257273/split-a-string-keeping-the-separators
pub fn get_tokens(input:&str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut last = 0;
    let is_separator = |c: char| {
        c.is_whitespace() || c == '|' || c == '<' || c == '>' || c == '&'
    };
    for (index, matched) in input.match_indices(is_separator) {
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