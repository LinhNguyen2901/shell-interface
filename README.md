# Shell Interface (COP4610 Project 1)

**Group 8**

A Unix-like shell written in Rust. It reads a line of input, expands environment
variables and tildes, searches `$PATH`, and runs external and built-in commands
with I/O redirection, piping, and background processing.

## Table of Contents

1. [Group Members](#group-members)
2. [Division of Labor](#division-of-labor)
3. [File Listing](#file-listing)
4. [How to Compile and Run](#how-to-compile-and-run)
5. [Features Implemented](#features-implemented)
6. [Extra Credit](#extra-credit)
7. [Known Bugs and Limitations](#known-bugs-and-limitations)
8. [Development Log](#development-log)
9. [Group Meetings](#group-meetings)

## Group Members

| Name        | GitHub ID(s)                          |
| ----------- | ------------------------------------- |
| Jenny Jiang | JennyJiang / specificShark            |
| Linh Nguyen | LinhNguyen2901                        |
| Sid Nguyen  | hoangdung-nguyen / HoangDung Nguyen   |

## Division of Labor

| Part                          | Members                      |
| ----------------------------- | ---------------------------- |
| Part 1: Prompt                | Sid Nguyen                   |
| Part 2: Environment Variables | Linh Nguyen                  |
| Part 3: Tilde Expansion       | Jenny Jiang                  |
| Part 4: `$PATH` Search        | Jenny Jiang      |
| Part 5: External Commands     | Linh Nguyen     |
| Part 6: I/O Redirection       | Linh Nguyen     |
| Part 7: Piping                | Sid Nguyen |
| Part 8: Background Processing | Sid Nguyen|
| Part 9: Internal Commands     | Jenny Jiang|
| Extra Credit 1: Unlimited pipes            | Sid Nguyen|
| Extra Credit 2: Piping and I/O redirection | Linh Nguyen|
| Extra Credit 3: Shell-ception              | Jenny Jiang|
| README/Documentation              | Linh Nguyen|

## File Listing

```
shell-interface/
|-- src/
|   |-- main.rs         Entry point and main read-parse-execute loop
|   |-- lexer.rs        Reads a line of input and splits it into tokens
|   |-- parser.rs       Turns tokens into a Command (pipes, redirects, &)
|   |-- command.rs      Command and SimpleCommand types; runs a pipeline
|   |-- exec.rs         fork/execv, pipe setup, and file redirection
|   |-- env_expand.rs   Expands $VAR tokens using getenv
|   |-- tilde.rs        Expands ~ and ~/... to $HOME
|   |-- path_search.rs  Searches the directories in $PATH for a command
|   |-- builtins.rs     Built-in commands: exit, cd, jobs
|   |-- shell.rs        Shell state: prompt, history, job list, reaping
|   `-- shellception.rs Increments SHLVL when the shell starts (extra credit)
|-- .gitignore          Ignores build output (target/)
|-- Cargo.toml          Package manifest and dependencies
|-- Cargo.lock          Locked dependency versions
`-- README.md           This file
```

Because this is a Rust project, we follow the standard Cargo layout in place of
a Makefile, `include/`, and `bin/`, as the assignment allows.

## How to Compile and Run

Requirements: a Rust toolchain (already installed on linprog). The only
dependencies are the `libc` and `nix` crates, both allowed by the assignment.

### Compile

```
cargo build --release
```

The executable is created at `target/release/shell-interface`. Build output is
ignored by git, so no binaries are committed.

### Run

```
cargo run --release
```

or run the binary directly:

```
./target/release/shell-interface
```

The shell reads the `USER`, `MACHINE`, and `PWD` environment variables for its
prompt, so they must be set (they are on linprog). Type `exit` to quit.

### Prompt

```
USER@MACHINE:PWD>
```

Example: `mnguyen@linprog2.cs.fsu.edu:/home/grads/mnguyen>`

## Features Implemented

| Part | Feature                | Where                                       |
| ---- | ---------------------- | ------------------------------------------- |
| 1    | Prompt                 | `shell.rs` (`prompt`)                       |
| 2    | Environment variables  | `env_expand.rs`                             |
| 3    | Tilde expansion        | `tilde.rs`                                  |
| 4    | `$PATH` search         | `path_search.rs`                            |
| 5    | External commands      | `exec.rs`, `command.rs`                     |
| 6    | I/O redirection        | `exec.rs` (`redirect_io`), `parser.rs`      |
| 7    | Piping                 | `command.rs`, `exec.rs`                     |
| 8    | Background processing  | `command.rs`, `shell.rs`                    |
| 9    | Built-in commands      | `builtins.rs`, `main.rs`                    |


## Extra Credit

| Extra credit                                        | Points | Status      |
| --------------------------------------------------- | ------ | ----------- |
| 1. Unlimited number of pipes                        | 2      | Implemented |
| 2. Piping and I/O redirection in a single command   | 2      | Implemented |
| 3. Shell-ception                                    | 1      | Implemented |

### 1. Unlimited number of pipes (2 points)

This was implemented together with Part 7 (piping), led by Sid Nguyen with
Linh Nguyen. Instead of hard-coding one or two pipes, the parser and
`Command::execute` treat a pipeline as a list of commands of any length, so
there is no limit on the number of pipes. Each command is forked in turn, and
the read end of each pipe is passed on as the next command's stdin.

Example (four pipes):

```
echo hello world | tr a-z A-Z | rev | cat | wc -c
```

### 2. Piping and I/O redirection in a single command (2 points)

This was implemented together with Part 6 (I/O redirection), led by Linh Nguyen
with Sid Nguyen and Jenny Jiang. Each command in a pipeline keeps its own
optional input file and output file, so redirection can be combined with pipes
without any extra code. Input redirection is normally used on the first command
and output redirection on the last.

Examples:

```
cat < input.txt | tr a-z A-Z
ls | grep src > out.txt
cat < input.txt | sort | uniq > out.txt
```

### 3. Shell-ception (1 point)

The shell can be started from inside itself, repeatedly. On startup it
increments the `SHLVL` environment variable (`shellception.rs`), so each nested
shell knows its depth. Each nested shell has its own history and job list, and
`exit` returns to the parent shell.

Example:

```
./target/release/shell-interface
./target/release/shell-interface
echo $SHLVL
exit
exit
```

## Known Bugs and Limitations

- The shell requires `USER`, `MACHINE`, and `PWD` to be set; otherwise it
  stops with an error at startup.
- For background pipelines only the last command's PID is tracked as the job.
- An input file that exists but is not a regular file (for example a
  directory) is not rejected before the command runs.
- Quotes, escaped characters, globs, and regular expressions are not handled,
  as allowed by the assignment. Only whole-token `$VAR` arguments are expanded.

## Development Log
### Jenny Jiang

| Date       | Work done                                                        |
| ---------- | ---------------------------------------------------------------- |
| 2026-09-23 | Added tilde expansion and `$PATH` search integration (PR #2)     |
| 2026-09-25 | Added built-in commands `cd`, `jobs`, and `exit` (PR #5)         |
| 2026-09-26 | Added the shell-ception extra credit (PR #11)                    |

### Linh Nguyen

| Date       | Work done                                                        |
| ---------- | ---------------------------------------------------------------- |
| 2026-09-22 | Environment variable expansion, lexer fix, external command execution with fork/execv                                |
| 2026-09-25 | Implemented I/O redirection, fixed a child path bug, added and implemented extra credit 2                                  |
| 2026-09-26 | Fixed a parser bug, fixed bugs and added missing requirements,   |
|            | finished piping with I/O redirection                     |
| 2026-09-27 | Wrote the README                                                 |

### Sid Nguyen

| Date       | Work done                                                        |
| ---------- | ---------------------------------------------------------------- |
| 2026-09-10 | Created the repository and `.gitignore`; implemented the parser  |
| 2026-09-24 | Started piping                                                   |
| 2026-09-25 | Finished piping (PR #4); started background processing (PR #6)   |
| 2026-09-26 | Finished background processing (PR #8); integrated I/O           |
|            | redirection with background jobs and fixed a parsing bug (PR #9) |

## Group Meetings

All three members attended every meeting.

### Meeting 1: Sept 10, 2026 (in person)

We met to plan the project and divide the work. We went through each part of
the assignment and decided who would take which parts (see Division of Labor
above). The assignments were fairly firm from this point on, and we submitted
the division of labor document on Canvas.

### Meeting 2: week of Sept 21, 2026 (in person)

We sat together to review our progress and talk through any blockers or
questions. We realized that Parts 6 to 8 (I/O redirection, piping, and
background processing) depend on Part 5 (external command execution), so we
agreed to finish Part 5 first, along with Parts 1 to 4, before continuing.

### Meeting 3: Friday, Sept 25, 2026

We met again to catch up and make sure everyone was on track. We discussed the
timeline for the remaining work and how we would test the shell over the
weekend before the Monday, Sept 28 deadline.