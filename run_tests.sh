#!/bin/bash
# Automated tests for Project 1 shell
# usage: bash tests/run_tests.sh [path/to/shell]   (default: ./bin/shell)

SHELL_BIN=$(realpath "${1:-./bin/shell}")
if [ ! -x "$SHELL_BIN" ]; then
    echo "shell not found at $SHELL_BIN, build it first"
    exit 1
fi

OLDPWD_SAVE=$(pwd)
T=$(mktemp -d)
mkdir -p "$T/home/dir1" "$T/adir"
printf 'hello world\n' > "$T/in.txt"
printf 'c\nb\na\n' > "$T/abc.txt"
printf 'not a program\n' > "$T/fake"
chmod +x "$T/fake"
printf 'echo inner $SHLVL\n' > "$T/inner.txt"
printf 'secret\n' > "$T/noread.txt"
chmod 000 "$T/noread.txt"
cd "$T" || exit 1

PASS=0
FAIL=0

# runs the shell with the given input, output goes into $OUT
run_shell() {
    OUT=$(printf '%b' "$1" | env USER=tester MACHINE=testmachine HOME="$T/home" PWD="$T" \
        SHLVL=1 PATH="$PATH" timeout 15 "$SHELL_BIN" 2>&1)
}

report() {
    if [ "$1" = "ok" ]; then
        PASS=$((PASS + 1))
        echo "PASS  $2"
    else
        FAIL=$((FAIL + 1))
        echo "FAIL  $2"
        echo "      expected: $3"
        echo "$OUT" | sed 's/^/      | /' | head -15
    fi
}

# how many seconds the shell took for this input
time_shell() {
    local start end
    start=$(date +%s)
    run_shell "$1"
    end=$(date +%s)
    ELAPSED=$((end - start))
}

# output should contain the text
expect() {
    run_shell "$2"
    if grep -qF -- "$3" <<< "$OUT"; then report ok "$1"; else report fail "$1" "$3"; fi
}

# output should NOT contain the text
expect_not() {
    run_shell "$2"
    if grep -qF -- "$3" <<< "$OUT"; then report fail "$1" "no '$3'"; else report ok "$1"; fi
}

echo "== Part 1: prompt =="
expect "prompt is USER@MACHINE:PWD>" 'exit\n' "tester@testmachine:$T> "

echo "== Part 2: environment variables =="
expect "echo \$USER" 'echo $USER\n' "tester"
expect "echo \$HOME" 'echo $HOME\n' "$T/home"
expect "two variables in one command" 'echo $USER $MACHINE\n' "tester testmachine"

echo "== Part 3: tilde expansion =="
expect "~ alone" 'echo ~\n' "$T/home"
expect "~/dir1" 'echo ~/dir1\n' "$T/home/dir1"
expect "ls ~ shows dir1" 'ls ~\n' "dir1"
expect "~abc is not expanded" 'echo ~abc\n' "~abc"
expect "a~ is not expanded" 'echo a~\n' "a~"

echo "== Part 4: \$PATH search =="
expect "ls found in PATH" 'ls\n' "abc.txt"
expect "command not found" 'nosuchcmd\n' "command not found"
expect "absolute path /bin/echo" '/bin/echo abs\n' "abs"
expect "bad path with slash" './nope\n' "not found"

echo "== Part 5: external commands =="
expect "command with args (ls -a)" 'ls -a\n' ".."
expect "command with many args" 'echo a b c d\n' "a b c d"
expect_not "non-program file does not crash the shell" './fake\n' "panicked"
expect "shell keeps going after bad program" './fake\necho still alive\n' "still alive"

