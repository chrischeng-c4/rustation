# Quickstart: History-Based Autosuggestions Testing

**Feature**: 003-autosuggestions
**Purpose**: Manual testing guide for validating autosuggestions functionality
**Date**: 2025-11-17

## Prerequisites

### Build and Install

```bash
# From repository root
cd /Users/chrischeng/projects/rust-station

# Build rush in release mode
cargo build --release -p rush

# Install to ~/.local/bin
mkdir -p ~/.local/bin
cp target/release/rush ~/.local/bin/

# Verify installation
rush --version
```

### Prepare Test History

```bash
# Start rush to create history file
rush

# Run these commands to populate history:
$ git status
$ git commit -m "test"
$ git stash
$ cargo build
$ cargo build --release
$ cargo test
$ ls -la
$ ls -lh
$ pwd
$ echo "hello world"
$ echo "test"

# Exit rush
$ exit
```

**Result**: History file now contains 11 commands at:
- macOS: `~/Library/Application Support/rush/history.txt`
- Linux: `~/.local/share/rush/history.txt`

---

## Test Scenarios

### US1: Basic Inline Suggestion Display (P1)

#### Test 1.1: Simple Prefix Match

**Steps**:
1. Start rush: `rush`
2. Type: `git s` (do not press Enter)
3. Observe the display

**Expected**:
- Display shows: `git s` followed by grayed-out `tash` (from most recent "git stash")
- Grayed text appears immediately after typing 's'
- Suggestion updates in real-time

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

#### Test 1.2: Multiple Matches (Most Recent Wins)

**Steps**:
1. In rush, type: `cargo b` (do not press Enter)
2. Observe which suggestion appears

**Expected**:
- Display shows: `cargo b` + grayed `uild --release` (most recent match)
- NOT "uild" from older "cargo build"

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

#### Test 1.3: No Match (No Suggestion)

**Steps**:
1. In rush, type: `xyz` (not in history)
2. Observe the display

**Expected**:
- Display shows: `xyz` with NO grayed text
- No suggestion displayed
- No errors or crashes

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

#### Test 1.4: Real-Time Updates

**Steps**:
1. In rush, type: `e`
2. Observe suggestion
3. Type: `c`
4. Observe suggestion change
5. Type: `h`
6. Observe suggestion change

**Expected**:
- After `e`: Suggests `cho "test"` or `cho "hello world"` (most recent echo)
- After `ec`: Suggests `ho "test"` or `ho "hello world"`
- After `ech`: Suggests `o "test"` or `o "hello world"`
- Suggestions update instantly (<50ms perceived lag)

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

### US2: Accept Suggestion with Arrow Key (P2)

#### Test 2.1: Accept Full Suggestion

**Steps**:
1. In rush, type: `git s`
2. Observe grayed suggestion
3. Press: `Right Arrow` key
4. Observe result

**Expected**:
- After Right Arrow: Input buffer becomes `git stash` (full text)
- Cursor moves to end of line
- Grayed suggestion disappears (now actual input)
- Can press Enter to execute "git stash"

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

#### Test 2.2: Accept Then Continue Typing

**Steps**:
1. In rush, type: `cargo b`
2. Press: `Right Arrow` (accept suggestion)
3. Type: ` -vv` (add flag)
4. Press: `Enter`

**Expected**:
- After Right Arrow: Buffer becomes `cargo build --release`
- After typing ` -vv`: Buffer becomes `cargo build --release -vv`
- Command executes: `cargo build --release -vv`

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

#### Test 2.3: Right Arrow When No Suggestion

**Steps**:
1. In rush, type: `xyz` (no match)
2. Press: `Right Arrow`
3. Observe behavior

**Expected**:
- Right Arrow acts as normal cursor movement (no-op at end of line)
- No error or crash
- Input remains `xyz`

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

### US3: Accept Partial Suggestion (P3)

#### Test 3.1: Accept Next Word

**Steps**:
1. In rush, type: `echo`
2. Observe suggestion (e.g., ` "hello world"`)
3. Press: `Alt+Right Arrow` (or `Ctrl+Right Arrow` on some systems)
4. Observe result

**Expected**:
- After Alt+Right: Buffer becomes `echo "hello` (one word accepted)
- Remaining suggestion: ` world"` shown in gray
- Cursor at end of `echo "hello`

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

#### Test 3.2: Accept Word-by-Word

**Steps**:
1. In rush, type: `cargo`
2. Press: `Alt+Right Arrow` (accept first word)
3. Observe result
4. Press: `Alt+Right Arrow` again (accept second word)
5. Observe result

**Expected**:
- After first Alt+Right: `cargo build`
- After second Alt+Right: `cargo build --release`
- Each keypress accepts one word at a time

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

#### Test 3.3: Single-Word Suggestion

**Steps**:
1. In rush, type: `pw`
2. Observe suggestion (` d` from "pwd")
3. Press: `Alt+Right Arrow`
4. Observe result

**Expected**:
- After Alt+Right: Buffer becomes `pwd` (entire suggestion accepted)
- Single-word suggestions accepted completely

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

### Edge Cases

#### Edge 1: Empty History

**Steps**:
1. Delete history file:
   - macOS: `rm ~/Library/Application\ Support/rush/history.txt`
   - Linux: `rm ~/.local/share/rush/history.txt`
2. Start rush: `rush`
3. Type: `git status`
4. Observe behavior

