#!/bin/bash
# Automated tests for Project 1 shell
# usage: bash tests/run_tests.sh [path/to/shell]   (default: ./bin/shell)

SHELL_BIN=$(realpath "${1:-./bin/shell}")
if [ ! -x "$SHELL_BIN" ]; then
    echo "shell not found at $SHELL_BIN, build it first"
    exit 1
fi

T=$(mktemp -d)
mkdir -p "$T/home/dir1" "$T/adir"
printf 'hello world\n' > "$T/in.txt"
printf 'c\nb\na\n' > "$T/abc.txt"
printf 'not a program\n' > "$T/fake"
chmod +x "$T/fake"
printf 'echo inner $SHLVL\n' > "$T/inner.txt"
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

echo
echo "Passed: $PASS   Failed: $FAIL"
rm -rf "$T"
[ "$FAIL" -eq 0 ]
