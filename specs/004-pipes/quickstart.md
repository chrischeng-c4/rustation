# Quickstart Guide: Using Pipes in Rush

**Feature**: Pipe Operator (`|`)
**Version**: 0.2.0+
**Status**: Specification Complete

## What Are Pipes?

Pipes let you connect commands so the output of one command becomes the input of the next. This is one of the most powerful features in Unix shells.

**Basic Pattern**:
```
command1 | command2
```

The `|` operator takes everything command1 prints (stdout) and sends it as input (stdin) to command2.

---

## Quick Examples

### Example 1: Filter Files

**Task**: List only text files in a directory

```bash
$ ls | grep txt
file1.txt
file2.txt
notes.txt
```

**What happens**:
1. `ls` lists all files
2. `|` sends that list to grep
3. `grep txt` filters lines containing "txt"

---

### Example 2: Count Lines

**Task**: Count how many error messages are in a log file

```bash
$ cat app.log | grep ERROR | wc -l
      42
```

**What happens**:
1. `cat app.log` outputs the file contents
2. First `|` sends output to grep
3. `grep ERROR` filters lines containing "ERROR"
4. Second `|` sends filtered lines to wc
5. `wc -l` counts the lines

---

### Example 3: See First Few Results

**Task**: Show first 5 markdown files

```bash
$ ls -la | grep "\.md" | head -5
-rw-r--r--  1 user  staff  1234 Nov 19 README.md
-rw-r--r--  1 user  staff   567 Nov 19 NOTES.md
-rw-r--r--  1 user  staff   890 Nov 19 TODO.md
```

**What happens**:
1. `ls -la` lists files with details
2. First `|` sends to grep
3. `grep "\.md"` filters markdown files
4. Second `|` sends to head
5. `head -5` shows first 5 lines

---

## Common Patterns

### Pattern 1: Search and Filter

```bash
# Find processes by name
$ ps aux | grep rust

# Find functions in code
$ cat main.rs | grep "fn "

# Search history
$ history | grep git
```

### Pattern 2: Count and Summarize

```bash
# Count lines in a file
$ cat file.txt | wc -l

# Count words
$ cat file.txt | wc -w

# Count unique lines
$ cat file.txt | sort | uniq | wc -l
```

### Pattern 3: Transform Data

```bash
# Sort lines alphabetically
$ cat names.txt | sort

# Remove duplicates
$ cat data.txt | sort | uniq

# Reverse order
$ cat numbers.txt | sort -r
```

### Pattern 4: Extract Specific Data

```bash
# Get just filenames (column 9)
$ ls -la | awk '{print $9}'

# Get first word of each line
$ cat file.txt | cut -d' ' -f1

# Extract emails
$ cat contacts.txt | grep -E '\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b'
```

---

## How Pipes Work

### Data Flow

```
┌─────────┐      ┌─────────┐      ┌─────────┐
│ command1│─────▶│ command2│─────▶│ command3│
└─────────┘      └─────────┘      └─────────┘
   stdout          stdin              stdin
                   stdout             stdout
```

- Each command's **stdout** (standard output) connects to the next command's **stdin** (standard input)
- Error messages (stderr) still go to your terminal
- All commands run **at the same time** (concurrently)

### Exit Codes

The pipeline's exit code is the **last command's** exit code:

```bash
$ true | false
$ echo $?
1           # false's exit code

$ false | true
$ echo $?
0           # true's exit code
```

This matches standard Unix shell behavior (bash, zsh, fish).

---

## Special Cases

### Pipes in Quotes

Pipes inside quotes are treated as literal text, not operators:

```bash
$ echo "hello | world"
hello | world
```

No pipeline created - the `|` is just part of the string.

### Empty Output

If a command produces no output, the next command receives empty input:

```bash
$ true | cat
# (no output - true produces nothing)
```

This is normal and not an error.

### Commands That Fail

If a command fails, error messages go to stderr (your terminal):

```bash
$ ls /nonexistent | grep foo
ls: /nonexistent: No such file or directory
```

