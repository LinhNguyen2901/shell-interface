use std::env;

pub fn get_env(token: &str) -> String {
    if token.starts_with("$") && token.len() > 1 {
        let var_name = &token[1..];
        match env::var(var_name) {
            Ok(value) => value,
            Err(_) => token.to_string(),
        }
    } else {
        token.to_string()
    }
}
