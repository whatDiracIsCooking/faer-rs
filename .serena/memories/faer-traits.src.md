# faer-traits/src Module Memory

## Overview
The `faer-traits/src` module contains a single file: `lib.rs` (5823 lines), which defines the core trait abstractions for the faer linear algebra library.

## File: lib.rs

### Key Components

#### 1. Core Traits
- **`ComplexField`**: Main trait for complex number types with SIMD support
  - Defines operations: zero, one, add, sub, mul, div, neg, conj, abs, sqrt, recip
  - Associated types: `Real`, `Arch`, `Index`, `SimdCtx`, `SimdVec`, `SimdMask`
  - SIMD operations for vectorized computation
  - Type-level constants: `IS_REAL`, `IS_NATIVE_F32`, `IS_NATIVE_C32`, etc.

- **`RealField`**: Trait for real number types, extends `ComplexField`
  - Requires `PartialOrd` and `num_traits::NumAssign`
  - Defines numeric limits: epsilon, min_positive, max_positive
  - Number of mantissa bits

- **`Conjugate`**: Trait for conjugate types
  - Associated types: `Conj` (conjugate type), `Canonical` (canonical form)
  - `IS_CANONICAL` const for checking if type is canonical

#### 2. Index Traits
- **`Index`**: Trait for unsigned index types (u32, u64, usize)
  - Conversion methods: `truncate`, `zx` (zero extend)
  - `FixedWidth` and `Signed` associated types
  - Sum with overflow checking

- **`SignedIndex`**: Trait for signed index types (i32, i64, isize)
  - Conversion methods: `truncate`, `zx`, `sx` (sign extend)
  - Maximum value constant
  - Sum with overflow checking

- **`IndexCore`**: Base trait for index operations

#### 3. Operation Traits
- **`AddByRef`, `SubByRef`, `MulByRef`, `DivByRef`, `NegByRef`**: Reference-based arithmetic
- **`RefOps`**: Trait combining all reference operations
- **`ByRef<T>`**: Trait for reference conversion

#### 4. SIMD Support
- **`SimdCtx<T, S>`**: SIMD context wrapper
  - Methods: splat, add, sub, mul, neg, conj, abs1, abs2, etc.
  - Mask operations: eq, lt, gt, le, ge, select
  - Load/store operations with masking
  - Index operations

- **`SimdCapabilities`** enum:
  - `None`: No SIMD support
  - `Copy`: Type is Copy but no SIMD
  - `Simd`: Full SIMD support

- **`SimdArch`**: Trait for SIMD architectures (pulp::Arch, pulp::Scalar)

- **`RealReg<T>`**: Wrapper for real SIMD registers

#### 5. Type Implementations

**Float Types (f32, f64)**:
- Full `ComplexField` and `RealField` implementations
- Native SIMD support via pulp
- Index types: u32 for f32, u64 for f64
- SIMD operations using pulp SIMD intrinsics

**Complex Types (Complex<f32>, Complex<f64>)**:
- Generic implementation for `Complex<T: RealField>`
- Component-wise SIMD operations
- Special handling for conjugate operations

**ComplexImpl Types**:
- `ComplexImpl<f32>`, `ComplexImpl<f64>`: Optimized complex implementations
- Native SIMD support with interleaved storage
- Uses pulp's c32s/c64s SIMD types
- Special handling for abs, abs1, abs2 operations
- `SIMD_ABS_SPLIT_REAL_IMAG` optimization flag

**ComplexConj<T>**:
- Conjugate view type
- `IS_CANONICAL = false`

**Symbolic**:
- Zero-sized symbolic type for compile-time analysis
- All operations return `Symbolic`
- `materialize()` creates zero-cost slices

**Quad-precision (fx128, cx128)**:
- `fx128 = qd::Quad`: Double-double precision (~128-bit)
- Full `ComplexField` and `RealField` implementations
- SIMD support via qd::simd module
- epsilon multiplied by 8, ~100 mantissa bits

#### 6. Helper Functions (math_utils module)
- `eps<T>()`, `nbits<T>()`: Numeric properties
- `min_positive<T>()`, `max_positive<T>()`: Limits
- `zero<T>()`, `one<T>()`, `nan<T>()`, `infinity<T>()`: Constants
- `real<T>()`, `imag<T>()`, `conj<T>()`, `copy<T>()`: Complex operations
- `add<T>()`, `sub<T>()`, `mul<T>()`, `div<T>()`, `neg<T>()`: Arithmetic
- `abs<T>()`, `abs1<T>()`, `abs2<T>()`, `absmax<T>()`: Magnitude
- `sqrt<T>()`, `recip<T>()`: Mathematical functions
- `min<T>()`, `max<T>()`, `hypot<T>()`: Comparison/utility
- `is_nan<T>()`, `is_finite<T>()`: Checks
- `mul_real<T>()`, `mul_pow2<T>()`: Specialized multiplication

#### 7. Internal Helper Functions
- `abs_impl<T: RealField>()`: Numerically stable absolute value for complex numbers
  - Handles overflow/underflow with scaling
- `recip_impl<T: RealField>()`: Numerically stable reciprocal
  - Special handling for NaN, infinity, zero
- `sqrt_impl<T: RealField>()`: Complex square root
  - Handles branch cuts properly

#### 8. Extension Traits (ext module)
- **`ComplexFieldExt`**: Convenience methods for `ComplexField`
  - Shorter method names: `zero()`, `one()`, `abs()`, etc.
- **`RealFieldExt`**: Convenience methods for `RealField`
  - Utility methods: `fmax()`, `fmin()`, `hypot()`

#### 9. Type Aliases
- `c64 = Complex<f64>`
- `c32 = Complex<f32>`
- `fx128 = qd::Quad`
- `cx128 = Complex<fx128>`
- `Real<T> = <<T as Conjugate>::Canonical as ComplexField>::Real`

#### 10. Macros
- `impl_op!`: Implements binary operators for all reference combinations
- `impl_assign_op!`: Implements assignment operators

### Dependencies
- `bytemuck`: Zero-copy type conversions (Pod trait)
- `num_complex`: Complex number type
- `num_traits`: Numeric traits (Zero, One, Num, NumAssign)
- `pulp`: SIMD abstraction layer (Simd, Arch, Scalar)
- `qd`: Quad-precision arithmetic

### Design Patterns
1. **Type-level programming**: Extensive use of associated types and const generics
2. **SIMD abstraction**: Generic over SIMD instruction sets via pulp
3. **Reference operations**: Minimize copies with reference-based arithmetic
4. **Numerical stability**: Careful handling of overflow/underflow in complex operations
5. **Zero-cost abstractions**: Symbolic type for compile-time analysis

### Key Features
- Unified interface for real, complex, and quad-precision types
- Portable SIMD support (f32, f64, Complex<f32>, Complex<f64>, Quad)
- Numerically stable implementations
- Support for conjugate views without data duplication
- Index abstraction for 32-bit and 64-bit indexing
- Extension traits for ergonomic API

### Performance Considerations
- SIMD vectorization for all numeric types
- Interleaved storage for complex SIMD (ComplexImpl)
- Specialized implementations for native float/complex types
- Mask operations for conditional SIMD operations
- FMA (fused multiply-add) support where available
