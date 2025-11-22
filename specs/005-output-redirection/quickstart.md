# Quickstart: Output Redirection

**Feature**: 005-output-redirection
**Date**: 2025-11-20
**Purpose**: Quick reference and usage examples for I/O redirection operators

## Overview

rush shell supports three I/O redirection operators for working with files:

| Operator | Name | Purpose | Example |
|----------|------|---------|---------|
| `>` | Output | Write to file (overwrite) | `ls > files.txt` |
| `>>` | Append | Add to file (append) | `echo "log" >> log.txt` |
| `<` | Input | Read from file | `cat < data.txt` |

---

## Basic Examples

### Output Redirection (`>`)

**Save command output to file**:
```bash
# Create/overwrite file with output
ls -la > directory_listing.txt

# Run multiple times - file is overwritten each time
echo "first" > file.txt
echo "second" > file.txt  # file now contains only "second"
```

**Capture errors** (requires stderr redirection - future feature):
```bash
# Currently only stdout is redirected
# Errors still appear on terminal
nonexistent_command > output.txt  # Error message on screen, output.txt empty
```

### Append Redirection (`>>`)

**Build up log files**:
```bash
# Create initial log entry
echo "Started at $(date)" > build.log

# Append additional entries
echo "Building..." >> build.log
echo "Testing..." >> build.log
echo "Done!" >> build.log
```

**Accumulate data**:
```bash
# Collect outputs from multiple commands
ls /tmp >> all_files.txt
ls /var >> all_files.txt
ls /etc >> all_files.txt
```

### Input Redirection (`<`)

**Feed file to command**:
```bash
# Count lines in file
wc -l < data.txt

# Sort file contents
sort < unsorted.txt

# Search file contents
grep "pattern" < large_file.txt
```

---

## Combined Redirection

### Input + Output

**Transform files**:
```bash
# Sort file and save result
sort < unsorted.txt > sorted.txt

# Process data through filter
grep "error" < log.txt > errors.txt

# Multiple transformations
tr '[:lower:]' '[:upper:]' < input.txt > output.txt
```

### Redirections with Pipes

**Complex data pipelines**:
```bash
# Read from file, pipe through commands, save result
cat < data.txt | grep "pattern" | sort > results.txt

# Chain multiple filters
< input.txt grep "foo" | sed 's/old/new/' | wc -l > count.txt
```

---

## Common Patterns

### Create Empty File

```bash
# Create or truncate file to zero bytes
> empty.txt

# Alternative (more explicit)
true > empty.txt
```

### Backup Before Overwrite

```bash
# Copy first, then overwrite
cp important.txt important.txt.bak
process_data < important.txt > important.txt.new
mv important.txt.new important.txt
```

### Append with Timestamp

```bash
# Add timestamped log entries
echo "[$(date)] Application started" >> app.log
echo "[$(date)] Processing data..." >> app.log
echo "[$(date)] Complete" >> app.log
```

### Combine Multiple Files

```bash
# Concatenate files into one
cat file1.txt > combined.txt
cat file2.txt >> combined.txt
cat file3.txt >> combined.txt

# Or use pipes
cat file1.txt file2.txt file3.txt > combined.txt
```

### Process File In-Place (Safe)

```bash
# WRONG: Truncates file before reading (file becomes empty!)
sort < data.txt > data.txt  # ❌ DON'T DO THIS

# CORRECT: Use temporary file
sort < data.txt > data.txt.sorted
mv data.txt.sorted data.txt  # ✅ Safe
```

---

## File Path Handling

### Relative Paths

```bash
# Current directory
echo "test" > output.txt

# Subdirectory (must exist)
echo "test" > subdir/output.txt

# Parent directory
echo "test" > ../output.txt
```

### Absolute Paths

```bash
# Explicit full path
echo "test" > /tmp/output.txt

# Home directory expansion
echo "test" > ~/Documents/output.txt
```

### Paths with Spaces

```bash
# Use quotes for paths with spaces
echo "test" > "my file.txt"
echo "test" > "Documents/My Project/output.txt"

# Without quotes = error
echo "test" > my file.txt  # ❌ Error: tries to redirect to "my", extra arg "file.txt"
```

---

## Quote Handling

### Operators in Strings

```bash
# Operators inside quotes are literal text
echo "a > b" > file.txt  # file contains: a > b
echo "redirect: >" > file.txt  # file contains: redirect: >

# Escape outside quotes to treat as literal
echo a \> b  # outputs: a > b (no redirection)
```

### When Quotes Matter

```bash
# These behave differently:
echo test > output.txt  # Redirects to file
echo "test > output.txt"  # Prints string with > in it

# Redirection happens outside quotes:
echo "my data" > file.txt  # ✅ Redirects
echo "my data" ">" file.txt  # ✅ Redirects (quotes don't block operator)
echo "my data >" file.txt  # ❌ Error: operator inside string, file.txt treated as arg
```

---

## Error Scenarios

### File Not Found (Input)

```bash
# Input file doesn't exist
$ cat < nonexistent.txt
rush: nonexistent.txt: file not found
```

### Permission Denied (Output)

