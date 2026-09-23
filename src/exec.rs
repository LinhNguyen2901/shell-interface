use nix::unistd::{fork, execv, ForkResult};
use nix::sys::wait::waitpid;
use std::ffi::CString;

pub fn execute_command(path: &str, args: &[String]) {
    let c_path = CString::new(path).unwrap();

    let mut c_args = Vec::new();
    c_args.push(c_path.clone());
    for arg in args {
        c_args.push(CString::new(arg.as_str()).unwrap());
    }

    let result = unsafe { fork() };

    match result {
        Ok(ForkResult::Child) => {
            execv(&c_path, &c_args).unwrap();
        }
        Ok(ForkResult::Parent { child }) => {
            waitpid(child, None).unwrap();
        }
        Err(_) => {
            println!("Fail to fork");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_ls_no_args() {
        execute_command("/bin/ls", &[]);
    }

    #[test]
    fn runs_echo_with_args() {
        execute_command("/bin/echo", &["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn runs_ls_with_flag() {
        execute_command("/bin/ls", &["-al".to_string()]);
    }
}