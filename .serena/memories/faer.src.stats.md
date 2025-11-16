# faer/src/stats Module

## Overview

The `faer/src/stats` module provides statistical functions for matrix operations, focusing on computing means and variances for rows and columns of matrices. It includes optimized SIMD implementations and handles NaN values according to user preferences.

## Module Structure

- **mod.rs**: Main module file with random distributions and prelude
- **meanvar.rs**: Mean and variance computation implementations

## Files

### faer/src/stats/mod.rs

**Purpose**: Module entry point providing random value distributions for matrices and complex numbers.

**Key Components**:

1. **ComplexDistribution<Re, Im>** (lines 17-28)
   - Generic random value distribution for complex numbers
   - Creates complex distributions from independent real and imaginary distributions
   - Method: `new(re: Re, im: Im) -> Self`

2. **Random Distribution Structs** (gated by `rand` feature):
   - `CwiseMatDistribution<Rows, Cols, D>` (lines 47-51): Component-wise matrix distribution
   - `CwiseColDistribution<Rows, D>` (lines 53-56): Component-wise column distribution
   - `CwiseRowDistribution<Cols, D>` (lines 58-61): Component-wise row distribution
   - `UnitaryMat<Dim, D>` (lines 63-66): Unitary matrix distribution

3. **Trait Extensions**:
   - `DistributionExt` (lines 37-44): Provides `rand()` method as shorthand for `sample()`

4. **Distribution Implementations**:
   - Matrix distribution sampling (lines 67-77): Creates matrices with component-wise sampling
   - Column distribution sampling (lines 78-85): Creates column vectors
   - Row distribution sampling (lines 86-93): Creates row vectors
   - Unitary matrix sampling (lines 94-123): Generates unitary matrices via QR decomposition
   - Complex number sampling (lines 124-132): Samples complex numbers from Re/Im distributions

5. **Prelude Module** (lines 4-15):
   - Exports commonly used types: `ComplexDistribution`, distribution types, and re-exports from `rand` crates

**Exports**:
- `NanHandling`, `col_mean`, `col_varm`, `row_mean`, `row_varm` from meanvar module

### faer/src/stats/meanvar.rs

**Purpose**: Implements highly optimized mean and variance calculations for matrix rows and columns with NaN handling support.

**Key Components**:

1. **NanHandling Enum** (lines 6-13)
   - `Propagate`: NaNs are passed through arithmetic operations
   - `Ignore`: NaNs are skipped and excluded from count

2. **Helper Functions**:
   - `from_usize<T: RealField>(n: usize) -> T` (lines 15-18): Converts usize to field type
   - `reduce<T: ComplexField, S: pulp::Simd>()` (lines 20-30): Reduces SIMD counters to scalar

3. **Public API Functions**:

   **Column Mean** (lines 687-697):
   - `col_mean<T: ComplexField>(out: ColMut<'_, T>, mat: MatRef<'_, T>, nan: NanHandling)`
   - Computes mean of each column, stores results in `out`
   - Dispatches to optimized implementation based on NaN handling mode

   **Row Mean** (lines 700-707):
   - `row_mean<T: ComplexField>(out: RowMut<'_, T>, mat: MatRef<'_, T>, nan: NanHandling)`
   - Computes mean of each row by transposing and calling `col_mean`

   **Column Variance** (lines 710-724):
   - `col_varm<T: ComplexField>(out: ColMut<'_, T::Real>, mat: MatRef<'_, T>, col_mean: ColRef<'_, T>, nan: NanHandling)`
   - Computes variance of each column given pre-computed means
   - Returns real-valued variance (using `abs2()` for complex)

   **Row Variance** (lines 727-743):
   - `row_varm<T: ComplexField>(out: RowMut<'_, T::Real>, mat: MatRef<'_, T>, row_mean: RowRef<'_, T>, nan: NanHandling)`
   - Computes variance of each row by transposing and calling `col_varm`

4. **Optimized SIMD Implementations**:

   **Row-Major Ignore NaN**:
   - `col_mean_row_major_ignore_nan` (lines 31-157): SIMD mean computation skipping NaNs
   - `col_varm_row_major_ignore_nan` (lines 158-311): SIMD variance computation skipping NaNs
   - Uses 4-way unrolled loops for maximum throughput
   - Tracks non-NaN count per element
   - Handles head/tail masks for unaligned data

   **Row-Major Propagate NaN**:
   - `col_mean_row_major_propagate_nan` (lines 312-364): SIMD mean computation propagating NaNs
   - `col_varm_row_major_propagate_nan` (lines 365-469): SIMD variance computation propagating NaNs
   - Simpler implementation without NaN checking
   - Uses 4-way unrolling

