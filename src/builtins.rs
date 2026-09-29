use std::env;
use std::path::Path;
use nix::sys::wait::waitpid;

use crate::shell::Job;

pub fn is_builtin(name: &str) -> bool {
    name == "exit" || name == "cd" || name == "jobs"
}

// returns true if cd worked -> main knows it is a valid command
pub fn cd(args: &[String]) -> Result<(), String> {
    if args.len() > 1 {
        return Err("cd: too many arguments".to_string());
    }

    let target = if args.len() == 0 {
        match env::var("HOME") {
            Ok(home) => home,
            Err(_) => {
                return Err("cd: HOME not set".to_string());
            }
        }
    } else {
        args[0].clone()
    };

    let path = Path::new(&target);
    if !path.exists() {
        return Err(format!("cd: {}: No such file or directory", target));
    }
    if !path.is_dir() {
        return Err(format!("cd: {}: Not a directory", target));
    }

    if env::set_current_dir(path).is_err() {
        return Err(format!("cd: {}: could not change directory", target));
    }

    // update $PWD so the prompt shows the new directory
    if let Ok(new_dir) = env::current_dir() {
        unsafe {
            env::set_var("PWD", new_dir);
        }
    }
    Ok(())
}

pub fn jobs(job_list: &[Job]) {
    if job_list.is_empty() {
        println!("No active background processes");
    }
    for job in job_list {
        println!("[{}]+ {} {}", job.job_num, job.pid, job.cmd_line);
    }
}

pub fn exit(history: &[String], job_list: &[Job]) {
    // have to wait for background jobs before quitting
    for job in job_list {
        let _ = waitpid(job.pid, None);
        println!("[{}]+ done {}", job.job_num, job.cmd_line);
    }

    if history.is_empty() {
        println!("No valid commands");
    } else if history.len() < 3 {
        println!("Last valid command:");
        println!("{}", history[history.len() - 1]);
    } else {
        println!("Last three valid commands:");
        for i in (history.len() - 3)..history.len() {
            println!("{}", history[i]);
        }
    }
}
