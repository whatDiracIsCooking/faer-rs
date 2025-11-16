# faer/src/col Module Documentation

## Overview

The `col` module in faer-rs provides column vector types and operations. It implements three main column types with different ownership semantics:
- **ColRef**: Immutable view over a column vector (similar to `&[T]` but strided)
- **ColMut**: Mutable view over a column vector (similar to `&mut [T]` but strided)
- **Col**: Heap-allocated, resizable column vector (owned)

## File Structure

### mod.rs (257 lines)
Main module file that defines:
- Core type aliases: `ColRef<'a, T, Rows, RStride>`, `ColMut<'a, T, Rows, RStride>`, `Col<T, Rows>`
- Generic wrapper `generic::Col<Inner>` for all column types
- `ColIndex` trait for slicing columns with indices or ranges
- `AsColRef` and `AsColMut` traits for conversion to column views
- `AsMatRef`/`AsMatMut`/`AsMat` implementations for columns (treating them as single-column matrices)
- Internal `ColView` struct storing `ptr: NonNull<T>`, `nrows: Rows`, `row_stride: RStride`

**Key Design**: Columns are stored internally as matrices with 1 column.

### colref.rs (774 lines)
Implements immutable column references (`ColRef`):

**Core struct**: `Ref<'a, T, Rows, RStride>` containing `ColView` and lifetime marker

**Key methods**:
- Construction: `from_ref()`, `from_slice()`, `from_raw_parts()`
- Access: `as_ptr()`, `nrows()`, `ncols()`, `row_stride()`, `ptr_at()`, `ptr_inbounds_at()`
- Slicing: `split_at_row()`, `subrows()`, `get()`, `get_unchecked()`
- Transformations: `transpose()`, `conjugate()`, `canonical()`, `adjoint()`, `reverse_rows()`
- Shape conversion: `as_row_shape()`, `as_dyn_rows()`, `as_dyn_stride()`
- Iteration: `iter()`, `par_iter()`, `par_partition()`
- Operations: `map()`, `for_each()`, `cloned()`, `to_owned()`
- Norms: `norm_max()`, `norm_l2()`, `squared_norm_l2()`, `norm_l1()`, `sum()`
- Matrix view: `as_mat()`, `as_diagonal()`
- Utilities: `is_all_finite()`, `has_nan()`, `max()`, `min()` (for RealField types)

**Special features**:
- `ColRef<'a, T, Rows, ContiguousFwd>` provides `as_slice()` for contiguous columns
- `ColRef<'a, T, Dim<'ROWS>, ContiguousFwd>` provides `as_array()` for lifetime-bound arrays
- Trait implementations: `Copy`, `Clone`, `Reborrow`, `ReborrowMut`, `IntoConst`, `Sync`, `Send`

### colmut.rs (772 lines)
Implements mutable column references (`ColMut`):

**Core struct**: `Mut<'a, T, Rows, RStride>` containing `ColView` and lifetime marker

**Key methods** (mirrors ColRef but with mutable variants):
- Construction: `from_mut()`, `from_slice_mut()`, `from_raw_parts_mut()`
- Access: `as_ptr_mut()`, `ptr_at_mut()`, `ptr_inbounds_at_mut()`
- Slicing: `split_at_row_mut()`, `subrows_mut()`, `get_mut()`, `get_mut_unchecked()`
- Transformations: `transpose_mut()`, `conjugate_mut()`, `canonical_mut()`, `adjoint_mut()`, `reverse_rows_mut()`
- Shape conversion: `as_row_shape_mut()`, `as_dyn_rows_mut()`, `as_dyn_stride_mut()`
- Iteration: `iter_mut()`, `par_iter_mut()`, `par_partition_mut()`
- Operations: `map()`, `for_each()`, `map_mut()`, `for_each_mut()`, `copy_from()`, `fill()`
- Matrix view: `as_mat_mut()`, `as_diagonal_mut()`

**Special features**:
- `ColMut<'a, T, Rows, ContiguousFwd>` provides `as_slice_mut()` for contiguous columns
- `ColMut<'a, T, Dim<'ROWS>, ContiguousFwd>` provides `as_array_mut()` for lifetime-bound arrays
- Trait implementations: `Reborrow`, `ReborrowMut`, `IntoConst`, `Sync`, `Send`
- Uses `SyncCell<T>` for safe parallel iteration with rayon

### colown.rs (687 lines)
Implements owned column vectors (`Col`):

**Core struct**: `Own<T, Rows>` wrapping `Mat<T, Rows, usize>`

**Key methods**:
- Construction: `from_fn()`, `zeros()`, `ones()`, `full()`, `from_iter()`
- Capacity: `try_reserve()`, `reserve()`
- Resizing: `resize_with()`, `truncate()`
- Shape conversion: `into_row_shape()`, `into_diagonal()`, `into_transpose()`
- Delegates most methods to underlying `ColRef`/`ColMut` views via reborrowing

