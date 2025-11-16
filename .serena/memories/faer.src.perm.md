# faer/src/perm - Permutation Module

## Overview
The `faer/src/perm` module provides permutation matrix functionality for the faer linear algebra library. It includes both owned and borrowed permutation types, along with operations for applying permutations to matrix rows and columns.

## Module Structure

### Files
- `mod.rs` - Main module file with public API and permutation operations
- `permref.rs` - Immutable permutation view/reference type
- `permown.rs` - Owned permutation type

## Key Types

### `Perm<I, N>` (Owned Permutation)
Located in: `permown.rs`

```rust
pub struct Own<I: Index, N: Shape = usize> {
    forward: alloc::boxed::Box<[N::Idx<I>]>,
    inverse: alloc::boxed::Box<[N::Idx<I>]>,
}
```

- Owns the permutation data via boxed slices
- Stores both forward and inverse permutation mappings
- Generic over index type `I` and shape `N`

**Key Methods:**
- `new_checked()` - Creates permutation with validation
- `new_unchecked()` - Creates permutation without validation (unsafe)
- `into_arrays()` - Consumes and returns forward/inverse arrays
- `arrays()` - Borrows forward/inverse arrays
- `len()` - Returns dimension of permutation
- `into_inverse()` - Consumes and returns inverse permutation
- `inverse()` - Returns inverse permutation view
- `transpose()` - Alias for `inverse()`
- `as_shape()` - Returns permutation with different shape type
- `into_shape()` - Consumes and converts to different shape type

### `PermRef<'a, I, N>` (Permutation Reference)
Located in: `permref.rs`

```rust
pub struct Ref<'a, I: Index, N: Shape = usize> {
    forward: &'a [N::Idx<I>],
    inverse: &'a [N::Idx<I>],
}
```

- Immutable view of permutation data
- Copy + Clone (cheap to copy)
- References both forward and inverse permutation mappings

**Key Methods:**
- `new_checked()` - Creates permutation view with validation
- `new_unchecked()` - Creates permutation view without validation (unsafe)
- `arrays()` - Returns tuple of forward/inverse slices
- `bound_arrays()` - Returns bound array references (for `Dim<'N>`)
- `len()` - Returns dimension
- `inverse()` - Returns inverse permutation
- `transpose()` - Alias for `inverse()`
- `canonicalized()` - Converts to fixed-width index type
- `uncanonicalized()` - Converts from fixed-width index type
- `as_shape()` - Reshapes permutation view

### Generic Wrapper
Located in: `mod.rs`

```rust
pub mod generic {
    pub struct Perm<Inner>(pub Inner);
}
```

- Generic wrapper type that works with both `Own` and `Ref`
- Implements `Deref`, `DerefMut`, `Reborrow`, `ReborrowMut`, `IntoConst`
- Allows unified interface for owned and borrowed permutations

## Core Operations

### Row/Column Swapping

**`swap_cols<N, T>(a: ColMut<'_, T, N>, b: ColMut<'_, T, N>)`** (mod.rs:34)
- Swaps values in two columns
- Uses `zip!` macro for efficient element-wise swapping

**`swap_rows<N, T>(a: RowMut<'_, T, N>, b: RowMut<'_, T, N>)`** (mod.rs:68)
- Swaps values in two rows
- Implemented via transpose + `swap_cols`

**`swap_rows_idx<M, N, T>(mat: MatMut<'_, T, M, N>, a: Idx<M>, b: Idx<M>)`** (mod.rs:98)
- Swaps rows at given indices
- Optimizes out swap when `a == b`

**`swap_cols_idx<M, N, T>(mat: MatMut<'_, T, M, N>, a: Idx<N>, b: Idx<N>)`** (mod.rs:135)
- Swaps columns at given indices
- Optimizes out swap when `a == b`

### Permutation Application

**`permute_rows<I, T>(dst: MatMut<'_, T>, src: MatRef<'_, T>, perm_indices: PermRef<'_, I>)`** (mod.rs:256)
- Applies row permutation from source to destination matrix
- Optimizes based on stride (row-major vs column-major)
- Two strategies:
  - Element-by-element when row stride < col stride
  - Row-by-row copy when col stride < row stride

**`permute_cols<I, T>(dst: MatMut<'_, T>, src: MatRef<'_, T>, perm_indices: PermRef<'_, I>)`** (mod.rs:230)
- Applies column permutation from source to destination
- Implemented via transpose + `permute_rows`

