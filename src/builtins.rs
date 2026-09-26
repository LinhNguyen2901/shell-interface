use std::env;
use std::path::Path;
use nix::sys::wait::waitpid;

use crate::shell::Job;

pub fn is_builtin(name: &str) -> bool {
    name == "exit" || name == "cd" || name == "jobs"
}

// returns true if cd worked -> main knows it is a valid command
pub fn cd(args: &[String]) -> bool {
    if args.len() > 1 {
        println!("cd: too many arguments");
        return false;
    }

    let target = if args.len() == 0 {
        match env::var("HOME") {
            Ok(home) => home,
            Err(_) => {
                println!("cd: HOME not set");
                return false;
            }
        }
    } else {
        args[0].clone()
    };

    let path = Path::new(&target);
    if !path.exists() {
        println!("cd: {}: No such file or directory", target);
        return false;
    }
    if !path.is_dir() {
        println!("cd: {}: Not a directory", target);
        return false;
    }

    if env::set_current_dir(path).is_err() {
        println!("cd: {}: could not change directory", target);
        return false;
    }

    // update $PWD so the prompt shows the new directory
    if let Ok(new_dir) = env::current_dir() {
        unsafe {
            env::set_var("PWD", new_dir);
        }
    }
    true
}

pub fn jobs(job_list: &Vec<Job>) {
    if job_list.is_empty() {
        println!("No active background processes");
        return;
    }
    for job in job_list {
        println!("[{}]+ {} {}", job.job_num, job.pid, job.cmd_line);
    }
}

pub fn exit(history: &Vec<String>, job_list: &Vec<Job>) {
    // have to wait for background jobs before quitting
    for job in job_list {
        let _ = waitpid(job.pid, None);
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