```bash
# Can't write to directory
$ echo "test" > /root/file.txt
rush: /root/file.txt: permission denied
```

### Is a Directory (Output)

```bash
# Can't redirect to directory
$ echo "test" > /tmp
rush: /tmp: is a directory
```

### Missing File Path

```bash
# Operator without file path
$ echo "test" >
rush: parse error: expected file path after > operator
```

---

## Performance Tips

### Stream Large Files

```bash
# ✅ Efficient: Streams data, constant memory
cat large_file.txt | grep "pattern" > results.txt

# ✅ Also efficient: Direct pipe to file
grep "pattern" large_file.txt > results.txt
```

### Avoid Unnecessary Pipes

```bash
# ❌ Inefficient: Extra process
cat file.txt | > output.txt  # Unnecessary cat

# ✅ Efficient: Direct redirection
< file.txt tee output.txt
```

### Append vs Overwrite

```bash
# ✅ Append is O(1) regardless of file size
echo "new line" >> huge_log_file.txt  # Fast even for GB files

# ⚠️ Overwrite truncates entire file first
echo "new content" > huge_file.txt  # Truncates then writes
```

---

## Integration with Other Features

### With Pipes (Feature 004)

```bash
# Redirections work naturally with pipes
ls | grep txt | wc -l > count.txt

# Multiple redirections and pipes
cat < input.txt | sort | uniq > output.txt
```

### With Background Jobs (Future Feature)

```bash
# Will work with job control
long_running_command > log.txt &

# Redirect both stdout and append
process_data >> results.txt &
```

---

## Limitations (MVP)

The following are **not yet supported** in this feature:

### Stderr Redirection
```bash
# ❌ Not yet implemented
command 2> errors.txt
command 2>> errors.log
```

### Combined Stdout/Stderr
```bash
# ❌ Not yet implemented
command &> output.txt
command &>> output.txt
```

### Here Documents
```bash
# ❌ Not yet implemented
cat << EOF
multi-line
text
EOF
```

### File Descriptor Manipulation
```bash
# ❌ Not yet implemented
command 3> file.txt
command >&3
```

### Process Substitution
```bash
# ❌ Not yet implemented
diff <(sort a.txt) <(sort b.txt)
command >(tee file.txt)
```

These features are deferred to future releases.

---

## Troubleshooting

### "File not found" but file exists

**Problem**: Shell can't find file
**Solution**: Check path is correct (relative/absolute), check permissions

```bash
# Check file exists
ls -l data.txt

# Try absolute path
cat < /full/path/to/data.txt
```

### Redirection creates empty file

**Problem**: Command failed but file was created empty
**Solution**: File is created/truncated before command runs (this is correct POSIX behavior)

```bash
# File created even if command fails
nonexistent_cmd > output.txt  # output.txt is empty

# To avoid: check command first
which my_command && my_command > output.txt
```

### Operators in strings not working

**Problem**: Quote handling confusion
**Solution**: Operators work outside quotes only

```bash
# ❌ Wrong: operator inside string
echo "data" > "file.txt > backup.txt"  # Tries to create file with > in name

# ✅ Correct: operator outside quotes
echo "data" > file.txt
```

### Same file input and output

**Problem**: `sort < file.txt > file.txt` empties file
**Solution**: File is truncated before reading (race condition)

```bash
# ❌ Wrong: file destroyed
sort < data.txt > data.txt  # data.txt becomes empty

# ✅ Correct: use temp file
sort < data.txt > data.txt.tmp && mv data.txt.tmp data.txt
```

---

## Quick Reference Card

```
┌─────────────────────────────────────────────────────────┐
│ rush Shell Redirection Quick Reference                 │
├─────────────────────────────────────────────────────────┤
│ OPERATOR │ PURPOSE              │ EXAMPLE                │
├──────────┼──────────────────────┼────────────────────────┤
│ >        │ Output (overwrite)   │ ls > files.txt         │
│ >>       │ Output (append)      │ echo x >> log.txt      │
│ <        │ Input                │ sort < data.txt        │
├──────────┼──────────────────────┼────────────────────────┤
│ COMBINED EXAMPLES                                        │
├──────────────────────────────────────────────────────────┤
│ < in.txt cmd > out.txt  │ Read from file, write to file  │
│ cat < x | grep a > y    │ With pipes                     │
│ cmd > a.txt > b.txt     │ Multiple (last wins)           │
├──────────────────────────────────────────────────────────┤
│ QUOTES                                                   │
├──────────────────────────────────────────────────────────┤
│ echo "a > b"            │ Literal (no redirection)       │
│ echo data > "my file"   │ Space in file name             │
├──────────────────────────────────────────────────────────┤
│ ERRORS                                                   │
├──────────────────────────────────────────────────────────┤
│ cat < missing.txt       │ file not found                 │
│ echo x > /root/file     │ permission denied              │
│ echo x > /tmp           │ is a directory                 │
└──────────────────────────────────────────────────────────┘
```

---

**Next Steps**:
- See [spec.md](./spec.md) for complete feature specification
- See [data-model.md](./data-model.md) for implementation details
- See [contracts/redirection-api.md](./contracts/redirection-api.md) for behavior contracts