**Expected**:
- No suggestions appear (history empty)
- Shell functions normally
- No errors or crashes

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

#### Edge 2: Cursor in Middle of Line

**Steps**:
1. In rush, type: `git status`
2. Press: `Home` key (move cursor to start)
3. Press: `Right Arrow` 3 times (cursor at position 3, after "git")
4. Observe suggestions

**Expected**:
- No suggestions appear (cursor not at end)
- Shell behaves normally
- Can edit text normally

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

#### Edge 3: Very Long Suggestion

**Steps**:
1. Add long command to history:
   ```bash
   $ cargo build --release --features "feature1 feature2 feature3" --target x86_64-apple-darwin --verbose
   $ exit
   ```
2. Restart rush: `rush`
3. Type: `cargo b`
4. Observe suggestion display

**Expected**:
- Suggestion displays up to terminal width
- If longer than terminal, truncated with no visual artifacts
- Full text still available for acceptance (not truncated internally)

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

#### Edge 4: Special Characters in History

**Steps**:
1. Add commands with special characters:
   ```bash
   $ echo "hello 'world'"
   $ git commit -m "fix: improve \"parser\""
   $ echo hello\ world
   $ exit
   ```
2. Restart rush: `rush`
3. Type: `echo "h` and observe
4. Type: `git c` and observe
5. Type: `echo h` and observe

**Expected**:
- Suggestions display special characters correctly
- Quotes, escapes render without artifacts
- Accepting suggestion preserves exact text

**Actual**: _[Fill in during testing]_

**Status**: ⬜ PASS / ⬜ FAIL

---

### Performance Testing

#### Perf 1: Large History (10k Entries)

**Steps**:
1. Generate large history:
   ```bash
   # Run this in rush
   $ for i in {1..10000}; do echo "command_$i"; done >> ~/Library/Application\ Support/rush/history.txt
   $ exit
   ```
2. Restart rush: `rush`
3. Type: `command_9` and start timer
4. Observe when suggestion appears

**Expected**:
- Suggestion appears within 50ms (perceived as instant)
- No noticeable lag
- Shell remains responsive

**Actual**: _[Fill in during testing]_

**Measured Latency**: _[Fill in]_ ms

**Status**: ⬜ PASS / ⬜ FAIL

---

## Success Criteria Validation

### SC-001: Typing Time Reduction

**Test Method**:
1. Without autosuggestions: Type "git status" manually (time it)
2. With autosuggestions: Type "git s" + Right Arrow (time it)
3. Calculate reduction

**Expected**: >50% typing time reduction

**Actual**:
- Without: _[Fill in]_ seconds
- With: _[Fill in]_ seconds
- Reduction: _[Fill in]_ %

**Status**: ⬜ PASS / ⬜ FAIL

---

### SC-002: Suggestion Latency

**Test Method**:
1. Type single character
2. Measure time until suggestion visible

**Expected**: <50ms (perceived as instant)

**Actual**: _[Fill in]_ ms

**Status**: ⬜ PASS / ⬜ FAIL

---

### SC-003: User Adoption

**Test Method**:
1. Run 10 commands
2. Count how many times you used suggestion acceptance

**Expected**: At least 1 suggestion accepted

**Actual**: _[Fill in]_ / 10 commands used suggestions

**Status**: ⬜ PASS / ⬜ FAIL

---

## Debugging

### Enable Verbose Logging

```bash
# Start rush with debug logging
rush -vv

# Check logs
# macOS:
tail -f ~/Library/Application\ Support/rush/rush-v0.1.0.log

# Linux:
tail -f ~/.local/share/rush/rush-v0.1.0.log
```

### Common Issues

| Issue | Possible Cause | Solution |
|-------|---------------|----------|
| No suggestions appear | History empty | Run commands to populate history |
| Wrong suggestion shown | Expectation mismatch | Verify history order (most recent wins) |
| Lag when typing | Performance issue | Check history size, measure latency |
| Crashes | Implementation bug | Check logs, report with steps to reproduce |

---

## Reporting Results

### Test Summary Template

```markdown
## Autosuggestions Test Results

**Date**: [Date]
**Tester**: [Name]
**Rush Version**: [version from rush --version]
**Platform**: [macOS version / Linux distro]

### Test Results

- US1 Tests: ⬜ / 4 passed
- US2 Tests: ⬜ / 3 passed
- US3 Tests: ⬜ / 3 passed
- Edge Cases: ⬜ / 4 passed
- Performance: ⬜ / 1 passed

**Total**: ⬜ / 15 tests passed

### Issues Found

1. [Issue description]
   - Steps to reproduce
   - Expected behavior
   - Actual behavior

### Performance Metrics

- Suggestion latency: ⬜ ms
- Large history (10k) latency: ⬜ ms
- Typing time reduction: ⬜%

### Notes

[Any additional observations]
```

---

## Next Steps

After completing all tests:

1. **If all tests pass**:
   - Document results
   - Mark feature as ready for production
   - Update KNOWN_ISSUES.md

2. **If tests fail**:
   - Document failures in detail
   - Create issues for each failure
   - Prioritize fixes

3. **Performance issues**:
   - If latency >50ms: Implement optimizations (LRU cache, async search)
   - If memory >1MB: Review caching strategy

---

**Testing Complete**: ⬜ YES / ⬜ NO (in progress)
