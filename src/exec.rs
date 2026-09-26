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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use nix::sys::wait::{waitpid, WaitStatus};

    fn run(path: &str, args: &[String], in_file: Option<&str>, out_file: Option<&str>) -> WaitStatus {
        let mut inputfd: RawFd = 0;
        let mut children: Vec<Pid> = Vec::new();
        execute_command(path, args, &mut inputfd, true, &mut children, in_file, out_file);
        waitpid(children[0], None).expect("waitpid failed")
    }

    #[test]
    fn output_redirect_creates_new_file() {
        let f = "test_out_new.txt";
        fs::remove_file(f).ok();

        run("/bin/echo", &["hello".to_string()], None, Some(f));

        let contents = fs::read_to_string(f).expect("file should have been created");
        assert_eq!(contents.trim(), "hello");
        fs::remove_file(f).ok();
    }

    #[test]
    fn output_redirect_overwrites_existing_file() {
        let f = "test_out_overwrite.txt";
        fs::write(f, "old content that must be gone").unwrap();

        run("/bin/echo", &["new_content".to_string()], None, Some(f));

        let contents = fs::read_to_string(f).unwrap();
        assert_eq!(contents.trim(), "new_content");
        fs::remove_file(f).ok();
    }

    #[test]
    fn output_redirect_has_correct_permissions() {
        let f = "test_out_perms.txt";
        fs::remove_file(f).ok();

        run("/bin/echo", &["x".to_string()], None, Some(f));

        let mode = fs::metadata(f).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "expected -rw------- (0600)");
        fs::remove_file(f).ok();
    }

    #[test]
    fn input_redirect_reads_from_file() {
        let in_f = "test_in_source.txt";
        let out_f = "test_in_result.txt";
        fs::write(in_f, "content from file\n").unwrap();
        fs::remove_file(out_f).ok();

        run("/bin/cat", &[], Some(in_f), Some(out_f));

        let contents = fs::read_to_string(out_f).unwrap();
        assert_eq!(contents, "content from file\n");
        fs::remove_file(in_f).ok();
        fs::remove_file(out_f).ok();
    }

    #[test]
    fn input_redirect_does_not_modify_source_file() {
        let in_f = "test_in_unmodified.txt";
        let original = "do not touch this content\n";
        fs::write(in_f, original).unwrap();

        run("/bin/cat", &[], Some(in_f), None);

        let after = fs::read_to_string(in_f).unwrap();
        assert_eq!(after, original, "input file must not be modified by the process");
        fs::remove_file(in_f).ok();
    }

    #[test]
    fn input_redirect_missing_file_signals_error() {
        let missing = "test_this_file_does_not_exist.txt";
        fs::remove_file(missing).ok(); // make sure it really doesn't exist

        let status = run("/bin/cat", &[], Some(missing), None);

        // the child must NOT have exited successfully -- it must signal an error,
        // not silently succeed as if nothing was wrong
        match status {
            WaitStatus::Exited(_, code) => assert_ne!(code, 0, "expected a non-zero/error exit for a missing input file"),
            WaitStatus::Signaled(_, _, _) => { /* also counts as an error signal */ }
            other => panic!("unexpected wait status: {:?}", other),
        }
    }

    #[test]
    fn input_redirect_non_regular_file_signals_error() {
        // a directory is not a regular file
        let status = run("/bin/cat", &[], Some("/tmp"), None);

        match status {
            WaitStatus::Exited(_, code) => assert_ne!(code, 0, "expected a non-zero/error exit when input is not a regular file"),
            WaitStatus::Signaled(_, _, _) => { /* also counts as an error signal */ }
            other => panic!("unexpected wait status: {:?}", other),
        }
    }

    #[test]
    fn combined_input_and_output_redirect() {
        let in_f = "test_combined_in.txt";
        let out_f = "test_combined_out.txt";
        fs::write(in_f, "roundtrip content\n").unwrap();
        fs::remove_file(out_f).ok();

        run("/bin/cat", &[], Some(in_f), Some(out_f));

        let contents = fs::read_to_string(out_f).unwrap();
        assert_eq!(contents, "roundtrip content\n");
        fs::remove_file(in_f).ok();
        fs::remove_file(out_f).ok();
    }
}