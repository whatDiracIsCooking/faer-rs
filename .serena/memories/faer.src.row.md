# faer/src/row Module Documentation

## Overview
The `row` module provides row vector types for the faer linear algebra library. It implements three main row types with different ownership semantics, along with indexing and slicing capabilities.

## File Structure
- `mod.rs` - Module definitions, type aliases, and traits
- `rowref.rs` - Immutable row reference implementation (`Ref`)
- `rowown.rs` - Owned row implementation (`Own`)
- `rowmut.rs` - Mutable row reference implementation (`Mut`)
- `row_index.rs` - Indexing trait implementations

## Core Types

### Main Row Types
All types are defined in `mod.rs` as type aliases over `generic::Row<Inner>`:

1. **`RowRef<'a, T, Cols = usize, CStride = isize>`**
   - Immutable view over a row vector
   - Similar to an immutable reference to a strided slice
   - Allows partially/fully uninitialized data under certain conditions
   - Type alias: `generic::Row<Ref<'a, T, Cols, CStride>>`

2. **`RowMut<'a, T, Cols = usize, CStride = isize>`**
   - Mutable view over a row vector
   - Similar to a mutable reference to a strided slice
   - Allows partially/fully uninitialized data under certain conditions
   - Type alias: `generic::Row<Mut<'a, T, Cols, CStride>>`

3. **`Row<T, Cols = usize>`**
   - Heap-allocated resizable row vector
   - Guaranteed row-major layout (column stride = 1)
   - Type alias: `generic::Row<Own<T, Cols>>`

### Generic Wrapper (`mod.rs:56-159`)
```rust
#[repr(transparent)]
pub struct Row<Inner>(pub Inner);
```
- Transparent wrapper that implements `Deref` and `DerefMut`
- Implements `Reborrow`, `ReborrowMut`, `IntoConst` traits
- Provides indexing via `Index<Idx<Cols>>` and `IndexMut<Idx<Cols>>`

## Key Traits (`mod.rs`)

### `RowIndex<ColRange>` (lines 6-13)
Represents types that can be used to slice a row (index or range).
- `get()` - Slice with bound checks
- `get_unchecked()` - Slice without bound checks

### `AsRowRef` (lines 166-169)
Trait for types convertible to immutable row view.
- Requires `AsMatRef<Rows = One>`
- `as_row_ref()` - Returns `RowRef`

### `AsRowMut` (lines 161-164)
Trait for types convertible to mutable row view.
- Requires `AsRowRef`
- `as_row_mut()` - Returns `RowMut`

## Implementation Details

### rowref.rs - Immutable Row Reference

#### Struct Definition (lines 7-9)
```rust
pub struct Ref<'a, T, Cols = usize, CStride = isize> {
    pub(crate) trans: ColRef<'a, T, Cols, CStride>,
}
```
Internally stores a transposed column reference.

#### Key Constructors
- `from_ref(value: &'a T)` - Create from single element reference (line 56)
- `from_slice(slice: &'a [T])` - Create from slice (line 63)
- `from_raw_parts(ptr, ncols, col_stride)` - Unsafe constructor (line 77)

#### Core Methods
- **Shape queries**: `nrows()`, `ncols()`, `shape()`, `col_stride()`
- **Element access**: `ptr_at()`, `ptr_inbounds_at()`, `at()`, `at_unchecked()`
- **Slicing**: `split_at_col()`, `subcols()`, `get()`, `get_unchecked()`
- **Transformations**: `transpose()`, `conjugate()`, `adjoint()`, `canonical()`
- **Views**: `as_col_shape()`, `as_dyn_cols()`, `as_dyn_stride()`, `reverse_cols()`
- **Iteration**: `iter()`, `par_iter()` (with rayon feature)
- **Conversion**: `as_mat()`, `as_diagonal()`, `as_slice()` (for contiguous)
- **Operations**: `map()`, `for_each()` (lines 120-129)

#### Norms and Aggregates (lines 452-495)
- `norm_max()`, `norm_l2()`, `squared_norm_l2()`, `norm_l1()`
- `sum()` - Sum of all elements
- `kron()` - Kronecker product

#### Min/Max (lines 611-656)
- `max()`, `min()` - For `RealField` types
- Internal implementations: `internal_max()`, `internal_min()`

#### Tests (lines 657-678)
Unit tests for min/max functionality.

### rowown.rs - Owned Row

#### Struct Definition (lines 5-7)
```rust
pub struct Own<T, Cols: Shape = usize> {
    pub(crate) trans: Col<T, Cols>,
}
```
Internally stores a transposed column vector.

#### Constructors (lines 8-57)
- `from_fn(ncols, f)` - Create with function
- `zeros(ncols)` - Create filled with zeros
- `ones(ncols)` - Create filled with ones
- `full(ncols, value)` - Create filled with value

#### Memory Management (lines 59-98)
- `try_reserve(capacity)` - Try to reserve capacity
- `reserve(capacity)` - Reserve capacity (panics on failure)
- `resize_with(new_ncols, f)` - Resize with function
- `truncate(new_ncols)` - Truncate to new size

