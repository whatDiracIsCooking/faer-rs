# faer/src/utils Module Documentation

## Overview

The `faer/src/utils` module contains utility functions and types used throughout the faer library. It includes compile-time bound-checked indexing, SIMD helpers, threading utilities, and approximate comparators for testing.

## Module Structure

```
faer/src/utils/
├── mod.rs      - Module exports and thread utilities
├── bound.rs    - Compile-time bound-checked indexing types
├── simd.rs     - SIMD helper utilities
├── approx.rs   - Approximate comparators for testing
└── slice.rs    - (appears to be empty/minimal)
```

---

## Files

### mod.rs

**Location**: `faer/src/utils/mod.rs`

**Purpose**: Exports submodules and provides thread parallelism utilities.

**Key Components**:

#### Submodule Exports
- `pub mod bound` - Compile-time bound-checked indexing types
- `pub mod approx` - Approximate comparators for testing purposes
- `pub mod simd` - SIMD helper utilities based on lifetime-bound indices

#### Thread Utilities (`mod thread`)

**`join_raw` function** (lines 10-50)
- Executes two operations, possibly in parallel
- Splits parallelism between the two operations
- Parameters:
  - `op_a: impl Send + FnOnce(Par)` - First operation
  - `op_b: impl Send + FnOnce(Par)` - Second operation
  - `parallelism: Par` - Parallelism level
- Implementation:
  - `Par::Seq` - Executes sequentially
  - `Par::Rayon(n_threads)` - Uses rayon for parallel execution if `n_threads > 1`, splits thread count between operations

**`Ptr<T>` type** (lines 52-61)
- Unsafe `Send` and `Sync` pointer wrapper
- Fields: `pub *mut T`
- Implements: `Copy`, `Clone`, `Send`, `Sync`

**`parallelism_degree` function** (lines 64-71)
- Returns the number of threads for a given parallelism level
- Returns 1 for `Par::Seq`, `n_threads.get()` for `Par::Rayon`

**`par_split_indices` function** (lines 78-95)
- Splits a range `0..n` into subsegments for parallel processing
- Parameters:
  - `n: usize` - Total range length
  - `idx: usize` - Consumer index
  - `chunk_count: usize` - Number of parallel consumers
- Returns: `(start: usize, length: usize)` for the subsegment
- Distributes remainder evenly across first chunks

---

### bound.rs

**Location**: `faer/src/utils/bound.rs`

**Purpose**: Provides compile-time bound-checked indexing using lifetime branding to ensure indices are valid for their associated dimensions.

**Key Types**:

#### `Dim<'n>` (lines 33-245)
- Lifetime-branded length/dimension
- Invariant: All instances with same lifetime correspond to same length
- Key methods:
  - `with<R>(dim: usize, f: impl for<'dim> FnOnce(Dim<'dim>) -> R) -> R` - Create new branded value
  - `new_unbound(dim: usize)` (unsafe) - Create with arbitrary brand
  - `new(dim: usize, guard: Guard<'n>)` - Create with unique brand
  - `unbound(self) -> usize` - Get unconstrained value
  - `partition<'head, 'tail>` - Split into two segments
  - `head_partition` - Partition from head
  - `advance(start: Idx<'n>, len: usize) -> IdxInc<'n>` - Advance index by length
  - `indices()` - Iterator over all valid indices
  - `par_indices()` - Parallel iterator (requires `rayon` feature)
  - `check<I: Index>(idx: I) -> Idx<'n, I>` - Check and convert to bounded index
  - `idx<I: Index>(idx: I) -> Idx<'n, I>` - Alias for `check`
  - `idx_inc<I: Index>(idx: I) -> IdxInc<'n, I>` - Check and convert to inclusive index
  - `try_check<I: Index>(idx: I) -> Option<Idx<'n, I>>` - Try to check, return None if out of bounds

