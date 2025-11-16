# Technical Research: Tab Completion

**Feature**: 002-tab-completion
**Date**: 2025-11-16
**Purpose**: Research technical unknowns and document implementation decisions

---

## Research Task 1: reedline Completer Trait API

### Question
How does reedline's Completer trait work? What interface must we implement?

### Findings

**reedline Completer Trait** (from reedline documentation and source):

```rust
pub trait Completer: Send {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion>;
}

pub struct Suggestion {
    pub value: String,
    pub description: Option<String>,
    pub extra: Option<Vec<String>>,
    pub span: Span,
    pub append_whitespace: bool,
}
```

**Key Points**:
- `complete()` receives the full line buffer and cursor position
- Returns `Vec<Suggestion>` with replacement text and metadata
- `Span` defines what part of the line to replace
- `description` allows showing help text in menus
- `append_whitespace` controls whether space is added after completion

**reedline Built-in Completers**:
- `DefaultCompleter` - simple prefix matching
- `FilePathCompleter` - filesystem completion
- No built-in command or flag completers

**Integration with Reedline**:
```rust
use reedline::{Reedline, DefaultCompleter};

let completer = Box::new(MyCompleter::new());
let mut line_editor = Reedline::create()
    .with_completer(completer);
```

### Decision

**Use reedline's Completer trait** with custom implementations for each priority tier:
- Implement `CommandCompleter` for P1 (command completion)
- Implement `PathCompleter` for P2 (file/directory completion)
- Implement `FlagCompleter` for P3 (flag completion)
- Create `CompositeCompleter` that delegates to appropriate completer based on context

**Rationale**:
- reedline already integrated in rush REPL
- Completer trait is simple and well-designed
- No reinventing the wheel
- Pure Rust (aligns with constitution Principle V)

**Alternatives Considered**:
- Custom completion system from scratch (rejected: unnecessary complexity, reinventing wheel)
- Using bash-completion scripts (rejected: not Rust-native, requires external dependencies)

---

## Research Task 2: PATH Scanning on macOS

### Question
How to efficiently scan PATH for executables on macOS? How to handle permissions, symlinks, and performance?

### Findings

**PATH Environment Variable**:
```rust
use std::env;

let path = env::var("PATH").unwrap_or_default();
let directories: Vec<&str> = path.split(':').collect();
```

**Scanning Directories**:
```rust
use std::fs;

for dir in directories {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            // Check if executable
            #[cfg(unix)]
            use std::os::unix::fs::PermissionsExt;
            if entry.metadata().ok()?.permissions().mode() & 0o111 != 0 {
                // It's executable
            }
        }
    }
}
```

**Performance Considerations**:
- Typical macOS PATH: 10-20 directories
- Typical executables: 500-1000 total
- Scanning time: ~50-100ms on first access
- Solution: Lazy load and cache

**Symlink Handling**:
- Use `fs::metadata()` which follows symlinks
- Broken symlinks will fail metadata check (safe to skip)

**Permission Handling**:
- Some PATH directories may be unreadable (e.g., system directories with restricted permissions)
- Use `read_dir().ok()` to skip permission-denied directories
- No need to elevate privileges

### Decision

**Lazy-loaded PATH cache**:
1. On first Tab press, scan all PATH directories
2. Cache executable names in `HashSet<String>`
3. Cache is valid for entire session (PATH changes rare in single session)
4. For case-insensitive matching on macOS, store lowercase keys

**Rationale**:
- First completion may take 50-100ms (acceptable one-time cost)
- Subsequent completions are instant (<1ms from memory)
- Meets <100ms performance goal (spec SC-001)
- Aligns with constitution Performance-First principle

**Alternatives Considered**:
- Background thread scanning (rejected: unnecessary complexity for v0.1)
- Persistent cache on disk (rejected: adds complexity, violates Zero-Config principle)
- Re-scan on every completion (rejected: too slow, violates Performance-First)