echo "== Part 6: I/O redirection =="
expect "cmd < file" 'cat < in.txt\n' "hello world"
run_shell 'echo out test > out1.txt\n'
if [ "$(cat out1.txt 2>/dev/null)" = "out test" ]; then report ok "cmd > file creates file"; else report fail "cmd > file creates file" "out1.txt = 'out test'"; fi
if [ "$(stat -c '%a' out1.txt 2>/dev/null)" = "600" ]; then report ok "new output file is -rw-------"; else report fail "new output file is -rw-------" "600, got $(stat -c '%a' out1.txt)"; fi
printf 'old old old\n' > old.txt
chmod 644 old.txt
run_shell 'echo new > old.txt\n'
if [ "$(cat old.txt)" = "new" ]; then report ok "cmd > file overwrites (no append)"; else report fail "cmd > file overwrites (no append)" "old.txt = 'new'"; fi
if [ "$(stat -c '%a' old.txt)" = "600" ]; then report ok "overwritten file becomes -rw-------"; else report fail "overwritten file becomes -rw-------" "600, got $(stat -c '%a' old.txt)"; fi
run_shell 'sort < abc.txt > sorted.txt\n'
if [ "$(head -1 sorted.txt)" = "a" ]; then report ok "cmd < in > out"; else report fail "cmd < in > out" "sorted.txt starts with a"; fi
run_shell 'sort > sorted2.txt < abc.txt\n'
if [ "$(head -1 sorted2.txt)" = "a" ]; then report ok "cmd > out < in"; else report fail "cmd > out < in" "sorted2.txt starts with a"; fi
expect "input file does not exist" 'cat < nofile.txt\n' "No such file"
expect "input is a directory" 'cat < adir\n' "Not a regular file"
cmp -s in.txt <(printf 'hello world\n')
if [ $? -eq 0 ]; then report ok "input file not modified"; else report fail "input file not modified" "in.txt unchanged"; fi
expect "missing file after >" 'ls >\n' "Incomplete"

echo "== Part 7: piping =="
expect "cmd1 | cmd2" 'cat abc.txt | sort\n' "a"
expect "cmd1 | cmd2 | cmd3" 'cat abc.txt | sort | head -1\n' "a"
expect "pipe output correct (wc)" 'cat abc.txt | wc -l\n' "3"
expect "pipe with command not found" 'ls | nosuchcmd\n' "command not found"
expect "pipe at start" '| ls\n' "Unexpected"
expect "pipe at end" 'ls |\n' "Incomplete"
expect "double pipe" 'ls | | wc\n' "Unexpected"

echo "== Part 8: background processing =="
expect "start prints [1] PID" 'sleep 1 &\n' "[1] "
expect "done message after finish" 'sleep 1 &\nsleep 2\necho x\n' "[1]+ done"
run_shell 'sleep 1 &\nsleep 2\necho x\n'
# the done message should come before the next prompt, not after it
if grep -A1 '+ done' <<< "$OUT" | tail -1 | grep -q 'testmachine:'; then report ok "done message printed before prompt"; else report fail "done message printed before prompt" "prompt on the line after done"; fi
expect "job numbers go up" 'sleep 1 &\nsleep 1 &\n' "[2] "
expect "background with pipe" 'cat abc.txt | sort &\nsleep 1\necho x\n' "[1]+ done"
run_shell 'echo bgfile > bg.txt &\nsleep 1\n'
if [ "$(cat bg.txt 2>/dev/null)" = "bgfile" ]; then report ok "background with > file"; else report fail "background with > file" "bg.txt = bgfile"; fi
expect "background with < file" 'cat < in.txt &\nsleep 1\n' "hello world"
expect "& alone is an error" '&\n' "Unexpected"
expect "& in the middle is an error" 'sleep 1 & ls\n' "Unexpected"

echo "== Part 9: internal commands =="
expect "cd dir then pwd" 'cd adir\npwd\n' "$T/adir"
expect "prompt changes after cd" 'cd adir\n' "tester@testmachine:$T/adir> "
expect "cd with no args goes HOME" 'cd\npwd\n' "$T/home"
expect "cd ~" 'cd ~\npwd\n' "$T/home"
expect "cd .." 'cd adir\ncd ..\npwd\n' "$T"
expect "cd too many args" 'cd a b\n' "too many arguments"
expect "cd target does not exist" 'cd nowhere\n' "No such file or directory"
expect "cd target not a directory" 'cd in.txt\n' "Not a directory"
expect "jobs with no jobs" 'jobs\n' "No active background processes"
expect "jobs shows [n]+ PID cmd" 'sleep 2 &\njobs\n' "[1]+ "
expect "jobs does not show finished job" 'sleep 1 &\nsleep 2\njobs\n' "No active background processes"
expect "exit with no valid commands" 'exit\n' "No valid commands"
expect "exit with 1 valid command" 'echo one\nexit\n' "echo one"
expect "exit shows last three" 'echo 1\necho 2\necho 3\necho 4\nexit\n' "echo 2"
expect_not "exit does not show 4th from last" 'echo 1\necho 2\necho 3\necho 4\nexit\n' "echo 1"
run_shell 'echo ok\nnosuchcmd\nexit\n'
if sed -n '/Last/,$p' <<< "$OUT" | grep -qx "nosuchcmd"; then report fail "invalid command not in history" "no nosuchcmd after exit"; else report ok "invalid command not in history"; fi
start=$(date +%s)
run_shell 'sleep 2 &\nexit\n'
end=$(date +%s)
if [ $((end - start)) -ge 2 ]; then report ok "exit waits for background jobs"; else report fail "exit waits for background jobs" ">= 2 seconds"; fi