#### `Idx<'n, I: Index>` (lines 66-304)
- Lifetime-branded index (exclusive upper bound)
- Invariant: All instances are valid indices for `Dim<'n>` and `<= I::Signed::MAX`
- Key methods:
  - `new_unbound(idx: I)` (unsafe) - Create with arbitrary brand
  - `new_unchecked(idx: I, dim: Dim<'n>)` (unsafe) - Create assuming valid
  - `new_checked(idx: I, dim: Dim<'n>)` - Create with bounds check (panics if invalid)
  - `unbound(self) -> I` - Get unconstrained value
  - `zx(self) -> Idx<'n>` - Zero-extend to `usize`
  - `truncate<I: Index>(self) -> Idx<'n, I>` - Truncate to smaller type
  - `to_incl() -> IdxInc<'n, I>` - Convert to inclusive index
  - `next() -> IdxInc<'n, I>` - Get next index
  - `excl() -> IdxInc<'n, I>` - Same as `to_incl()`
  - `from_slice_mut_checked/ref_checked` - Convert slices with bounds checking
  - `from_slice_mut_unchecked/ref_unchecked` (unsafe) - Convert slices without checking

#### `IdxInc<'n, I: Index>` (lines 75-385)
- Lifetime-branded partition index (inclusive upper bound)
- Invariant: All instances are valid partition places for `Dim<'n>` and `<= I::Signed::MAX`
- Key methods:
  - `ZERO: Self` - Constant zero index
  - `new_unbound(idx: I)` (unsafe) - Create with arbitrary brand
  - `new_unchecked(idx: I, dim: Dim<'n>)` (unsafe) - Create assuming valid
  - `new_checked(idx: I, dim: Dim<'n>)` - Create with bounds check
  - `unbound(self) -> I` - Get unconstrained value
  - `zx(self) -> IdxInc<'n>` - Zero-extend to `usize`
  - `to(upper: IdxInc<'n>)` - Iterator from self to upper
  - `range_to(upper: IdxInc<'n>)` - Same as `to()`

#### `MaybeIdx<'n, I: Index>` (lines 564-687)
- Index value or `None` (uses negative values for `None`)
- Key methods:
  - `from_index(idx: Idx<'n, I>)` - Create from valid index
  - `none()` - Create `None` value
  - `new_checked(idx: I::Signed, size: Dim<'n>)` - Create with bounds check
  - `new_unchecked(idx: I::Signed, size: Dim<'n>)` (unsafe) - Create without checking
  - `new_unbound(idx: I)` (unsafe) - Create with arbitrary brand
  - `unbound(self) -> I` - Get inner value
  - `idx(self) -> Option<Idx<'n, I>>` - Convert to Option
  - `sx(self) -> MaybeIdx<'n>` - Sign-extend to `usize`
  - `from_slice_mut_checked/ref_checked` - Convert slices with bounds checking
  - `as_slice_ref` - Convert to unconstrained slice

#### `Partition<'head, 'tail, 'n>` (lines 7-32)
- Represents a split of a range into two segments
- Fields:
  - `head: Dim<'head>` - Size of first half
  - `tail: Dim<'tail>` - Size of second half
- Methods:
  - `midpoint() -> IdxInc<'n>` - Get split point
  - `flip() -> Partition<'tail, 'head, 'n>` - Swap head and tail

#### `Array<'n, T>` (lines 720-829)
- Array with length tied to lifetime `'n`
- Key methods:
  - `from_ref(slice: &[T], size: Dim<'n>)` - Create from slice with length check
  - `from_mut(slice: &mut [T], size: Dim<'n>)` - Mutable version
  - `as_ref()` - Get unconstrained slice
  - `as_mut()` - Get mutable unconstrained slice
  - `len() -> Dim<'n>` - Get length
- Indexing: Implements `Index` and `IndexMut` for `Idx<'n>` and `Range<IdxInc<'n>>`

#### Compile-time Size Types

**`One`** (line 832)
- Dimension equal to one
- `const IS_BOUND: bool = true`

**`Zero`** (line 834)
- Index equal to zero

**`IdxIncOne<I: Index>`** (line 838)
- Index equal to zero or one

**`MaybeIdxOne<I: Index>`** (line 843)
- Index equal to zero, one, or a sentinel value

---

### simd.rs

**Location**: `faer/src/utils/simd.rs`

**Purpose**: SIMD helper utilities using lifetime-bound indices for safe vectorized operations.

**Key Types**:

#### `SimdCtx<'N, T: ComplexField, S: Simd>` (lines 6-490)
- SIMD context with lifetime-branded length
- Fields:
  - `ctx: T::SimdCtx<S>` - SIMD context from `ComplexField`
  - `len: Dim<'N>` - Length of the operation
  - `offset: usize` - Alignment offset
  - `head_end, body_end, tail_end: usize` - Segment boundaries
  - `head_mask, tail_mask: T::SimdMask<S>` - Masks for partial loads/stores
  - `head_mem_mask, tail_mem_mask: T::SimdMemMask<S>` - Memory masks

**Key Methods**:

- **`new(simd: T::SimdCtx<S>, len: Dim<'N>)`** (lines 157-198)
  - Creates SIMD context without alignment
  - Computes masks for head/tail segments
  - Asserts `T::SIMD_CAPABILITIES == SimdCapabilities::Simd`

- **`new_align(simd: T::SimdCtx<S>, len: Dim<'N>, align_offset: usize)`** (lines 200-305)
  - Creates SIMD context with alignment offset
  - Handles partial first vector (head)
  - Three cases:
    1. `align_offset == 0` - No alignment needed
    2. `align_offset <= len` - Normal case with head, body, tail
    3. `align_offset > len` - Entire data fits in one masked vector

- **`new_force_mask(simd: T::SimdCtx<S>, len: Dim<'N>)`** (lines 312-357)
  - Forces use of masked operations
  - Always creates a tail segment
  - Requires `len != 0`

- **`offset(&self) -> usize`** (line 308)
  - Returns alignment offset

- **`read<I: SimdIndex<'N, T, S>>`** (lines 359-366)
  - Reads SIMD vector from slice at given index

- **`write<I: SimdIndex<'N, T, S>>`** (lines 368-376)
  - Writes SIMD vector to slice at given index

- **`head_mask(&self) -> T::SimdMask<S>`** (line 379)
  - Returns mask for head segment

- **`tail_mask(&self) -> T::SimdMask<S>`** (line 384)
  - Returns mask for tail segment

- **`indices(&self)`** (lines 388-430)
  - Returns iterator structure for SIMD operations
  - Returns: `(Option<SimdHead>, Iterator<SimdBody>, Option<SimdTail>)`
  - Iterates over aligned SIMD-width chunks

- **`batch_indices<const BATCH: usize>(&self)`** (lines 432-489)
  - Returns batched iterators for loop unrolling
  - Returns: `(head, batched_body, remaining_body, tail)`
  - Useful for processing multiple SIMD vectors per iteration

**Implements `Deref` to `faer_traits::SimdCtx<T, S>`** (lines 38-45)

#### `SimdBody<'N, T: ComplexField, S: Simd>` (lines 491-520)
- Index for aligned SIMD body operations
- Field: `start: isize` - Offset from base pointer
- Methods:
  - `offset(&self) -> isize` - Get offset

#### `SimdHead<'N, T: ComplexField, S: Simd>` (lines 502-527)
- Index for partial head SIMD operations (with mask)
- Field: `start: isize` - Offset from base pointer

#### `SimdTail<'N, T: ComplexField, S: Simd>` (lines 508-534)
- Index for partial tail SIMD operations (with mask)
- Field: `start: isize` - Offset from base pointer

#### `SimdIndex<'N, T: ComplexField, S: Simd>` trait (lines 46-58)
- Trait for SIMD indexing types
- Methods:
  - `read(simd, slice, index) -> T::SimdVec<S>`
  - `write(simd, slice, index, value)`

**Implementations**:

- **`SimdBody`** (lines 59-89): Uses aligned loads/stores
- **`SimdHead`** (lines 90-122): Uses masked loads/stores with `head_mem_mask`
- **`SimdTail`** (lines 123-155): Uses masked loads/stores with `tail_mem_mask`

---

### approx.rs

**Location**: `faer/src/utils/approx.rs`

**Purpose**: Approximate equality comparators for testing floating-point values.

**Key Types**:

#### `ApproxEq<T>` (lines 5-29)
- Approximate equality comparator
- Fields:
  - `abs_tol: T` - Absolute tolerance
  - `rel_tol: T` - Relative tolerance
- Methods:
  - `eps() -> Self` - Creates comparator with `128 * machine_epsilon` tolerance
  - `mul(self, rhs: Real<T>) -> ApproxEq<T>` - Scale tolerances
