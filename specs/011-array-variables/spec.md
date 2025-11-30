# Feature Specification: Array Variables

**Feature Branch**: `011-array-variables`
**Created**: 2025-11-30
**Status**: Draft

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create and Access Arrays (Priority: P1)

Users want to create arrays and access individual elements. For example, `arr=(one two three)` creates an array, and `${arr[0]}` accesses the first element.

**Why this priority**: Core functionality - without creating and accessing arrays, the feature cannot work.

**Independent Test**: Create an array and access elements to verify they're stored and retrieved correctly.

**Acceptance Scenarios**:

1. **Given** `arr=(one two three)`, **When** executed, **Then** array is created with three elements
2. **Given** array created, **When** accessing `${arr[0]}`, **Then** returns `one`
3. **Given** array elements, **When** accessing with index, **Then** correct element returned

---

### User Story 2 - Array Expansion (Priority: P1)

Users want to expand entire arrays in commands. For example, `${arr[@]}` expands to all elements, which can be used to pass array elements as arguments to other commands.

**Why this priority**: Essential for using arrays as function arguments. Core feature.

**Independent Test**: Expand array and verify all elements are included as separate arguments.

**Acceptance Scenarios**:

1. **Given** `arr=(one two three)`, **When** executing `echo ${arr[@]}`, **Then** outputs `one two three`
2. **Given** array expansion, **When** used as arguments, **Then** each element becomes separate argument
3. **Given** array with empty elements, **When** expanded, **Then** empty elements preserved

---

### User Story 3 - Array Assignment and Modification (Priority: P2)

Users want to add elements to arrays, modify existing elements, and append to arrays. For example, `arr[3]=four` adds a fourth element, or `arr+=(four five)` appends elements.

**Why this priority**: Important for practical array manipulation. Less critical than basic access/expansion.

**Independent Test**: Create array, modify elements, and verify changes persist.

**Acceptance Scenarios**:

1. **Given** array with elements, **When** executing `arr[3]=four`, **Then** new element added at index 3
2. **Given** array, **When** executing `arr[0]=modified`, **Then** first element changed
3. **Given** array, **When** executing `arr+=(new1 new2)`, **Then** elements appended to end

---

### User Story 4 - Array Length and Iteration (Priority: P2)

Users want to get array length and iterate over arrays in loops. For example, `${#arr[@]}` returns the number of elements, useful in `for i in ${arr[@]}` loops.

**Why this priority**: Important for loops and scripting. Essential for practical use.

**Independent Test**: Get array length and iterate in loop, verifying all elements are visited.

**Acceptance Scenarios**:

1. **Given** `arr=(one two three)`, **When** executing `${#arr[@]}`, **Then** returns `3`
2. **Given** array, **When** using `for item in ${arr[@]}`, **Then** each element iterated once
3. **Given** empty array, **When** checking length, **Then** returns `0`

---

### User Story 5 - Array Element Deletion (Priority: P2)

Users want to delete array elements or unset entire arrays. For example, `unset arr[1]` removes the second element, or `unset arr` removes the entire array.

**Why this priority**: Important for array management. Less critical than basic operations.

**Independent Test**: Delete array elements and verify they're removed.

**Acceptance Scenarios**:

1. **Given** array with elements, **When** executing `unset arr[1]`, **Then** element at index 1 removed
2. **Given** array, **When** executing `unset arr`, **Then** entire array removed
3. **Given** array with deleted element, **When** accessing deleted index, **Then** returns empty

---

### Edge Cases

- Arrays with spaces in elements?
- Index ranges or slicing?
- Associative arrays vs indexed arrays?
- Array modification during iteration?
- Empty arrays and sparse arrays?
- Large arrays with many elements?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support array creation with `(elem1 elem2 elem3)` syntax
- **FR-002**: System MUST support indexed array access with `${arr[index]}` syntax
- **FR-003**: System MUST support array expansion with `${arr[@]}` to get all elements
- **FR-004**: System MUST support `${#arr[@]}` to get array length
- **FR-005**: System MUST support adding elements with `arr[index]=value`
- **FR-006**: System MUST support appending with `arr+=(new_elements)`
- **FR-007**: System MUST support element deletion with `unset arr[index]`
- **FR-008**: System MUST support array unset with `unset arr`
- **FR-009**: System MUST preserve array across command execution
- **FR-010**: System MUST properly handle arrays in loops and expansions

### Key Entities

- **Array**: Ordered collection of string values indexed by integers
- **Array Element**: Individual value stored in array at specific index
- **Array Index**: Integer position of element (0-based)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can create indexed arrays with 100% bash compatibility
- **SC-002**: Array access and expansion work correctly in all contexts
- **SC-003**: Array modification (add, delete, append) works reliably
- **SC-004**: Array length and iteration work correctly
- **SC-005**: All acceptance scenarios pass
- **SC-006**: Arrays persist and work with other shell features

## Assumptions

- Indexed arrays (0-based) are supported; associative arrays marked as optional
- Arrays store string values
- Elements can contain spaces if quoted
- Array operations follow bash conventions
- Sparse arrays (gaps in indices) are supported
- Unset elements are treated as empty strings

## Dependencies

- Builds on variable system (014)
- Depends on command parsing for array syntax recognition
- Integrates with loop constructs for iteration
- Works with expansion system for `${arr[@]}`

## Notes

- Arrays are fundamental for advanced shell scripting
- Bash-compatible syntax for maximum portability
- Performance important for large arrays
- Sparse arrays should be handled efficiently