echo "== Part 9 (more): cd =="
expect "cd absolute path /" 'cd /\npwd\n' "tester@testmachine:/> "
expect "cd \$HOME" 'cd $HOME\npwd\n' "$T/home"
expect "cd ~/dir1" 'cd ~/dir1\npwd\n' "$T/home/dir1"
expect "commands run in new dir after cd" 'cd adir\nls ..\n' "abc.txt"
expect "failed cd keeps old dir" 'cd adir\ncd nowhere\npwd\n' "$T/adir"
run_shell 'echo ok\ncd nowhere\nexit\n'
if sed -n '/Last/,$p' <<< "$OUT" | grep -q "cd nowhere"; then report fail "failed cd not in history" "no 'cd nowhere' after exit"; else report ok "failed cd not in history"; fi
expect "cd then relative path again" 'cd adir\ncd ..\ncd adir\npwd\n' "$T/adir"
expect_not "cd in a pipe does not crash" 'cd adir | cat\necho alive\n' "panicked"

echo "== Part 9 (more): jobs =="
expect "jobs lists 2 jobs (first)" 'sleep 2 &\nsleep 2 &\njobs\n' "[1]+ "
expect "jobs lists 2 jobs (second)" 'sleep 2 &\nsleep 2 &\njobs\n' "[2]+ "
expect "jobs shows command line" 'sleep 2 &\njobs\n' "sleep 2"
expect "job numbers not reused" 'sleep 1 &\nsleep 2\nsleep 1 &\n' "[2] "
run_shell 'sleep 1 &\nsleep 3 &\nsleep 2\njobs\n'
if grep -q '\[2\]+ [0-9]' <<< "$OUT" && ! grep -q '^\[1\]+ [0-9]' <<< "$OUT"; then report ok "jobs keeps running job, drops finished one"; else report fail "jobs keeps running job, drops finished one" "only [2] listed"; fi
run_shell 'jobs > jobsout.txt\n'
if grep -q "No active" jobsout.txt 2>/dev/null; then report ok "jobs > file"; else report fail "jobs > file" "jobsout.txt has jobs output"; fi
expect "jobs | cat" 'jobs | cat\n' "No active background processes"
expect "jobs with extra args does not crash" 'jobs extra\necho alive\n' "alive"

echo "== Part 9 (more): exit =="
run_shell 'echo a\necho b\nexit\n'
if sed -n '/Last/,$p' <<< "$OUT" | grep -qx "echo b" && ! sed -n '/Last/,$p' <<< "$OUT" | grep -qx "echo a"; then report ok "exit with 2 valid shows only the last one"; else report fail "exit with 2 valid shows only the last one" "echo b only"; fi
expect "exit with only invalid commands" 'nosuchcmd\ncd nowhere\nexit\n' "No valid commands"
expect_not "exit with args still exits" 'exit now\necho notrun\n' "notrun"
expect_not "commands after exit do not run" 'exit\necho notrun\n' "notrun"
time_shell 'sleep 1 &\nsleep 2 &\nexit\n'
if [ "$ELAPSED" -ge 2 ]; then report ok "exit waits for ALL background jobs"; else report fail "exit waits for ALL background jobs" ">= 2 seconds, got $ELAPSED"; fi
time_shell 'sleep 2 &\n'
if [ "$ELAPSED" -ge 2 ]; then report ok "Ctrl+D also waits for background jobs"; else report fail "Ctrl+D also waits for background jobs" ">= 2 seconds, got $ELAPSED"; fi
expect "exit in a pipe does not kill the shell" 'exit | cat\necho alive\n' "alive"
expect "background command counts as valid" 'sleep 1 &\nexit\n' "sleep 1 &"

