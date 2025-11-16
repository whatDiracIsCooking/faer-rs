# faer/src/diag Module

## Overview
The `faer/src/diag` module provides diagonal matrix types and operations in the faer-rs linear algebra library. It implements three main diagonal matrix types with different ownership semantics.

## File Structure
- `mod.rs` - Module definition, type aliases, and generic wrapper
- `diagref.rs` - Immutable diagonal matrix view (reference)
- `diagmut.rs` - Mutable diagonal matrix view
- `diagown.rs` - Owned diagonal matrix

## Core Types

### Type Aliases (mod.rs:9-15)
```rust
pub type DiagRef<'a, T, Dim = usize, Stride = isize> = generic::Diag<Ref<'a, T, Dim, Stride>>;
pub type DiagMut<'a, T, Dim = usize, Stride = isize> = generic::Diag<Mut<'a, T, Dim, Stride>>;
pub type Diag<T, Dim = usize> = generic::Diag<Own<T, Dim>>;
```

- **DiagRef** - Immutable view over diagonal matrix data
- **DiagMut** - Mutable view over diagonal matrix data
- **Diag** - Owned diagonal matrix

### Generic Wrapper (mod.rs:19-124)

The `generic::Diag<Inner>` wrapper provides:
- Transparent wrapper around inner type (`#[repr(transparent)]`)
- `Deref` and `DerefMut` implementations to access inner type
- `Reborrow`, `ReborrowMut`, and `IntoConst` trait implementations
- Indexing support via `Index` and `IndexMut` traits
- Helper methods `from_inner_ref` and `from_inner_mut` for safe wrapping

## Ref - Immutable View (diagref.rs)

### Structure (diagref.rs:3-5)
```rust
pub struct Ref<'a, T, Dim = usize, Stride = isize> {
    pub(crate) inner: ColRef<'a, T, Dim, Stride>,
}
```

Internally stores a column vector reference, representing the diagonal elements.

### Key Methods

#### Construction
- `from_ref(value: &'a T)` - Create diagonal view from single element (diagref.rs:51)
- `from_slice(slice: &'a [T])` - Create from slice (diagref.rs:58)
- `from_raw_parts(ptr, dim, stride)` - Unsafe construction from raw parts (diagref.rs:72)

#### Views and Transformations
- `column_vector(self)` - Returns diagonal as column vector view (diagref.rs:105)
- `as_ref(&self)` - Returns view over self (diagref.rs:111)
- `as_shape<D>(self, len: D)` - Reshape with dimension checking (diagref.rs:119)
- `as_dyn(self)` - Convert to dynamic shape (diagref.rs:129)
- `as_dyn_stride(self)` - Convert to dynamic stride (diagref.rs:139)
- `conjugate(self)` - Returns conjugate view (diagref.rs:149)
- `canonical(self)` - Returns unconjugated view (diagref.rs:162)

#### Iteration
- `map<U>(&self, f: impl FnMut(&T) -> U)` - Map function over elements (diagref.rs:92)
- `for_each(&self, f: impl FnMut(&T))` - Iterate with side effects (diagref.rs:98)

#### Properties
- `stride(&self)` - Returns stride in number of elements (diagref.rs:87)
- `dim(&self)` - Returns dimension (diagref.rs:175)

#### Validation (diagref.rs:179-205)
- `is_all_finite(&self)` - Check if all elements are finite
- `has_nan(&self)` - Check if any element is NaN

### Traits
- `Copy` and `Clone` (diagref.rs:13-19)
- `Debug` (diagref.rs:6-12)
- `Reborrow` and `ReborrowMut` (diagref.rs:20-39)
- `IntoConst` (diagref.rs:40-47)

## Mut - Mutable View (diagmut.rs)

### Structure (diagmut.rs:4-6)
```rust
pub struct Mut<'a, T, Dim = usize, Stride = isize> {
    pub(crate) inner: ColMut<'a, T, Dim, Stride>,
}
```

Similar to `Ref` but holds mutable column vector view.

### Key Methods

#### Construction
- `from_mut(value: &'a mut T)` - Create from mutable element reference (diagmut.rs:17)
- `from_slice_mut(slice: &'a mut [T])` - Create from mutable slice (diagmut.rs:24)
- `from_raw_parts_mut(ptr, dim, stride)` - Unsafe construction (diagmut.rs:38)

#### Views and Transformations
- `column_vector(self)` - Returns immutable column vector view (diagmut.rs:81)
- `column_vector_mut(self)` - Returns mutable column vector view (diagmut.rs:87)
- `as_ref(&self)` - Returns immutable view (diagmut.rs:93)
- `as_mut(&mut self)` - Returns mutable view (diagmut.rs:99)
- Shape/stride/conjugate methods similar to `Ref` but with mutable variants (diagmut.rs:172-224)

#### Mutation
- `fill(&mut self, value: T)` - Fill all elements with value (diagmut.rs:105)
- `copy_from(&mut self, rhs)` - Copy from another diagonal (diagmut.rs:235)
- `map_mut<U>(&mut self, f)` - Map with mutable closure (diagmut.rs:68)
- `for_each_mut(&mut self, f)` - Iterate with mutable closure (diagmut.rs:74)