#### Conversions (lines 100-126)
- `into_col_shape()` - Convert column shape
- `into_diagonal()` - Convert to diagonal matrix
- `into_transpose()` - Convert to column vector

#### Methods Forwarding to RowRef/RowMut (lines 145-517)
Provides the same API as `RowRef` and `RowMut` through reborrowing:
- All immutable methods forward to `self.rb()`
- All mutable methods forward to `self.rb_mut()`

#### Min/Max (lines 539-552)
Direct implementations for owned rows.

#### FromIterator (lines 553-564)
Implements collection from iterators.

#### Tests (lines 565-589)
Unit tests for min, max, and from_iter.

### rowmut.rs - Mutable Row Reference

#### Struct Definition (lines 6-8)
```rust
pub struct Mut<'a, T, Cols = usize, CStride = isize> {
    pub(crate) trans: ColMut<'a, T, Cols, CStride>,
}
```
Internally stores a mutable transposed column reference.

#### Constructors (lines 43-77)
- `from_mut(value: &'a mut T)` - Create from mutable element
- `from_slice_mut(slice: &'a mut [T])` - Create from mutable slice
- `from_raw_parts_mut(ptr, ncols, col_stride)` - Unsafe constructor

#### Immutable Methods (lines 78-320)
All immutable access methods delegate to `self.into_const()`:
- Shape queries, element access, slicing, transformations, etc.

#### Mutable-Specific Methods

**Pointer access** (lines 361-380):
- `as_ptr_mut()`, `ptr_at_mut()`, `ptr_inbounds_at_mut()`

**Mutable access** (lines 426-437):
- `at_mut()`, `at_mut_unchecked()` - Internal element access

**Mutable slicing** (lines 384-661):
- `split_at_col_mut()`, `subcols_mut()`, `get_mut()`, `get_mut_unchecked()`
- `as_col_shape_mut()`, `as_dyn_cols_mut()`, `as_dyn_stride_mut()`

**Mutable transformations** (lines 393-423):
- `transpose_mut()`, `conjugate_mut()`, `canonical_mut()`, `adjoint_mut()`

**Iteration** (lines 509-572):
- `iter_mut()` - Mutable iterator
- `par_iter_mut()`, `par_partition()`, `par_partition_mut()` (rayon)

**Operations** (lines 341-359):
- `copy_from()` - Copy from another row
- `fill()` - Fill with value
- `map_mut()`, `for_each_mut()` (lines 119-129)

**Slice conversion** (lines 607-634):
- `as_slice_mut()`, `as_array_mut()` - For contiguous layouts

#### Min/Max (lines 670-683)
Immutable min/max for mutable references.

#### Tests (lines 684-705)
Unit tests for min/max on mutable references.

### row_index.rs - Indexing Implementations

#### Range Indexing (lines 4-65)
Implements `RowIndex<ColRange>` for both `RowRef` and `RowMut`:
- Converts range to indices and validates bounds
- Returns subrow with appropriate length type
- `get()` - With bounds checking
- `get_unchecked()` - Without bounds checking (debug asserts only)

#### Single Element Indexing (lines 66-81)
Macro-generated implementations for `Idx<usize>` and `Idx<Dim<'N>>`:
- For `RowRef`: Returns `&'a T`
- For `RowMut`: Returns `&'a mut T`
- Both checked and unchecked variants

## Key Design Patterns

### Transpose-Based Implementation
All row types internally store a transposed column vector:
- `Ref` wraps `ColRef`
- `Own` wraps `Col`
- `Mut` wraps `ColMut`

This allows code reuse with the column module.

### Reborrowing Pattern
- `Reborrow` trait for immutable reborrowing (copyable)
- `ReborrowMut` trait for mutable reborrowing
- `IntoConst` trait for converting mutable to immutable

### Safety and Uninitialized Data
All row types explicitly note they can handle partially/fully uninitialized data:
- Care must be taken not to read uninitialized values
- No references to uninitialized data should be formed

### Memory Layout Guarantees
- `Row<T>` guarantees row-major layout (column stride = 1)
- `RowRef`/`RowMut` can have arbitrary strides

### Parallel Support
All types support Rayon parallel iteration when the `rayon` feature is enabled:
- `par_iter()`, `par_iter_mut()`
- `par_partition()`, `par_partition_mut()`

## Type Parameters

- `T` - Element type
- `Cols` - Column dimension type (usually `usize` or `Dim<'N>`)
- `CStride` - Column stride type (usually `isize` or `ContiguousFwd`)
- `'a` - Lifetime for borrowed references

## Integration with Matrix Types

All row types implement:
- `AsMatRef` - Convert to matrix reference (1 × Cols)
- `AsMatMut` - Convert to mutable matrix reference (for mutable types)
- Rows always have `Rows = One` type parameter

## Testing

Each implementation file includes unit tests:
- `rowref.rs`: Tests for `min()` and `max()` on RowRef
- `rowown.rs`: Tests for `min()`, `max()`, and `from_iter` on Row
- `rowmut.rs`: Tests for `min()` and `max()` on RowMut
