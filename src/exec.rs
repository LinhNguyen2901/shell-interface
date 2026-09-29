use std::ffi::CString;
use nix::unistd::{fork, ForkResult, close, execv, dup, dup2, pipe, Pid};
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
    in_file: Option<&str>,
    out_file: Option<&str>,
) -> Result<(Pid, RawFd), String> {
    let c_path = CString::new(path).unwrap();
    let mut child_pid: Pid = Pid::this();
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
            child_pid = child;
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
                        eprintln!("{}", e);
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
    Ok((child_pid, outputfd))
}

pub fn execute_builtin(
    shell: &mut shell::Shell,
    name: &str,
    args: &[String],
) -> Result<(), String> {
    if name == "exit" {
        builtins::exit(&shell.history, &shell.job_list);
        unsafe{ _exit(0); }
    } else if name == "cd" {
        match builtins::cd(args) {
            Ok(_) => (),
            Err(e) => {return Err(e);}
        }
    } else if name == "jobs" {
        // a job may have finished while the user was typing
        shell.reap_background_processes();
        builtins::jobs(&shell.job_list);
    }
    Ok(())
}

// runs a builtin in the shell itself (so cd still works), but sends its output to out_file for the time it runs if there is one
pub fn run_builtin_redirected(
    shell: &mut shell::Shell,
    name: &str,
    args: &[String],
    in_file: Option<&str>,
    out_file: Option<&str>,
) -> Result<(), String> {
    if let Some(path) = in_file {
        let p = std::path::Path::new(path);
        if !p.exists() {
            return Err(format!("{}: No such file or directory", path));
        }
        if !p.is_file() {
            return Err(format!("{}: Not a regular file", path));
        }
    }
    let out_path = match out_file {
        Some(p) => p,
        None => return execute_builtin(shell, name, args),
    };

    let file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(out_path)
    {
        Ok(f) => f,
        Err(_) => return Err(format!("{}: could not open for writing", out_path)),
    };
    file.set_permissions(Permissions::from_mode(0o600)).ok();

    // save the real stdout so we can put it back after
    std::io::stdout().flush().ok();
    let saved = match dup(1) {
        Ok(fd) => fd,
        Err(_) => return Err("could not redirect output".to_string()),
    };
    let fd = file.into_raw_fd();
    dup2(fd, 1).ok();
    close(fd).ok();

    let result = execute_builtin(shell, name, args);

    std::io::stdout().flush().ok();
    dup2(saved, 1).ok();
    close(saved).ok();
    result
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