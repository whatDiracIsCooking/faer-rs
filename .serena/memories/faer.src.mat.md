# faer/src/mat Module Memory

## Overview
The `faer/src/mat` module implements the core matrix types for the faer linear algebra library. It provides three main matrix representations: owned matrices (`Mat`), immutable views (`MatRef`), and mutable views (`MatMut`).

## Module Structure

### Files
- **mod.rs** - Module definition, type aliases, and trait definitions
- **matref.rs** - Immutable matrix view implementation (2009 lines)
- **matown.rs** - Owned matrix implementation with dynamic allocation (1700 lines)
- **matmut.rs** - Mutable matrix view implementation (1982 lines)
- **mat_index.rs** - Matrix indexing trait implementations (219 lines)

## Core Types

### 1. Mat<T, Rows, Cols> (Own<T, Rows, Cols>)
**Location**: `matown.rs`

Heap-allocated resizable matrix, similar to a 2D `Vec`.

**Memory Layout**:
- Column-major storage (row stride = 1)
- Column stride = capacity (may include padding for alignment)
- NOT necessarily contiguous - padding may exist between columns for performance
- Padding alignment: 64 bytes for power-of-2 sized types when possible

**Key Fields**:
```rust
struct Own<T, Rows: Shape, Cols: Shape> {
    raw: RawMat<T>,     // Pointer, capacities, layout
    nrows: Rows,         // Current row count
    ncols: Cols,         // Current column count
}

struct RawMat<T> {
    ptr: NonNull<T>,
    row_capacity: usize,
    col_capacity: usize,
    layout: StackReq,    // Allocation layout
}
```

**Key Methods**:
- `new()` - Empty 0×0 matrix
- `from_fn(nrows, ncols, f)` - Create from function
- `zeros(nrows, ncols)` - Zero-filled matrix
- `ones(nrows, ncols)` - One-filled matrix
- `identity(nrows, ncols)` - Identity matrix
- `reserve(row_capacity, col_capacity)` - Reserve capacity
- `resize_with(new_nrows, new_ncols, f)` - Resize with element function
- `truncate(new_nrows, new_ncols)` - Shrink dimensions
- `push_row(row)` / `push_col(col)` - Add rows/columns

**Alignment Strategy** (`align_for` function):
- If `needs_drop<T>()` or size not power-of-2: use natural alignment
- Otherwise: max(natural_align, 64) for better SIMD performance

### 2. MatRef<'a, T, Rows, Cols, RStride, CStride> (Ref)
**Location**: `matref.rs`

Immutable view over matrix data with arbitrary strides.

**Key Features**:
- Can be `Copy` (unlike `MatMut`)
- Supports both positive and negative strides
- Can view potentially uninitialized data (unsafe operations allowed)
- Zero-cost reborrowing via `Reborrow` trait

**Construction Methods**:
- `from_raw_parts(ptr, nrows, ncols, row_stride, col_stride)`
- `from_column_major_slice(slice, nrows, ncols)`
- `from_row_major_slice(slice, nrows, ncols)`
- `from_column_major_array<const ROWS, COLS>(array)`
- `from_repeated_ref(value, nrows, ncols)` - Broadcast single value

**View Transformations**:
- `transpose()` - Swap rows/cols and strides
- `conjugate()` - View as conjugated (for complex numbers)
- `adjoint()` - Conjugate transpose
- `reverse_rows()` / `reverse_cols()` - Reversed views
- `submatrix(row_start, col_start, nrows, ncols)`
- `split_at(row, col)` - Split into 4 quadrants
- `split_at_row(row)` / `split_at_col(col)`

**Iteration**:
- `row_iter()` / `col_iter()` - Sequential iteration
- `par_row_iter()` / `par_col_iter()` - Parallel iteration (rayon)
- `par_col_chunks(chunk_size)` / `par_col_partition(count)`

**Numeric Operations**:
- `norm_max()` / `norm_l1()` / `norm_l2()` / `squared_norm_l2()`
- `sum()` - Sum of all elements
- `determinant()` - Matrix determinant
- `kron(other)` - Kronecker product
- `min()` / `max()` - Min/max element (RealField only)

**Type Conversions**:
- `as_dyn()` - Convert to dynamic shape
- `as_shape(nrows, ncols)` - Assert and convert shape
- `try_as_col_major()` / `try_as_row_major()` - Try contiguous views