echo "== Stress / edge cases =="
expect "extra spaces and tabs" 'echo    a  \t  b   \n' "a b"
expect "no spaces around |" 'cat abc.txt|sort|head -1\n' "a"
expect "no spaces around <" 'cat<in.txt\n' "hello world"
run_shell 'echo nospace>ns.txt\n'
if [ "$(cat ns.txt 2>/dev/null)" = "nospace" ]; then report ok "no spaces around >"; else report fail "no spaces around >" "ns.txt = nospace"; fi
expect "no space before &" 'sleep 1&\n' "[1] "
expect "long command (20 args)" 'echo 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20\n' "1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20"
LONGARG=$(printf 'x%.0s' $(seq 1 180))
expect "190 char command" "echo $LONGARG\n" "$LONGARG"
expect "big output through pipe (no deadlock)" 'seq 100000 | wc -l\n' "100000"
expect "reader quits early (head)" 'seq 1000000 | head -1\necho alive\n' "alive"
expect "5 pipes" 'seq 20 | sort -n | tail -5 | head -3 | tail -1 | cat\n' "18"
expect "directory as a command" '/tmp\n' "not found"
expect "output to missing folder" 'echo x > nodir/f.txt\necho alive\n' "alive"
expect "output to a directory" 'echo x > adir\necho alive\n' "alive"
if [ "$(id -u)" != "0" ]; then
    expect "unreadable input file" 'cat < noread.txt\necho alive\n' "alive"
fi
expect "10 background jobs at once" "$(for i in $(seq 1 10); do printf 'sleep 1 &\\n'; done)sleep 2\necho x\n" "[10]+ done"
INPUT=""
for i in $(seq 1 300); do INPUT="${INPUT}echo loop$i | cat\n"; done
OUT=$(printf '%b' "${INPUT}echo finished | cat\n" | env USER=tester MACHINE=testmachine \
    HOME="$T/home" PWD="$T" bash -c "ulimit -n 64; timeout 60 $SHELL_BIN" 2>&1)
if grep -q "finished" <<< "$OUT"; then report ok "300 pipes with only 64 fds (no fd leak)"; else report fail "300 pipes with only 64 fds (no fd leak)" "finished"; fi
INPUT=""
for i in $(seq 1 300); do INPUT="${INPUT}nosuch$i | cat\n"; done
OUT=$(printf '%b' "${INPUT}echo finished | cat\n" | env USER=tester MACHINE=testmachine \
    HOME="$T/home" PWD="$T" bash -c "ulimit -n 64; timeout 60 $SHELL_BIN" 2>&1)
if grep -q "finished" <<< "$OUT"; then report ok "300 failed pipes (no fd leak)"; else report fail "300 failed pipes (no fd leak)" "finished"; fi

echo "== Extra credit =="
expect "more than two pipes" 'cat abc.txt | sort | head -2 | tail -1\n' "b"
run_shell 'cat < abc.txt | sort > pr.txt\n'
if [ "$(head -1 pr.txt 2>/dev/null)" = "a" ]; then report ok "pipe + redirection together"; else report fail "pipe + redirection together" "pr.txt starts with a"; fi
expect "shell-ception: SHLVL goes up" "$SHELL_BIN < inner.txt\n" "inner 3"
expect "shell-ception: outer shell continues" "$SHELL_BIN < inner.txt\necho back\n" "back"

echo "== General =="
expect_not "no debug prints" 'echo hi\n' "token 0"
expect_not "no crash on empty line" '\n   \nexit\n' "panicked"
expect "Ctrl+D / end of input exits" 'echo hi\n' "Last valid command"


echo "== Code checks (run from the project folder) =="
cd "$OLDPWD_SAVE" || exit 1
if [ -d src ]; then
    LONG=$(awk 'length > 99 {print FILENAME":"FNR}' src/*.rs)
    if [ -z "$LONG" ]; then report ok "all lines under 100 chars"; else OUT="$LONG"; report fail "all lines under 100 chars" "no long lines"; fi
    BAD=$(grep -n "std::process::Command\|system(\|execvp\|execl" src/*.rs)
    if [ -z "$BAD" ]; then report ok "only fork/execv used"; else OUT="$BAD"; report fail "only fork/execv used" "no Command/system/execvp"; fi
fi

echo
echo "Passed: $PASS   Failed: $FAIL"
chmod 644 "$T/noread.txt"
rm -rf "$T"
[ "$FAIL" -eq 0 ]