**`permute_rows_in_place<I, T>(matrix: MatMut<'_, T>, perm_indices: PermRef<'_, I>, stack: &mut MemStack)`** (mod.rs:320)
- Applies row permutation in-place using temporary storage
- Requires workspace allocation (see `permute_rows_in_place_scratch`)

**`permute_cols_in_place<I, T>(matrix: MatMut<'_, T>, perm_indices: PermRef<'_, I>, stack: &mut MemStack)`** (mod.rs:350)
- Applies column permutation in-place using temporary storage
- Requires workspace allocation (see `permute_cols_in_place_scratch`)

### Workspace Calculation

**`permute_rows_in_place_scratch<I, T>(nrows: usize, ncols: usize) -> StackReq`** (mod.rs:297)
- Computes workspace requirement for in-place row permutation
- Returns `StackReq` for dynamic stack allocation

**`permute_cols_in_place_scratch<I, T>(nrows: usize, ncols: usize) -> StackReq`** (mod.rs:305)
- Computes workspace requirement for in-place column permutation
- Returns `StackReq` for dynamic stack allocation

## Design Patterns

### Index Type Generics
- Permutations are generic over index type `I: Index`
- Supports different index widths (e.g., `u32`, `usize`)
- `canonicalized()` / `uncanonicalized()` for type conversions

### Shape System
- Generic over `N: Shape` (can be `usize` or `Dim<'N>`)
- `Dim<'N>` provides compile-time bounds checking
- `as_shape()` / `into_shape()` for shape conversions

### Reborrow Pattern
- Implements `Reborrow` and `ReborrowMut` traits
- Allows temporary borrowing without consuming owned values
- `rb()` and `rb_mut()` methods for ergonomic borrowing

### Dual Representation
- Stores both forward and inverse permutation arrays
- Enables O(1) inverse computation
- Trades memory for performance

## Validation

### Checked Construction
Both `Perm` and `PermRef` validate:
1. Arrays have same length
2. Length ≤ `I::Signed::MAX`
3. All indices are valid (< n)
4. Forward and inverse are true inverses: `inverse[forward[i]] == i`

### Safety Invariants
Unchecked constructors require:
- Valid permutation arrays (each index 0..n appears exactly once)
- Forward and inverse are true inverses
- Length within index type bounds

## Usage Examples

### Swapping Columns (mod.rs:12-31)
```rust
let mut m = mat![
    [1.0, 2.0, 3.0],
    [4.0, 5.0, 6.0],
    [7.0, 8.0, 9.0],
    [10.0, 14.0, 12.0],
];
let (a, b) = m.two_cols_mut(0, 2);
perm::swap_cols(a, b);
// Result: columns 0 and 2 are swapped
```

### Swapping Rows (mod.rs:46-64)
```rust
let mut m = mat![
    [1.0, 2.0, 3.0],
    [4.0, 5.0, 6.0],
    [7.0, 8.0, 9.0],
    [10.0, 14.0, 12.0],
];
let (a, b) = m.two_rows_mut(0, 2);
perm::swap_rows(a, b);
// Result: rows 0 and 2 are swapped
```

## Performance Considerations

### Stride-Based Optimization
- `permute_rows` chooses strategy based on matrix layout
- Row-major: prefers row-by-row copying
- Column-major: prefers element-by-element access

### In-Place Operations
- Require temporary storage allocation
- Use `dyn_stack::MemStack` for efficient stack-based allocation
- Workspace size = full matrix size

### Transpose Optimization
- Column operations implemented via transpose of row operations
- Leverages existing row operation implementations
- Avoids code duplication

## Dependencies
- `crate::Idx` - Index wrapper type
- `crate::internal_prelude::*` - Core matrix types
- `dyn_stack::StackReq` - Dynamic stack allocation
- `linalg::zip` - Element-wise iteration
- `reborrow` - Reborrowing traits
- `alloc::boxed::Box` - Heap allocation (for owned permutations)

## Type Aliases
```rust
pub type Perm<I, N = usize> = generic::Perm<Own<I, N>>;
pub type PermRef<'a, I, N = usize> = generic::Perm<Ref<'a, I, N>>;
```

## Module Exports
```rust
pub use permown::Own;
pub use permref::Ref;
pub mod generic { /* ... */ }
```
