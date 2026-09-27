use std::env;

// Extra credit: SHLVL goes up by 1 every time our shell starts inside another shell
pub fn increase_shell_level() {
    let level: i32 = match env::var("SHLVL") {
        Ok(val) => val.parse().unwrap_or(0),
        Err(_) => 0,
    };
    unsafe {
        env::set_var("SHLVL", (level + 1).to_string());
    }
}