**Special features**:
- Memory layout guaranteed to be column-major (row stride = 1)
- Implements `FromIterator<T>` with optimized iterator collection
- Custom `from_iter_imp()` handles both ExactSizeIterator and general iterators
- All mutation operations available through `as_mut()` reborrowing
- Trait implementations: `Clone`, `Reborrow`, `ReborrowMut`

### col_index.rs (81 lines)
Implements the `ColIndex` trait for indexing and slicing columns:

**Implementations**:
1. **Range slicing**: `ColIndex<RowRange>` for `ColRef` and `ColMut` where `RowRange: IntoRange`
   - Returns subcolumn views
   - Validates bounds in `get()`, skips checks in `get_unchecked()`

2. **Single element access**: `ColIndex<Idx<R>>` for `ColRef` and `ColMut`
   - Returns `&T` or `&mut T`
   - Uses internal `at()` and `at_mut()` methods

**Macro**: `idx_impl!` generates implementations for both `usize` and `Dim<'N>` indices

## Core Type Hierarchy

```
ColView<T, Rows, RStride>        // Internal representation (ptr, nrows, row_stride)
    ├── Ref<'a, T, Rows, RStride>      // Immutable reference wrapper
    ├── Mut<'a, T, Rows, RStride>      // Mutable reference wrapper
    └── Own<T, Rows>                    // Owned wrapper (around Mat)

generic::Col<Inner>               // Generic wrapper (transparent)
    ├── ColRef<'a, T, Rows, RStride> = generic::Col<Ref<'a, T, Rows, RStride>>
    ├── ColMut<'a, T, Rows, RStride> = generic::Col<Mut<'a, T, Rows, RStride>>
    └── Col<T, Rows> = generic::Col<Own<T, Rows>>
```

## Key Traits

### ColIndex<RowRange>
Enables slicing columns with ranges or indices:
- `get(this: Self, row: RowRange) -> Self::Target` - with bounds checking
- `get_unchecked(this: Self, row: RowRange) -> Self::Target` - without bounds checking

### AsColRef / AsColMut
Conversion traits for obtaining column views:
- `AsColRef::as_col_ref(&self) -> ColRef<'_, Self::T, Self::Rows>`
- `AsColMut::as_col_mut(&mut self) -> ColMut<'_, Self::T, Self::Rows>`
- Automatically implemented for types implementing `AsMatRef<Cols = One>`

### Reborrow / ReborrowMut
Enable ergonomic reborrowing patterns:
- `rb()` - immutable reborrow (shortens lifetime)
- `rb_mut()` - mutable reborrow (shortens lifetime)
- Essential for working with views in faer's API

## Type Parameters

- **T**: Element type (typically numeric types)
- **Rows**: Shape type parameter (usually `usize` or `Dim<'N>` for compile-time sizes)
- **RStride**: Row stride type (usually `isize` or `ContiguousFwd` for unit stride)

## Memory Layout

Columns are guaranteed to be stored in column-major order:
- For `Col<T, Rows>`: row stride is always 1
- For `ColRef`/`ColMut`: row stride can be arbitrary (specified by `RStride`)
- Allows zero-copy views into matrices and other data structures

## Uninitialized Memory Support

Important note from documentation:
> Unlike a slice, the data pointed to by `ColRef<'_, T>` / `ColMut<'_, T>` is allowed to be partially or fully uninitialized under certain conditions. Care must be taken to not perform any operations that read the uninitialized values.

## Integration with Matrix Types

Columns integrate seamlessly with matrices:
- `as_mat()` / `as_mat_mut()` - view column as single-column matrix
- All columns implement `AsMatRef` with `Cols = One`
- Can be used anywhere a matrix is expected

## Parallel Operations (with rayon feature)

- `par_iter()` / `par_iter_mut()` - parallel iteration
- `par_partition()` / `par_partition_mut()` - split into chunks for parallel processing
- Thread-safety ensured through proper `Sync`/`Send` bounds

## Testing

Each file includes unit tests:
- `test_col_min()` / `test_col_max()` - verify min/max operations
- `test_from_iter()` - verify iterator collection (in colown.rs)
- Tests cover both empty and non-empty columns

## Notable Implementation Details

1. **Type erasure pattern**: The generic `Col<Inner>` wrapper provides a uniform interface regardless of ownership semantics.

2. **Zero-cost abstractions**: Most methods are `#[inline]` and forward to underlying implementations.

3. **Safety**: Unsafe code is carefully isolated in pointer manipulation, with comprehensive bounds checking in safe APIs.

4. **const_cast pattern**: Internal `const_cast()` method allows converting `ColRef` to `ColMut` for implementing mutable operations.

5. **Dimension binding**: `bind_r()` method enables lifetime-based dimension tracking with `Dim<'N>`.

6. **Rayon integration**: Uses `SyncCell<T>` wrapper for safe parallel mutable iteration.

7. **Iterator optimization**: `from_iter_imp()` handles ExactSizeIterator separately for better performance.

## Related Modules

- `faer/src/mat` - Matrix types (columns are internally single-column matrices)
- `faer/src/row` - Row vector types (transpose of columns)
- `faer/src/diag` - Diagonal matrix types (columns can be viewed as diagonals)