### 3. MatMut<'a, T, Rows, Cols, RStride, CStride> (Mut)
**Location**: `matmut.rs`

Mutable view over matrix data.

**Key Differences from MatRef**:
- NOT `Copy` (requires reborrowing)
- Enforces no aliasing via lifetime system
- Provides mutable element access

**Construction**:
- `from_raw_parts_mut(ptr, nrows, ncols, row_stride, col_stride)`
- `from_column_major_slice_mut(slice, nrows, ncols)`
- `from_row_major_slice_mut(slice, nrows, ncols)`

**Mutable Operations**:
- `fill(value)` - Fill all elements
- `copy_from(other)` - Copy from another matrix
- `copy_from_triangular_lower(other)` - Copy lower triangle
- `copy_from_triangular_upper(other)` - Copy upper triangle
- `two_rows_mut(i0, i1)` / `two_cols_mut(j0, j1)` - Simultaneous access

**Reborrowing** (critical for usability):
```rust
use reborrow::*;
let mut mat = Mat::zeros(5, 5);
let mut view = mat.as_mut();

// Without reborrow - view is moved
// some_fn(view);
// some_fn(view); // ERROR: value used after move

// With reborrow - temporary borrow
some_fn(view.rb_mut());
some_fn(view.rb_mut()); // OK!
```

## Indexing System

### MatIndex Trait
**Location**: `mat_index.rs`

Enables flexible indexing with ranges and indices.

```rust
// Single element
let x: &T = mat.get(i, j);

// Row slice
let row: RowRef = mat.get(i, ..);

// Column slice
let col: ColRef = mat.get(.., j);

// Submatrix
let sub: MatRef = mat.get(2..5, 3..7);
```

**Implementations**:
- `MatIndex<Idx<R>, Idx<C>>` → `&T` or `&mut T`
- `MatIndex<Idx<R>, Range>` → `RowRef` or `RowMut`
- `MatIndex<Range, Idx<C>>` → `ColRef` or `ColMut`
- `MatIndex<Range, Range>` → `MatRef` or `MatMut`

## Generic Wrapper System

### generic::Mat<Inner>
**Location**: `mod.rs`

Transparent wrapper providing unified interface:

```rust
#[repr(transparent)]
pub struct Mat<Inner>(pub Inner);
```

Implements:
- `Deref` / `DerefMut` to `Inner`
- `Reborrow` / `ReborrowMut` for view types
- `Index` / `IndexMut` using `Idx<Rows>`, `Idx<Cols>`

This allows:
```rust
let mat: Mat<Own<f64>>;
let view: Mat<Ref<f64>>;
let mut_view: Mat<Mut<f64>>;
// All share common interface via the wrapper
```

## Shape and Stride System

### Shape Types
- `usize` - Dynamic shape (runtime value)
- `Dim<'a>` - Compile-time tracked dimension

### Stride Types
- `isize` - Dynamic stride
- `ContiguousFwd` - Compile-time stride of +1
- `ContiguousRev` - Compile-time stride of -1

**Stride Reversal**:
```rust
trait Stride {
    type Rev: Stride;
    fn rev(self) -> Self::Rev;
}
```

Allows efficient view reversal without data copying.

## Traits

### AsMatRef / AsMatMut / AsMat<T>
**Location**: `mod.rs`

Unified interface for matrix-like types:

```rust
pub trait AsMatRef {
    type T;
    type Rows: Shape;
    type Cols: Shape;
    type Owned: AsMat<...>;

    fn as_mat_ref(&self) -> MatRef<'_, Self::T, ...>;
}

pub trait AsMatMut: AsMatRef {
    fn as_mat_mut(&mut self) -> MatMut<'_, Self::T, ...>;
}

pub trait AsMat<T>: AsMatMut {
    fn zeros(rows: Self::Rows, cols: Self::Cols) -> Self;
    fn truncate(&mut self, rows: Self::Rows, cols: Self::Cols);
}
```

## Memory Management

### Allocation (matown.rs)
**RawMat::try_with_capacity**:
1. Compute alignment: `align_for(size, align, needs_drop)`
2. Round up row_capacity to alignment multiple
3. Allocate `row_capacity * col_capacity * size_of::<T>()` bytes
4. Store in `StackReq` for proper deallocation

