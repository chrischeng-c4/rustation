# API Contract: Redirection Behavior

**Feature**: 005-output-redirection
**Date**: 2025-11-20
**Purpose**: Define behavior contracts and invariants for I/O redirection operations

## Overview

This document specifies the exact behavior of redirection operators (`>`, `>>`, `<`) as observable by users and tests. These contracts serve as the source of truth for implementation and validation.

---

## Output Redirection (`>`)

### Contract OR-001: File Creation

**Given**: File does not exist at target path
**When**: Command executes with `> path/to/file.txt`
**Then**:
- File is created at specified path
- File permissions are 0644 (rw-r--r--)
- File initially contains zero bytes
- Command stdout is redirected to file
- Command stderr still goes to terminal

**Test**: `echo "test" > /tmp/newfile.txt && cat /tmp/newfile.txt` → outputs "test"

---

### Contract OR-002: File Truncation

**Given**: File exists at target path with content "old content"
**When**: Command executes with `> path/to/file.txt`
**Then**:
- Existing file is truncated to zero bytes BEFORE command executes
- File permissions are unchanged
- Command stdout is redirected to file
- Previous content is lost (overwritten)

**Test**:
```bash
echo "old" > /tmp/file.txt
echo "new" > /tmp/file.txt
cat /tmp/file.txt  # outputs only "new"
```

---

### Contract OR-003: Empty Command Output