The pipeline continues - grep receives empty stdin and returns exit code 1 (no matches).

---

## Practical Use Cases

### Use Case 1: Log Analysis

**Goal**: Find and count warnings in application logs

```bash
# Count warnings
$ cat app.log | grep WARN | wc -l

# Show first 10 warnings
$ cat app.log | grep WARN | head -10

# Find warnings about database
$ cat app.log | grep WARN | grep database
```

### Use Case 2: File Management

**Goal**: Find large files

```bash
# List files by size
$ ls -lS | head -10

# Find files modified today
$ ls -lt | head -5

# Count files in directory
$ ls -1 | wc -l
```

### Use Case 3: Text Processing

**Goal**: Clean up and analyze text file

```bash
# Remove blank lines and count
$ cat file.txt | grep -v '^$' | wc -l

# Sort and remove duplicates
$ cat file.txt | sort | uniq

# Find most common words
$ cat file.txt | tr ' ' '\n' | sort | uniq -c | sort -rn | head -10
```

### Use Case 4: Code Search

**Goal**: Find specific code patterns

```bash
# Find all TODO comments
$ cat *.rs | grep TODO

# Count functions
$ cat *.rs | grep "fn " | wc -l

# Find error handling
$ cat *.rs | grep "Error" | head -20
```

---

## Performance Notes

Rush pipes are **fast**:

- Commands run **concurrently** (not waiting for each to finish)
- Uses OS-level pipes (kernel handles buffering)
- Overhead is **<5ms** compared to running commands separately

**Example**: This pipeline completes in ~1 second (not 3):
```bash
$ sleep 1 | sleep 1 | sleep 1
# All three sleep commands run at the same time
```

---

## Common Mistakes

### Mistake 1: Pipe Before Command

**Wrong**:
```bash
$ | grep foo
rush: parse error: Empty command before pipe
```

**Right**:
```bash
$ ls | grep foo
```

### Mistake 2: Pipe After Command (Nothing Following)

**Wrong**:
```bash
$ ls |
rush: parse error: Empty command after pipe
```

**Right**:
```bash
$ ls | grep txt
```

### Mistake 3: Double Pipe

**Wrong**:
```bash
$ ls | | grep
rush: parse error: Empty command before pipe
```

**Right**:
```bash
$ ls | grep txt
```

### Mistake 4: Confusing Pipe with Boolean OR

Pipes (`|`) are different from logical OR (`||`):

```bash
# Pipe: stdout → stdin
$ ls | grep foo

# Logical OR: run second if first fails
$ command1 || command2
```

---

## Tips and Tricks

### Tip 1: Build Pipelines Incrementally

Start simple, add stages as needed:

```bash
# Start with basic command
$ cat app.log

# Add filtering
$ cat app.log | grep ERROR

# Add counting
$ cat app.log | grep ERROR | wc -l
```

### Tip 2: Use `head` to Preview Large Output

```bash
# Instead of overwhelming your terminal
$ cat huge_file.txt | head -20
```

### Tip 3: Combine with Other Features

Pipes work with all rush features:

```bash
# Tab completion works in pipelines
$ ls | grep txt<TAB>

# Syntax highlighting shows pipes
$ cat file.txt | grep pattern
#            ↑ highlighted as operator

# History suggests full pipelines
$ ls | grep<→>     # Suggests "ls | grep txt"
```

---

## Next Steps

- **Learn more**: See [CLI.md](../../../crates/rush/CLI.md) for all rush features
- **Advanced features**: Check roadmap for redirections (`>`, `<`) in v0.3.0
- **Report issues**: File bugs at GitHub if pipes don't work as expected

---

## Summary

**Remember**:
- `command1 | command2` sends command1's output to command2's input
- Commands run concurrently (fast!)
- Exit code is last command's exit code
- Pipes in quotes (`"a | b"`) are treated as text

**Most Common Uses**:
- `ls | grep pattern` - filter files
- `cat file | wc -l` - count lines
- `cat file | grep X | head -5` - search and preview

Happy piping! 🚀