### Reallocation (do_reserve_with):
1. Allocate new `RawMat` with larger capacity
2. Column-by-column copy: `copy_nonoverlapping(old, new, col_bytes)`
3. Drop old allocation

### Drop Implementation
**Own<T>** drop:
- If `needs_drop::<T>()`: iterate columns, drop each column's elements
- Always: deallocate via `RawMatUnit<T>` drop
- **Column-wise iteration** prevents partial drop issues

### Safety Guards
- `DropCol<T>` - Ensures single column is dropped on panic
- `DropMat<T>` - Ensures multiple columns are dropped on panic
- `DropIter<I>` - Ensures iterator cleanup on panic

## Conjugation System

For complex number support:

```rust
// View real matrix as complex (Canonical = itself)
let real_view: MatRef<f64>;

// View complex with possible conjugation
let conj_view: MatRef<c64::Conj>; // May be conjugated
let canonical = conj_view.canonical(); // MatRef<c64>

// Conjugate tracking
pub enum Conj { No, Yes }
Conj::get::<T>() // Returns whether T is conjugated type
```

## Key Patterns

### 1. Reborrow Pattern
All view types implement `Reborrow` and `ReborrowMut`:
```rust
fn process(mat: MatMut<'_, f64>) { /* consumes mat */ }

let mut mat = Mat::zeros(5, 5);
let mut view = mat.as_mut();
process(view.rb_mut()); // Temporary borrow
process(view.rb_mut()); // Can reuse!
```

### 2. Into-Const Pattern
Mutable views convert to immutable:
```rust
impl MatMut<'a, T> {
    pub fn into_const(self) -> MatRef<'a, T>;
}
```

Many methods use: `self.into_const().<method>()` to reuse code.

### 3. Guard Pattern
Dimensionality typing with `generativity`:
```rust
make_guard!(M);
make_guard!(N);
let M = nrows.bind(M);
let N = ncols.bind(N);
// M and N are now Dim<'M> and Dim<'N>
```

### 4. Const-Cast Pattern
Mutable operations on immutable data (internally):
```rust
unsafe {
    let immutable: MatRef<'a, T> = ...;
    let mutable: MatMut<'a, T> = immutable.const_cast();
    // SAFETY: original data was uniquely borrowed
}
```

## Testing

### Test Coverage
- Matrix operations: resize, push_row, push_col
- Min/Max functions
- View transformations
- Edge cases: empty matrices, single element
- Different strides and layouts

## Performance Considerations

1. **Column-major layout**: Optimized for column-wise operations
2. **Padding**: 64-byte alignment for SIMD when beneficial
3. **Zero-copy views**: All view operations are zero-cost
4. **Parallel iteration**: Rayon integration for data parallelism
5. **Stride flexibility**: Negative strides for reversal without copying

## Safety Invariants

### MatRef Safety
- Pointer must be valid for reads for lifetime `'a`
- No mutable aliasing during `'a`
- Data must be initialized before forming references
- All elements within matrix bounds must be accessible

### MatMut Safety
- Pointer must be valid for reads/writes for lifetime `'a`
- **No aliasing whatsoever** (including self-aliasing)
- No two elements at same address (e.g., via zero stride)
- Unique access guarantee for entire `'a`

### Mat Safety
- Owns its allocation
- Capacity ≥ dimensions always
- Column-major invariant: row_stride = 1

## Common Pitfalls

1. **Move semantics**: Forgetting to use `.rb()` or `.rb_mut()` with MatMut
2. **Stride confusion**: Row-major vs column-major construction
3. **Capacity assumptions**: Mat may have internal padding
4. **Aliasing**: Creating overlapping mutable views is UB
5. **Uninitialized data**: MatRef can view uninit memory - must not read

## Integration Points

- **Traits**: `Reborrow`, `ReborrowMut`, `IntoConst`, `Shape`, `Stride`
- **Indexing**: `Idx`, `IdxInc`, ranges via `IntoRange`
- **Numeric**: `ComplexField`, `RealField`, `Conjugate`
- **Parallel**: Rayon's `IndexedParallelIterator`
- **Allocation**: Custom `StackReq` layout tracking

## Versioning Notes
Based on recent commits (Jan 2025):
- Added min/max methods for RealField matrices
- Added suitesparse license files
- Deduplication of SIMD code
- Pulp updates and reformatting
- Triangular solve helpers
