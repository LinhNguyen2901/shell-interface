use std::env;

// Part 3: only expand "~" by itself or a token that starts with "~/"
pub fn expand_tilde(token: &str) -> String {
    if token == "~" || token.starts_with("~/") {
        match env::var("HOME") {
            Ok(home) => return format!("{}{}", home, &token[1..]),
            Err(_) => return token.to_string(),
        }
    }
    token.to_string()
}
