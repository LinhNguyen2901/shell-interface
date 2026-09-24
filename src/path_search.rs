use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;

// Part 4: returns the full path of the command, or None if it can't be found
pub fn find_command(cmd: &str) -> Option<String> {
    if cmd.contains('/') {
        if is_executable(cmd) {
            return Some(cmd.to_string());
        }
        return None;
    }

    let path_var = match env::var("PATH") {
        Ok(p) => p,
        Err(_) => return None,
    };

    for dir in path_var.split(':') {
        // an empty entry in $PATH means the current directory
        let dir = if dir.is_empty() { "." } else { dir };
        let full_path = format!("{}/{}", dir, cmd);

        if is_executable(&full_path) {
            return Some(full_path);
        }
    }
    None
}

fn is_executable(path: &str) -> bool {
    match fs::metadata(path) {
        Ok(meta) => meta.is_file() && (meta.permissions().mode() & 0o111) != 0,
        Err(_) => false,
    }
}
