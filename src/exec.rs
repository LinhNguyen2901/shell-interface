use std::ffi::CString;
use nix::unistd::{fork, ForkResult, close, execv, dup2, pipe, Pid};
use nix::libc::_exit;
use std::os::fd::{RawFd, IntoRawFd};
use std::fs::{OpenOptions, Permissions};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::io::Write;
use crate::builtins;
use crate::shell;

pub fn execute_command(
    shell: &mut shell::Shell,
    path: &str,
    args: &[String],
    inputfd: RawFd,
    last: bool,
    children: &mut Vec<Pid>,
    in_file: Option<&str>,
    out_file: Option<&str>,
) -> Result<RawFd, String> {
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
    let mut outputfd: RawFd = 0;
    match unsafe{fork()} {
        Ok(ForkResult::Parent { child, .. }) => {
            children.push(child);
            if inputfd != 0 {
                close(inputfd).ok();
            }
            if !last && out_file.is_none() {
                close(pipefd.1).ok();
                outputfd = pipefd.0;
            }
        }
        Ok(ForkResult::Child) => {
            redirect_io(inputfd, pipefd, last, in_file, out_file);
            if builtins::is_builtin(&path) {
                match execute_builtin(shell, path, args) {
                    Ok(()) => {
                        std::io::stdout().flush().unwrap();
                        std::io::stderr().flush().unwrap();
                        unsafe { _exit(0) }
                    }
                    Err(e) => {
                        eprintln!("{}: {}", path, e);
                        std::io::stdout().flush().unwrap();
                        std::io::stderr().flush().unwrap();
                        unsafe { _exit(1) }
                    }
                }
            }
            // execv only returns if it failed (ex: file is not a real program)
            else if let Err(e) = execv(&c_path, &c_args) {
                eprintln!("{}: {}", path, e.desc());
                std::io::stderr().flush().unwrap();
                unsafe { _exit(127) }
            }
        }
        Err(_) => {return Err("Fork failed".to_string())}
    }
    Ok(outputfd)
}

pub fn execute_builtin(shell: &mut shell::Shell, name: &str, args: &[String]) -> Result<(), String>{
    if name == "exit" {
        builtins::exit(&shell.history, &shell.job_list);
        unsafe{ _exit(0); }
    } else if name == "cd" {
        match builtins::cd(args) {
            Ok(_) => (),
            Err(e) => {return Err(e);}
        }
    } else if name == "jobs" {
        builtins::jobs(&shell.job_list);
    }
    Ok(())
}

pub fn execute_builtin(
    name: &str,
    args: &[String],
    inputfd: RawFd,
    last: bool,
    children: &mut Vec<Pid>,
    in_file: Option<&str>,
    out_file: Option<&str>,
    history: &[String],
    job_list: &[Job],
) -> RawFd {
    let mut pipefd: (RawFd, RawFd) = (-1, -1);
    if !last && out_file.is_none() {
        let (i, o) = pipe().expect("Failed to create pipe.");
        pipefd = (i.into_raw_fd(), o.into_raw_fd());
    }
    let mut outputfd: RawFd = 0;
    match unsafe{fork()} {
        Ok(ForkResult::Parent { child, .. }) => {
            children.push(child);
            if inputfd != 0 {
                close(inputfd).ok();
            }
            if !last && out_file.is_none() {
                close(pipefd.1).ok();
                outputfd = pipefd.0;
            }
        }
        Ok(ForkResult::Child) => {
            redirect_io(inputfd, pipefd, last, in_file, out_file);
            match name {
                "cd" => { builtins::cd(args); }
                "jobs" => { builtins::jobs(job_list); }
                "exit" => { builtins::exit(history, job_list); }
                _ => {}
            }
            unsafe { _exit(0) }
        }
        Err(_) => println!("Fork failed"),
    }
    outputfd
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
                unsafe { _exit(1) }
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
                unsafe { _exit(1) }
            }
        };
        // mode() only applies to new files, so reset it when overwriting an old file
        file.set_permissions(Permissions::from_mode(0o600)).ok();
        let fd = file.into_raw_fd();
        dup2(fd, 1).expect("Failed to redirect stdout to file");
        close(fd).ok();
    } else if !last {
        close(pipefd.0).ok();
        dup2(pipefd.1, 1).expect("Failed to redirect stdout");
        close(pipefd.1).ok();
    }
}