**Given**: Command produces no stdout output
**When**: Command executes with `> file.txt`
**Then**:
- File is created (if doesn't exist) or truncated (if exists)
- File contains zero bytes
- No error is reported (empty file is valid)

**Test**: `true > /tmp/empty.txt && wc -c /tmp/empty.txt` → outputs "0"

---

### Contract OR-004: Directory Error

**Given**: Target path is an existing directory
**When**: Command attempts `> /existing/directory`
**Then**:
- Error message: "`<path>: is a directory`"
- Command is not executed
- Exit code is non-zero (1)

**Test**: `echo "test" > /tmp` → error: "/tmp: is a directory"

---

### Contract OR-005: Permission Error

**Given**: Target path parent directory is not writable
**When**: Command attempts `> /readonly/dir/file.txt`
**Then**:
- Error message: "`<path>: permission denied`"
- Command is not executed
- Exit code is non-zero (1)

**Test**: `echo "test" > /root/file.txt` (as non-root) → error: "permission denied"

---

## Append Redirection (`>>`)

### Contract AR-001: File Creation

**Given**: File does not exist at target path
**When**: Command executes with `>> path/to/file.txt`
**Then**:
- File is created (same as `>`)
- File permissions are 0644
- Command stdout is redirected to file
- Behavior identical to `>` for non-existent files

**Test**: `echo "test" >> /tmp/newfile.txt && cat /tmp/newfile.txt` → outputs "test"

---

### Contract AR-002: Append to Existing File

**Given**: File exists at target path with content "line1\n"
**When**: Command executes with `>> path/to/file.txt`
**Then**:
- File is NOT truncated
- Command stdout is appended to end of file
- Existing content is preserved
- File pointer seeks to end before writing

**Test**:
```bash
echo "line1" > /tmp/file.txt
echo "line2" >> /tmp/file.txt
cat /tmp/file.txt  # outputs "line1\nline2\n"
```

---

### Contract AR-003: Multiple Appends

**Given**: File exists with content "a\n"
**When**: Multiple commands execute with `>>` in sequence
**Then**:
- Each append adds to end of file
- Order is preserved
- No data loss between appends

**Test**:
```bash
echo "a" > /tmp/file.txt
echo "b" >> /tmp/file.txt
echo "c" >> /tmp/file.txt
cat /tmp/file.txt  # outputs "a\nb\nc\n"
```

---

### Contract AR-004: Large File Append

**Given**: File exists with 1GB of content
**When**: Command executes with `>> file.txt`
**Then**:
- Only new data is written (no full file rewrite)
- Append operation is O(1) with respect to existing file size
- File system handles seek-to-end efficiently

**Test**: Benchmark append to large file (should be <10ms regardless of file size)

---

## Input Redirection (`<`)

### Contract IR-001: File Reading

**Given**: File exists at source path with content "test data\n"
**When**: Command executes with `< path/to/file.txt`
**Then**:
- File is opened for reading
- File content is connected to command stdin
- Command receives file content as if typed interactively
- Command stdout/stderr unaffected (still go to terminal)

**Test**: `cat < /tmp/file.txt` → outputs file content

---

### Contract IR-002: File Not Found

**Given**: File does not exist at source path
**When**: Command attempts `< /nonexistent/file.txt`
**Then**:
- Error message: "`<path>: file not found`"
- Command is not executed
- Exit code is non-zero (1)

**Test**: `cat < /tmp/nonexistent.txt` → error: "file not found"

---

### Contract IR-003: Empty File Input

**Given**: File exists but contains zero bytes
**When**: Command executes with `< empty.txt`
**Then**:
- Command receives EOF immediately on stdin
- Command executes normally (empty input is valid)
- Behavior same as if user pressed Ctrl+D immediately

**Test**: `cat < /tmp/empty.txt` → outputs nothing (but no error)

---

### Contract IR-004: Binary File Input

**Given**: File contains binary data (not text)
**When**: Command executes with `< binary.dat`
**Then**:
- Raw bytes are passed to command stdin
- No transformation or validation applied
- Command receives bytes as-is
- Text-processing commands may produce garbage output (but no error from shell)

**Test**: `cat < /bin/ls | head -1` → outputs binary garbage (but shell doesn't error)

---

## Combined Redirections

### Contract CR-001: Input and Output

**Given**: input.txt contains "data\n"
**When**: Command executes `< input.txt command > output.txt`
**Then**:
- input.txt connected to stdin
- output.txt receives stdout
- Command reads from input.txt and writes to output.txt
- Order of redirections doesn't matter (`> out < in` same as `< in > out`)

**Test**: `sort < unsorted.txt > sorted.txt && cat sorted.txt` → shows sorted content

---

### Contract CR-002: Multiple Output Redirections (Last Wins)

**Given**: Command line `echo test > a.txt > b.txt`
**When**: Command executes
**Then**:
- Both a.txt and b.txt are created/truncated
- Only b.txt receives output (last wins)
- a.txt ends up empty
- No error is reported (valid POSIX behavior)

**Test**:
```bash
echo "test" > /tmp/a.txt > /tmp/b.txt
cat /tmp/a.txt  # empty
cat /tmp/b.txt  # contains "test"
```

---

### Contract CR-003: Multiple Input Redirections (Last Wins)

**Given**: Command line `cat < a.txt < b.txt`
**When**: Command executes
**Then**:
- Only b.txt is used for stdin (last wins)
- a.txt is ignored (may still be validated to exist)
- Command reads from b.txt

**Test**:
```bash
echo "from a" > /tmp/a.txt
echo "from b" > /tmp/b.txt
cat < /tmp/a.txt < /tmp/b.txt  # outputs "from b" only
```

---

### Contract CR-004: Redirections with Pipes

**Given**: Command line `cat < input.txt | grep pattern > output.txt`
**When**: Pipeline executes
**Then**:
- First segment (cat): stdin from input.txt, stdout to pipe
- Second segment (grep): stdin from pipe, stdout to output.txt
- Redirections and pipes compose naturally
- Data flows: input.txt → cat → pipe → grep → output.txt

**Test**: `cat < data.txt | grep "x" > results.txt && cat results.txt` → shows filtered results

---

## Quote Handling

### Contract QH-001: Operators in Double Quotes

**Given**: Command line `echo "a > b"`
**When**: Command executes
**Then**:
- `>` is treated as literal text (not operator)
- Output is exactly: `a > b`
- No file redirection occurs
- No files are created

**Test**: `echo "a > b"` → outputs "a > b" (not redirected)

---

### Contract QH-002: Operators in Single Quotes

**Given**: Command line `echo 'a > b'`
**When**: Command executes
**Then**:
- `>` is treated as literal text (not operator)
- Output is exactly: `a > b`
- No file redirection occurs

**Test**: `echo 'a > b'` → outputs "a > b"

---

### Contract QH-003: Escaped Operators

**Given**: Command line `echo a \> b`
**When**: Command executes
**Then**:
- Backslash escapes the `>`
- Output is: `a > b` (literal)
- No file redirection occurs

**Test**: `echo a \> b` → outputs "a > b"

---

### Contract QH-004: Quoted File Paths

**Given**: Command line `echo test > "my file.txt"`
**When**: Command executes
**Then**:
- File named "my file.txt" (with space) is created
- Quotes allow spaces in file paths
- Redirection still occurs (operator outside quotes)

**Test**: `echo test > "my file.txt" && cat "my file.txt"` → outputs "test"

---

## Error Handling

### Contract EH-001: Syntax Error - Missing File Path

**Given**: Command line `echo test >`
**When**: Parser processes command
**Then**:
- Error message: "Expected file path after > operator"
- Command is not executed
- Exit code is non-zero

**Test**: `echo test >` → parse error before execution

---

### Contract EH-002: Syntax Error - Empty File Path

**Given**: Command line `echo test > ""`
**When**: Parser processes command
**Then**:
- Error message: "Empty file path for redirection"
- Command is not executed
- Exit code is non-zero

**Test**: `echo test > ""` → parse error

---

### Contract EH-003: File System Full

**Given**: File system has no space remaining
**When**: Command executes with `> file.txt`
**Then**:
- Error message includes "No space left on device" or similar
- Command may have executed but output lost
- Exit code reflects file system error

**Test**: (Requires simulated full disk - integration test)

---

### Contract EH-004: Interrupted Command

**Given**: Command is running with output redirection
**When**: User presses Ctrl+C
**Then**:
- Command is interrupted
- File handles are closed properly (RAII)
- Partial output may be in file
- No file descriptors leaked

**Test**: `sleep 10 > /tmp/test.txt` → Ctrl+C → file closed properly

---

## Performance Contracts

### Contract PC-001: Redirection Setup Overhead

**Given**: Command executes without redirections (baseline)
**When**: Same command executes with `> output.txt`
**Then**:
- Overhead is less than 1ms (constitution requirement)
- File open, truncate, and Stdio setup complete in <1ms
- Measured on modern hardware (2020+ Mac)

**Test**: Benchmark comparison (feature requirement SC-004)

---

### Contract PC-002: Large Output Performance

**Given**: Command produces 100MB of output
**When**: Command executes with `> output.txt`
**Then**:
- Output writing is streamed (not buffered in memory)
- Memory usage remains constant (no accumulation)
- Performance similar to direct stdout (kernel handles buffering)

**Test**: `yes "test" | head -n 10000000 > /tmp/large.txt` → constant memory

---

### Contract PC-003: Append Performance Scaling

**Given**: Files of varying sizes (1KB, 1MB, 1GB)
**When**: Command executes with `>> file.txt`
**Then**:
- Append time is constant regardless of existing file size
- O(1) performance (seek-to-end is OS operation)
- No correlation between file size and append duration

**Test**: Benchmark append to files of different sizes

---

## Backward Compatibility Contracts

### Contract BC-001: No Redirections

**Given**: Command has no redirection operators
**When**: Command executes normally
**Then**:
- Behavior identical to pre-redirection implementation
- All existing 286 tests continue to pass
- Zero performance impact (no overhead for unused feature)

**Test**: Run full existing test suite → 100% pass rate

---

### Contract BC-002: Existing Pipe Behavior

**Given**: Command uses pipe operator `|`
**When**: Pipeline executes
**Then**:
- Pipe behavior unchanged from feature 004
- Pipes can be combined with redirections
- Existing pipe tests continue to pass

**Test**: Run feature 004 pipe tests → 100% pass rate

---

## Contract Validation

Each contract maps to test cases in the test suite:

| Contract ID | Test Type | Test File |
|-------------|-----------|-----------|
| OR-001 to OR-005 | Integration | redirection_tests.rs (output) |
| AR-001 to AR-004 | Integration | redirection_tests.rs (append) |
| IR-001 to IR-004 | Integration | redirection_tests.rs (input) |
| CR-001 to CR-004 | Integration | combined_tests.rs |
| QH-001 to QH-004 | Unit | redirection_parser_tests.rs |
| EH-001 to EH-004 | Integration | redirection_tests.rs (errors) |
| PC-001 to PC-003 | Benchmark | redirection_bench.rs |
| BC-001 to BC-002 | Integration | Existing test suites |

---

**Contract Status**: Defined and ready for implementation validation.
