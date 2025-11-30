# Technical Plan: Array Variables (Feature 011)

**Branch**: `011-array-variables` | **Status**: Ready for Implementation

## Summary

Implement bash-style indexed arrays: `arr=(a b c)`, `arr[0]=value`, `${arr[@]}`, `${arr[*]}`, `${#arr[@]}`

## Architecture

**Components**:
- **ArrayValue**: New variable type (enum) storing Vec<String>
- **ArrayIndexing**: Parser support for `arr[i]` syntax
- **ArrayExpansion**: Expand `${arr[@]}` and `${arr[*]}`
- **ArrayModification**: Support `arr[i]=value` and `arr+=(val)`

**Integration Points**:
- `executor/variables.rs`: Add ArrayValue variant
- `executor/parser.rs`: Add array syntax parsing
- `executor/expansion.rs`: Handle array expansion

## Critical Decisions

1. **Zero-based indexing** (matches bash)
2. **Sparse arrays supported** (missing indices allowed)
3. **Negative indices unsupported** (simplicity)
4. **`${arr[@]}` vs `${arr[*]}`**: First expands to separate words, second to single string
5. **Max array size**: No limit (practical memory limit)

## Implementation Phases

**Phase 1** (1-2 days): Array type + storage
**Phase 2** (2-3 days): Parsing + syntax support
**Phase 3** (2-3 days): Expansion + integration
**Phase 4** (1-2 days): Operators (+=, delete)

## User Stories

- US1: Create arrays `arr=(a b c)`
- US2: Access elements `${arr[0]}`
- US3: Get array length `${#arr[@]}`
- US4: Iterate arrays `for x in "${arr[@]}"`
- US5: Modify arrays `arr[1]=new`, `arr+=(x)`

---

**Estimated Duration**: 6-10 hours
**Complexity**: Medium-High
