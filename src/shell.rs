use std::env;
use nix::{sys::wait::{waitpid, WaitPidFlag, WaitStatus},unistd::Pid};
pub struct Job {
    pub job_num: usize,
    pub pid: Pid,
    pub cmd_line: String,
}
impl Job {
    pub fn new(num: usize, pid: Pid, cmd: String) -> Self {
        Self {
            job_num: num,
            pid: pid,
            cmd_line: cmd
        }
    }
}
pub struct Shell {
    pub history: Vec<String>,
    pub job_list: Vec<Job>,
    pub user: String,
    pub machine: String,
    pub pwd: String,
    pub jobs: usize
}
impl Shell {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            job_list: Vec::new(),
            user: env::var("USER").expect("USER environment variable must be set"),
            machine: env::var("MACHINE").expect("MACHINE environment variable must be set"),
            pwd: env::var("PWD").expect("PWD environment variable must be set"),
            jobs: 0
        }
    }
    pub fn update_env(&mut self){
        self.user = env::var("USER").expect("USER environment variable must be set");
        self.machine = env::var("MACHINE").expect("MACHINE environment variable must be set");
        self.pwd = env::var("PWD").expect("PWD environment variable must be set");
    }
    pub fn prompt(&self) -> String {
        format!("{}@{}:{}> ", self.user, self.machine, self.pwd)
    }
    pub fn reap_background_processes(&mut self) {
        loop {
            match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::Exited(child_pid, _)) => {
                    if let Some(job) = self.job_list.iter().find(|j| j.pid == child_pid) {
                        println!("[{}]+ done {}", job.job_num, job.cmd_line);
                    }
                    self.job_list.retain(|job| job.pid != child_pid);                
                }
                Ok(WaitStatus::Signaled(child_pid, signal, _core_dump)) => {
                    println!("Child {} was killed by signal: {:?}", child_pid, signal);
                    self.job_list.retain(|job| job.pid != child_pid);
                }
                Ok(WaitStatus::StillAlive) | Err(nix::Error::ECHILD) => {
                    break;
                }
                Err(err) => {
                    eprintln!("Error waiting for child process: {:?}", err);
                    break;
                }
                _ => {
                    println!("Other state change occurred (Stopped/Continued).");
                }
            }
        }
    }
}// job struct