- Implements `equator::Cmp<T, T>` (lines 48-58):
  - Checks if `|lhs - rhs| <= abs_tol` OR `|lhs - rhs| <= rel_tol * max(|lhs|, |rhs|)`

#### `ApproxEqError` (line 32)
- Error type for approximate equality failures

#### `CwiseMat<Cmp>` (line 10)
- Component-wise matrix comparator wrapper
- Implements `equator::Cmp<L, R>` for matrices (lines 59-85):
  - Compares dimensions first
  - Then compares each element using wrapped comparator

#### Error Types

**`CwiseMatError<Rows: Shape, Cols: Shape, Error>`** (lines 34-37)
```rust
enum CwiseMatError {
    DimMismatch,
    Elements(Vec<(Idx<Rows>, Idx<Cols>, Error)>),
}
```

**`CwiseColError<Rows: Shape, Error>`** (lines 39-42)
```rust
enum CwiseColError {
    DimMismatch,
    Elements(Vec<(Idx<Rows>, Error)>),
}
```

**`CwiseRowError<Cols: Shape, Error>`** (lines 44-47)
```rust
enum CwiseRowError {
    DimMismatch,
    Elements(Vec<(Idx<Cols>, Error)>),
}
```

---

### slice.rs

**Location**: `faer/src/utils/slice.rs`

**Status**: Appears to be empty or minimal (only 1 line)

---

## Design Patterns

### Lifetime Branding

The core design pattern in `bound.rs` is **lifetime branding** using generativity:

1. A `Dim<'n>` value brands a specific runtime length with a unique lifetime `'n`
2. `Idx<'n>` and `IdxInc<'n>` values with the same lifetime are guaranteed to be valid for that dimension
3. The type system ensures indices can't be mixed between different dimensions
4. At runtime, the values are just `usize`, but the type system provides compile-time safety

**Example**:
```rust
Dim::with(5, |dim| {
    let idx = dim.check(3); // Idx<'n> where 'n is tied to dim
    // idx is guaranteed valid for any Array<'n, T> with same 'n
});
```

### SIMD Safety

The `simd.rs` module uses lifetime branding to ensure SIMD operations are safe:

1. `SimdCtx<'N, T, S>` is tied to a specific length via `Dim<'N>`
2. `SimdHead`, `SimdBody`, `SimdTail` indices are branded with the same lifetime
3. Operations can only use indices that match the context's length
4. Masks ensure partial loads/stores at boundaries are safe

### Parallelism Abstraction

The `thread` module provides a lightweight abstraction over parallel execution:

- `Par::Seq` - Sequential execution
- `Par::Rayon(n_threads)` - Parallel execution using rayon
- `join_raw` splits work between two operations based on parallelism level

---

## Dependencies

- `core` - Core Rust types
- `faer_traits` - Trait definitions (ComplexField, SimdCapabilities, etc.)
- `pulp` - SIMD abstraction (`Simd` trait)
- `generativity` - Lifetime branding (`Guard`)
- `equator` - Comparison traits and assertions
- `bytemuck` - Safe transmutation
- `rayon` (optional) - Parallel iterators
- `spindle` (rayon feature) - Work stealing

---

## Usage Notes

### Safety

- Many functions are marked `unsafe` because they bypass runtime bounds checking
- The lifetime branding system provides compile-time guarantees that make these operations safe when used correctly
- Users should prefer safe constructors (`new_checked`, `check`) unless performance is critical

### Performance

- In release builds, bounded indices use unchecked indexing for zero overhead
- In debug builds, bounds are still checked for safety
- SIMD operations automatically handle alignment and partial vectors

### Testing

- `approx.rs` provides utilities for floating-point comparison in tests
- Use `ApproxEq::eps()` for default tolerance
- Scale tolerance with `*` operator for stricter/looser comparisons

---

## Summary

The `faer/src/utils` module provides foundational utilities for safe and efficient numerical computing:

- **Compile-time bounds checking** via lifetime branding (`bound.rs`)
- **SIMD vectorization helpers** with automatic masking (`simd.rs`)
- **Parallelism abstraction** for rayon integration (`mod.rs`)
- **Approximate equality** for floating-point testing (`approx.rs`)

These utilities enable the rest of faer to maintain both safety and performance by leveraging Rust's type system to enforce invariants at compile time while generating efficient runtime code.
