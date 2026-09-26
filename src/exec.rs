use std::ffi::CString;
use nix::{unistd::{fork, ForkResult, close, execv, dup2, pipe, Pid}};
use std::os::fd::{RawFd, IntoRawFd};

pub fn execute_command(path: &str, args: &[String], inputfd: RawFd, last: bool, children: &mut Vec<Pid>) -> RawFd {
    let c_path = CString::new(path).unwrap();

    let mut c_args = Vec::new();
    c_args.push(c_path.clone());
    for arg in args {
        c_args.push(CString::new(arg.as_str()).unwrap());
    }

    let mut pipefd: (RawFd, RawFd) = (-1, -1);
    if !last {
        println!("balls");
        let (i, o) = pipe().expect("Failed to create pipe.");
        pipefd = (i.into_raw_fd(), o.into_raw_fd())
    }
    let mut outputfd: RawFd = 0;
    match unsafe{fork()} {
        Ok(ForkResult::Parent { child, .. }) => {
            children.push(child);
            if inputfd != 0 {
                close(inputfd).ok();
            }
            if !last {
                close(pipefd.1).ok();
                outputfd = pipefd.0;
            }
        }
        Ok(ForkResult::Child) => {
            redirect_io(inputfd, pipefd, last);
            execv(&c_path, &c_args).unwrap();
        }
        Err(_) => println!("Fork failed"),
    }
    outputfd
}


fn redirect_io(inputfd: RawFd, pipefd: (RawFd, RawFd), last: bool) {
    if inputfd != 0 {
        dup2(inputfd, 0).expect("Failed to redirect stdin");
        close(inputfd).ok();
    }

    if !last {
        close(pipefd.0).ok();
        dup2(pipefd.1, 1).expect("Failed to redirect stdout");
        close(pipefd.1).ok();
    }
}