### Traits
- `Debug` (diagmut.rs:7-13)
- `Reborrow`, `ReborrowMut`, `IntoConst` (diagmut.rs:244-277)

## Own - Owned Diagonal (diagown.rs)

### Structure (diagown.rs:4-7)
```rust
#[derive(Clone)]
pub struct Own<T, Dim: Shape = usize> {
    pub(crate) inner: Col<T, Dim>,
}
```

Owns the diagonal data as a column vector.

### Key Methods

#### Construction (diagown.rs:172-208)
- `zeros(dim: Dim)` - Create diagonal filled with zeros
- `ones(dim: Dim)` - Create diagonal filled with ones
- `full(dim: Dim, value: T)` - Create diagonal filled with value

#### Views
- `as_ref(&self)` - Returns `DiagRef` view (diagown.rs:61)
- `as_mut(&mut self)` - Returns `DiagMut` view (diagown.rs:71)
- `column_vector(&self)` - Returns column vector reference (diagown.rs:43)
- `column_vector_mut(&mut self)` - Returns mutable column vector (diagown.rs:49)
- `into_column_vector(self)` - Consumes and returns owned column (diagown.rs:55)

#### Operations
- `map(&self, f)` - Map over elements (diagown.rs:22)
- `map_mut(&mut self, f)` - Map with mutation (diagown.rs:32)
- `for_each(&self, f)` - Iterate (diagown.rs:27)
- `for_each_mut(&mut self, f)` - Iterate with mutation (diagown.rs:37)
- `copy_from(&mut self, rhs)` - Copy from another diagonal (diagown.rs:213)

#### Properties
- `stride(&self)` - Always returns 1 for owned diagonal (diagown.rs:17)
- `dim(&self)` - Returns dimension (diagown.rs:167)

### Traits
- `Clone` (diagown.rs:4)
- `Debug` (diagown.rs:8-12)
- `Reborrow` and `ReborrowMut` (diagown.rs:222-241)

## Conversion Traits (mod.rs:125-194)

### AsDiagRef
Trait for types that can be converted to diagonal matrix view.
```rust
pub trait AsDiagRef {
    type T;
    type Dim: Shape;
    fn as_diag_ref(&self) -> DiagRef<'_, Self::T, Self::Dim>;
}
```

Implemented for:
- `DiagRef` (mod.rs:139)
- `DiagMut` (mod.rs:150)
- `&M where M: AsDiagRef` (mod.rs:171)
- `&mut M where M: AsDiagRef` (mod.rs:180)

### AsDiagMut
Trait for types that can be converted to mutable diagonal matrix view.
```rust
pub trait AsDiagMut: AsDiagRef {
    fn as_diag_mut(&mut self) -> DiagMut<'_, Self::T, Self::Dim>;
}
```

Implemented for:
- `DiagMut` (mod.rs:162)
- `&mut M where M: AsDiagMut` (mod.rs:189)

## Design Patterns

### Reborrowing
All three types support reborrowing through the `Reborrow` and `ReborrowMut` traits, allowing:
- Temporary borrows without consuming the original value
- Conversion between different lifetime parameters
- Integration with faer's zero-cost abstraction system

### Column Vector Backing
Diagonal matrices are internally represented as column vectors:
- Enables reuse of existing column vector operations
- Natural representation: diagonal elements stored sequentially
- Stride parameter controls spacing between elements

### Generic Wrapper Pattern
The `generic::Diag<Inner>` wrapper provides:
- Type-level abstraction over ownership (Ref/Mut/Own)
- Common functionality through `Deref`/`DerefMut`
- Zero-cost abstraction via `#[repr(transparent)]`

### View Conversions
Extensive support for converting between different view types:
- Shape conversions (static ↔ dynamic dimensions)
- Stride conversions (static ↔ dynamic strides)
- Mutability conversions (mut → ref via `into_const`)
- Conjugate views for complex numbers

## Usage Patterns

### Creating Diagonals
```rust
// From slice
let data = [1.0, 2.0, 3.0];
let diag = DiagRef::from_slice(&data);

// Owned diagonal
let diag = Diag::zeros(5);
let diag = Diag::ones(3);
let diag = Diag::full(4, 42.0);
```

### Accessing Elements
```rust
// Via indexing (requires reborrow-capable inner type)
let val = diag[Idx(2)];

// Via column vector view
let col = diag.column_vector();
let val = col.at(Idx(2));
```

### Transformations
```rust
// Map elements
let doubled = diag.map(|x| x * 2.0);

// Iterate with side effects
diag.for_each(|x| println!("{}", x));

// Conjugate for complex numbers
let conj_view = diag.conjugate();
```

## Integration with faer

The diagonal module integrates with faer's broader architecture:
- Uses `Shape` trait for compile-time and runtime dimensions
- Uses `Stride` trait for memory layout flexibility
- Uses `Conjugate` trait for complex number support
- Uses `ComplexField` for numeric operations
- Leverages column vector types (`Col`, `ColRef`, `ColMut`)

## Safety

Unsafe code is limited to:
- Raw pointer construction (`from_raw_parts`, `from_raw_parts_mut`)
- Pointer casting in wrapper conversions (`from_inner_ref`, `from_inner_mut`)

All unsafe operations delegate safety requirements to underlying column vector types.
