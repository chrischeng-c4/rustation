# Feature Specification: String Manipulation

**Feature Branch**: `033-string-manipulation`
**Created**: 2025-12-08
**Status**: Draft
**Dependencies**: 032-parameter-expansion

## Overview

Add string manipulation operators to parameter expansion, allowing prefix/suffix removal and pattern replacement.

## Operators

### Prefix Removal
- `${var#pattern}` - Remove shortest prefix matching pattern
- `${var##pattern}` - Remove longest prefix matching pattern

### Suffix Removal
- `${var%pattern}` - Remove shortest suffix matching pattern
- `${var%%pattern}` - Remove longest suffix matching pattern

### Pattern Replacement
- `${var/pattern/replacement}` - Replace first match
- `${var//pattern/replacement}` - Replace all matches
- `${var/#pattern/replacement}` - Replace prefix match
- `${var/%pattern/replacement}` - Replace suffix match

### Pattern Syntax
- `*` - Match any string (including empty)
- `?` - Match any single character
- Literal characters match themselves

## Examples

```bash
path="/usr/local/bin/script.sh"

# Prefix removal
${path#*/}       # usr/local/bin/script.sh (shortest)
${path##*/}      # script.sh (longest)

# Suffix removal
${path%/*}       # /usr/local/bin (shortest)
${path%%/*}      # (empty - longest)

# File extension handling
filename="archive.tar.gz"
${filename%.*}   # archive.tar (remove one extension)
${filename%%.*}  # archive (remove all extensions)
${filename##*.}  # gz (get extension)

# Replacement
str="hello world world"
${str/world/universe}   # hello universe world
${str//world/universe}  # hello universe universe
${str/#hello/hi}        # hi world world
${str/%world/earth}     # hello world earth
```

## Functional Requirements

- **FR-001**: System MUST implement `${var#pattern}` for shortest prefix removal
- **FR-002**: System MUST implement `${var##pattern}` for longest prefix removal
- **FR-003**: System MUST implement `${var%pattern}` for shortest suffix removal
- **FR-004**: System MUST implement `${var%%pattern}` for longest suffix removal
- **FR-005**: System MUST implement `${var/pat/rep}` for first replacement
- **FR-006**: System MUST implement `${var//pat/rep}` for all replacements
- **FR-007**: System MUST implement `${var/#pat/rep}` for prefix replacement
- **FR-008**: System MUST implement `${var/%pat/rep}` for suffix replacement
- **FR-009**: Pattern matching MUST support `*` (any string) and `?` (single char)
- **FR-010**: Empty pattern MUST match empty string
- **FR-011**: Missing replacement MUST default to empty string (deletion)

## Success Criteria

- All string manipulation operators work correctly
- Pattern matching with * and ? works as expected
- Edge cases (empty var, empty pattern, no match) handled correctly
- All tests pass