---

## Research Task 3: Filesystem Completion Patterns

### Question
How to handle special cases: hidden files, spaces in paths, case sensitivity, large directories?

### Findings

**Hidden Files** (starting with `.`):
- Show in completions if user has typed `.` prefix
- Example: `ls .git` → show `.github/`, `.gitignore`
- Example: `ls` → don't show hidden files
- Reasoning: User intent is clear from input

**Spaces in Paths**:
- Completed paths must be properly quoted
- Example: `ls My Do<TAB>` → `ls "My Documents/"`
- Use `shlex` crate for proper shell quoting
- Or manually quote if contains spaces/special chars

**Case Sensitivity**:
- macOS: case-insensitive by default (HFS+/APFS default)
- Linux: case-sensitive
- Decision: Follow platform convention
```rust
#[cfg(target_os = "macos")]
const CASE_SENSITIVE: bool = false;
#[cfg(not(target_os = "macos"))]
const CASE_SENSITIVE: bool = true;
```

**Large Directories**:
- Some directories have thousands of entries (/usr/bin, node_modules)
- Listing 10,000+ files is slow and overwhelming
- Solution: Limit completions displayed to 50 items (per spec FR-010)
- Show message: "200+ matches, type more characters"

**Performance for Path Completion**:
- Use `fs::read_dir()` which is lazy (doesn't stat all files)
- Only metadata check on matched entries
- Filter as early as possible to minimize I/O

### Decision

**Filesystem completion strategy**:
1. Parse input to extract partial path
2. Determine parent directory and prefix to match
3. Read parent directory entries
4. Filter by prefix (case-sensitive/insensitive based on platform)
5. Show hidden files only if prefix starts with `.`
6. Limit results to 50 items
7. Quote completions with spaces

**Rationale**:
- Handles 95% of common scenarios (spec SC-003)
- Respects user intent (hidden files)
- Performance acceptable for typical directories
- Aligns with bash/zsh behavior (familiar UX)

**Alternatives Considered**:
- Always show hidden files (rejected: too noisy)
- No limit on results (rejected: overwhelming for large dirs)
- Always case-sensitive (rejected: frustrating on macOS)

---

## Research Task 4: Completion Menu UX

### Question
How should the completion menu work? Navigation, visual feedback, limiting matches?

### Findings

**reedline Menu Behavior**:
- Automatically displays menu when `Vec<Suggestion>` has multiple items
- User can navigate with Tab or arrow keys
- Enter selects, Escape cancels
- Menu shows `value` and optional `description`

**Best Practices from other shells**:
- bash: Shows simple list, cycle with Tab
- zsh: Rich menu with descriptions, arrow key navigation
- fish: Inline preview with descriptions, arrow keys

**Limiting Matches**:
- If >50 matches, show message instead of overwhelming menu
- Example: "154 matches for 'ca', type more characters"
- User types more → results narrow → menu appears when ≤50

**Menu Layout**:
```
$ git st<TAB>
status      Show the working tree status
stash       Stash the changes in a dirty working directory
stage       Add file contents to the staging area
```

**Visual Feedback**:
- Highlight matching prefix
- Show count when multiple matches
- Clear indicator when no matches

### Decision

**Completion menu design**:
1. Return up to 51 `Suggestion` items from completer
2. If 51 returned, show message: "50+ matches, type more"
3. If ≤50, show menu with descriptions (for flags)
4. For commands/paths: simple list without descriptions (less noise)
5. Let reedline handle navigation (Tab, arrows)

**Rationale**:
- Leverages reedline's built-in menu (less code)
- 50-item limit prevents overwhelming users (spec FR-010)
- Simple UX aligns with Zero-Config principle
- Descriptions for flags improve discoverability

**Alternatives Considered**:
- Unlimited menu (rejected: overwhelming)
- Fuzzy matching (rejected: out of scope per spec)
- Custom menu UI (rejected: unnecessary complexity)

---

## Research Task 5: Performance Optimization

### Question
How to ensure <100ms completion latency? What caching strategies? Memory management?

### Findings

**Performance Bottlenecks**:
1. PATH scanning: 50-100ms first time
2. Directory listing: 10-50ms for large dirs
3. String matching: <1ms (negligible)
4. Menu rendering: handled by reedline

**Caching Strategies**:

**CommandCompleter Cache**:
```rust
struct CommandCompleter {
    cache: Option<HashSet<String>>,  // Lazy-loaded
}

impl Completer for CommandCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        if self.cache.is_none() {
            self.cache = Some(self.scan_path());  // One-time scan
        }
        // Use cache for matching
    }
}
```

**PathCompleter Cache**:
- No persistent cache (directories change frequently)
- Cache directory listing per completion request
- Example: `ls src/r<TAB>` → cache `src/` entries for this completion
- Throw away after completion to avoid stale data

**FlagCompleter Cache**:
- Static data (flags don't change)
- Use `lazy_static!` or `once_cell` for compile-time initialization
```rust
use lazy_static::lazy_static;

lazy_static! {
    static ref GIT_FLAGS: Vec<&'static str> = vec!["--version", "--help", ...];
}
```

**Memory Management**:
- CommandCompleter cache: ~100-200KB (1000 commands × 100 bytes avg)
- PathCompleter: no persistent cache
- FlagCompleter: ~10-20KB static data
- Total: ~200KB additional memory (well within <10MB constitution limit)

**Lazy Loading**:
- PATH scan on first Tab press (not on startup)
- Avoids startup time penalty
- Aligns with Performance-First principle

### Decision

**Caching strategy**:
1. **CommandCompleter**: Lazy-loaded session cache (never invalidate)
2. **PathCompleter**: No cache (always fresh from filesystem)
3. **FlagCompleter**: Static compile-time data
4. All completers implement lazy initialization

**Performance targets** (from spec SC-001):
- First command completion: <100ms (scan + match)
- Subsequent command completions: <10ms (cache lookup)
- Path completion: <50ms typical (directory listing + match)
- Flag completion: <5ms (static data lookup)

**Rationale**:
- Meets <100ms requirement (spec SC-001)
- Minimal memory overhead (~200KB)
- No stale data issues
- Simple implementation

**Alternatives Considered**:
- Persistent disk cache (rejected: complexity, stale data risk)
- Background refresh (rejected: unnecessary for v0.1)
- No caching (rejected: too slow, violates Performance-First)

---

## Summary of Decisions

| Research Area | Decision | Rationale |
|---------------|----------|-----------|
| **Completer Interface** | Use reedline's `Completer` trait | Already integrated, simple, Rust-native |
| **PATH Scanning** | Lazy-loaded session cache | Fast after first use, meets <100ms goal |
| **Filesystem Patterns** | Platform-aware, prefix matching, 50-item limit | Handles 95% of cases, familiar UX |
| **Menu UX** | Leverage reedline menu, limit to 50 items | Simple, prevents overwhelming users |
| **Performance** | Lazy init + selective caching | Meets <100ms target, <10MB memory |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| PATH scan >100ms | Low | Medium | Acceptable one-time cost, subsequent completions cached |
| Large directory (10k+ files) | Medium | Medium | Limit to 50 results, show message |
| Permission denied on PATH dirs | Low | Low | Skip unreadable directories silently |
| Memory usage exceeds 10MB | Very Low | Low | Cache is ~200KB, well within limit |
| Symlink loops | Very Low | Low | OS handles via metadata, safe |

---

## Next Steps

Research complete. Ready for Phase 1 design:
1. Create `data-model.md` defining cache structures and entities
2. Create `contracts/completer-trait.md` documenting interfaces
3. Create `quickstart.md` for developer testing guide

All decisions documented. No blocking unknowns remaining.