5. **Fallback Implementations** (non-SIMD):
   - `col_mean_ignore_nan_fallback` (lines 470-494): Scalar mean computation ignoring NaNs
   - `col_varm_ignore_nan_fallback` (lines 495-533): Scalar variance computation ignoring NaNs
   - `col_mean_propagate_nan_fallback` (lines 534-552): Scalar mean computation propagating NaNs
   - `col_varm_propagate_nan_fallback` (lines 553-581): Scalar variance computation propagating NaNs

6. **Dispatch Functions** (choose optimal implementation):
   - `col_mean_ignore` (lines 582-602): Dispatches to SIMD or fallback for NaN-ignoring mean
   - `col_varm_ignore` (lines 603-631): Dispatches to SIMD or fallback for NaN-ignoring variance
   - `col_mean_propagate` (lines 632-655): Dispatches to SIMD or fallback for NaN-propagating mean
   - `col_varm_propagate` (lines 656-684): Dispatches to SIMD or fallback for NaN-propagating variance
   - Prefer SIMD for row-major matrices with multiple columns
   - Handle negative strides by reversing rows/columns

7. **Tests** (lines 744-1020):
   - `test_meanvar_propagate`: Tests with NaN propagation mode
   - `test_meanvar_ignore_nan_nonan_c32`: Tests c32 (complex f32) without NaNs
   - `test_meanvar_ignore_nan_yesnan_c32`: Tests c32 with NaNs in ignore mode
   - `test_meanvar_ignore_nan_nonan_c64`: Tests c64 (complex f64) without NaNs
   - `test_meanvar_ignore_nan_yesnan_c64`: Tests c64 with NaNs in ignore mode

## Architecture and Design Patterns

### Performance Optimizations

1. **SIMD Acceleration**: Uses `pulp` crate for portable SIMD
   - 4-way loop unrolling for maximum instruction-level parallelism
   - Vectorized operations for sum, comparison, and selection
   - Efficient reduction of SIMD registers

2. **Memory Layout Awareness**:
   - Detects row-major contiguous layout for optimal SIMD processing
   - Reverses matrices with negative strides before processing
   - Falls back to scalar code for unfavorable layouts

3. **Batch Processing**:
   - Processes 256 iterations before reducing counters (ignore NaN mode)
   - Prevents counter overflow while maintaining efficiency

### NaN Handling Strategy

1. **Propagate Mode**:
   - Standard arithmetic that propagates NaN through calculations
   - Simpler and faster implementation
   - Suitable when data is guaranteed NaN-free

2. **Ignore Mode**:
   - Filters out NaN values during computation
   - Adjusts denominators based on actual non-NaN count
   - More complex with per-element counting
   - Special cases: all NaN → result is NaN, single value → variance is zero

### Type System Usage

- Generic over `ComplexField` trait (supports real and complex numbers)
- Variance always returns real type (`T::Real`)
- Uses const generics and shape types for compile-time checks

## Usage Examples

From tests:
```rust
// Basic usage with propagate mode
let A = mat![[c32(1.2, 2.3), c32(3.4, 1.2)], [c32(1.7, -1.0), c32(-3.8, 1.95)]];
let mut col_mean = Col::zeros(A.nrows());
col_mean(col_mean.as_mut(), A.as_ref(), NanHandling::Propagate);

// Variance computation requires pre-computed mean
let mut col_var = Col::zeros(A.nrows());
col_varm(col_var.as_mut(), A.as_ref(), col_mean.as_ref(), NanHandling::Propagate);

// Ignore NaN mode - NaNs are filtered out
col_mean(col_mean.as_mut(), A.as_ref(), NanHandling::Ignore);
```

## Dependencies

- `crate::assert`: Assertion utilities
- `crate::internal_prelude::*`: Internal faer types (Mat, Col, Row, etc.)
- `faer_traits::RealReg`: Wrapper type for real-valued SIMD registers
- `pulp`: SIMD abstraction library
- `bytemuck`: Safe transmutation between types
- `rand` (feature-gated): Random number generation
- `rand_distr` (feature-gated): Random distributions

## Notes

- Module allows missing docs (`#![allow(missing_docs)]`)
- All public functions are `#[track_caller]` for better panic messages
- Extensive test coverage for both real and complex types
- Variance uses Bessel's correction (n-1 denominator) for sample variance
