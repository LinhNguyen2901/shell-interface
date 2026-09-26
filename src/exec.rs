use std::ffi::CString;
use nix::unistd::{fork, ForkResult, close, execv, dup2, pipe, Pid};
use std::os::fd::{RawFd, IntoRawFd};
use std::fs::OpenOptions;
use std::os::unix::fs::OpenOptionsExt;

pub fn execute_command(
    path: &str,
    args: &[String],
    inputfd: &mut RawFd,
    last: bool,
    children: &mut Vec<Pid>,
    in_file: Option<&str>,
    out_file: Option<&str>,
) {
    let c_path = CString::new(path).unwrap();

    let mut c_args = Vec::new();
    c_args.push(c_path.clone());
    for arg in args {
        c_args.push(CString::new(arg.as_str()).unwrap());
    }

    let mut pipefd: (RawFd, RawFd) = (-1, -1);
    if !last && out_file.is_none() {
        let (i, o) = pipe().expect("Failed to create pipe.");
        pipefd = (i.into_raw_fd(), o.into_raw_fd());
    }

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            children.push(child);
            if *inputfd != 0 {
                close(*inputfd).ok();
            }
            if !last && out_file.is_none() {
                close(pipefd.1).ok();
                *inputfd = pipefd.0;
            }
        }
        Ok(ForkResult::Child) => {
            redirect_io(*inputfd, pipefd, last, in_file, out_file);
            execv(&c_path, &c_args).unwrap();
        }
        Err(_) => println!("Fork failed"),
    }
}

fn redirect_io(
    inputfd: RawFd,
    pipefd: (RawFd, RawFd),
    last: bool,
    in_file: Option<&str>,
    out_file: Option<&str>,
) {
    if let Some(path) = in_file {
        let file = match OpenOptions::new().read(true).open(path) {
            Ok(f) => f,
            Err(_) => {
                eprintln!("{}: No such file or directory", path);
                std::process::exit(1);
            }
        };
        let fd = file.into_raw_fd();
        dup2(fd, 0).expect("Failed to redirect stdin from file");
        close(fd).ok();
    } else if inputfd != 0 {
        dup2(inputfd, 0).expect("Failed to redirect stdin");
        close(inputfd).ok();
    }

    if let Some(path) = out_file {
        let file = match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)
        {
            Ok(f) => f,
            Err(_) => {
                eprintln!("{}: could not open for writing", path);
                std::process::exit(1);
            }
        };
        let fd = file.into_raw_fd();
        dup2(fd, 1).expect("Failed to redirect stdout to file");
        close(fd).ok();
    } else if !last {
        close(pipefd.0).ok();
        dup2(pipefd.1, 1).expect("Failed to redirect stdout");
        close(pipefd.1).ok();
    }
}