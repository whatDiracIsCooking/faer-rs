# faer-ffi/src Memory Document

## Overview
The `faer-ffi/src` directory contains the Foreign Function Interface (FFI) layer for the faer linear algebra library. This enables faer to be called from other languages like C, C++, Python, etc.

## File: lib.rs (faer-ffi/src/lib.rs)

### Purpose
The main FFI module that provides C-compatible bindings to faer's linear algebra operations. All exported functions follow a naming convention: `libfaer_v0_23_<operation>_<type>`.

---

## Core Data Structures

### Matrix and Vector Types

**MatRef** (lines 14-20)
- C-compatible read-only matrix reference
- Fields: `ptr: *const c_void`, `nrows`, `ncols`, `row_stride`, `col_stride`
- Converts to faer's `MatRef<'a, T>` via `faer()` method

**MatMut** (lines 23-29)
- C-compatible mutable matrix reference
- Same structure as MatRef but with mutable pointer
- Converts to faer's `MatMut<'a, T>`

**VecRef** (lines 32-36)
- C-compatible read-only vector reference
- Fields: `ptr: *const c_void`, `len`, `stride`
- Can convert to: `DiagRef`, `ColRef`, or `RowRef`

**VecMut** (lines 39-43)
- C-compatible mutable vector reference
- Can convert to: `DiagMut`, `ColMut`, or `RowMut`

**SliceRef** (lines 46-49) and **SliceMut** (lines 52-55)
- C-compatible slice types without stride
- Used for raw memory access

### Enumerations

**Accum** (lines 61-64)
- `Replace`: Overwrite destination
- `Add`: Add to destination
- Used in matrix operations to control accumulation

**Conj** (lines 67-70)
- `No`: Don't conjugate
- `Yes`: Conjugate (for complex numbers)

**ParTag** (lines 73-76)
- `Seq`: Sequential execution
- `Rayon`: Parallel execution with thread pool

**Par** (lines 79-82)
- Parallelization configuration
- Fields: `tag: ParTag`, `nthreads: usize`

**Block** (lines 85-93)
- Describes matrix block structure
- Variants: `Rectangular`, `TriangularLower`, `TriangularUpper`, `StrictTriangularLower`, `StrictTriangularUpper`, `UnitTriangularLower`, `UnitTriangularUpper`

### Memory Management

**Layout** (lines 96-107)
- Memory layout descriptor
- Fields: `len_bytes`, `align_bytes`
- Converts from/to `StackReq`

**MemAlloc** (lines 110-124)
- Memory allocation descriptor
- Fields: `ptr: *mut c_void`, `len_bytes`
- Converts to faer's `MemStack` via `faer()` method

---

## Macro System

### funcs! macro (lines 276-371)
- Generates type-specialized FFI functions
- Creates separate functions for each numeric type: `f32`, `f64`, `fx128`, `c32`, `c64`, `cx128`
- Handles both single and dual type parameters (for index types)
- Naming pattern: `libfaer_v0_23_<func>_<type>`
- Example: `matmul<T>` generates `libfaer_v0_23_matmul_f32`, `libfaer_v0_23_matmul_f64`, etc.

### cerr! macro (lines 372-406)
- Defines C-compatible error/result enums
- Converts Rust `Result` types to C enums with explicit variants
- Includes `Unknown` variant for unhandled errors

### cparams! macro (lines 408-456)
- Defines C-compatible parameter structs
- Auto-generates default constructors for each type
- Bidirectional conversion between C and Rust types
- Fills missing fields with `faer::auto!(f32)` defaults

---

## Linear Algebra Operations (linalg module, lines 458-2522)

### Decompositions Supported

#### 1. Cholesky Decompositions

**LLT (Standard Cholesky)** (lines 984-1075)
- Functions: `llt_factor_in_place`, `llt_solve_in_place`, `llt_reconstruct`, `llt_inverse`
- Parameters: `LltParams` (recursion_threshold, block_size)
- Regularization: `LltRegularization` (dynamic regularization support)
- Status: `LltStatus` (Ok or NonPositivePivot error)

**Pivoted LLT** (lines 1076-1188)
- Functions: `piv_llt_factor_in_place`, `piv_llt_solve_in_place`, `piv_llt_reconstruct`, `piv_llt_inverse`
- Parameters: `PivLltParams` (block_size)
- Additional data: permutation arrays (fwd/bwd)
- Status: `PivLltStatus` (includes rank and transposition_count)

**LDLT** (lines 1190-1287)
- Functions: `ldlt_factor_in_place`, `ldlt_solve_in_place`, `ldlt_reconstruct`, `ldlt_inverse`
- Parameters: `LdltParams` (recursion_threshold, block_size)
- Regularization: `LdltRegularization` (with optional signs tracking)
- Status: `LdltStatus` (Ok or ZeroPivot error)

**LBLT (Bunch-Kaufman)** (lines 1289-1421)
- Functions: `lblt_factor_in_place`, `lblt_solve_in_place`, `lblt_reconstruct`, `lblt_inverse`
- Parameters: `LbltParams` (pivoting strategy, par_threshold, block_size)
- Pivoting strategies: Partial, PartialDiag, Rook, RookDiag, Full
- Additional data: subdiagonal vector, permutation arrays
- Status: `LbltStatus` (always Ok, includes transposition_count)

#### 2. QR Decompositions

**Standard QR** (lines 1520-1720)
- Functions: `qr_factor_in_place`, `qr_solve_in_place`, `qr_solve_transpose_in_place`, `qr_solve_lstsq_in_place`, `qr_reconstruct`, `qr_inverse`
- Parameters: `QrParams` (blocking_threshold, par_threshold)
- Recommended block size function provided
- Householder representation (Q_basis, Q_coeff)

**Column-Pivoted QR** (lines 1721-1950)
- Functions: `colpiv_qr_factor_in_place`, `colpiv_qr_solve_in_place`, `colpiv_qr_solve_transpose_in_place`, `colpiv_qr_solve_lstsq_in_place`, `colpiv_qr_reconstruct`, `colpiv_qr_inverse`
- Parameters: `ColPivQrParams` (blocking_threshold, par_threshold)
- Additional data: permutation arrays
- Better for rank-deficient matrices

#### 3. LU Decompositions

**Partial Pivoting LU** (lines 1952-2124)
- Functions: `partial_piv_lu_factor_in_place`, `partial_piv_lu_solve_in_place`, `partial_piv_lu_solve_transpose_in_place`, `partial_piv_lu_reconstruct`, `partial_piv_lu_inverse`
- Parameters: `PartialPivLuParams` (recursion_threshold, block_size, par_threshold)
- Row permutations only

**Full Pivoting LU** (lines 2125-2325)
- Functions: `full_piv_lu_factor_in_place`, `full_piv_lu_solve_in_place`, `full_piv_lu_solve_transpose_in_place`, `full_piv_lu_reconstruct`, `full_piv_lu_inverse`
- Parameters: `FullPivLuParams` (par_threshold)
- Both row and column permutations
- More stable but slower than partial pivoting

#### 4. Eigenvalue Decompositions (EVD)

**Self-Adjoint EVD** (lines 2368-2400)
- Functions: `self_adjoint_evd_scratch`, `self_adjoint_evd`
- For Hermitian/symmetric matrices
- Parameters: `SelfAdjointEvdParams` (tridiag params, recursion_threshold)
- Real eigenvalues guaranteed

**General EVD** (lines 2402-2457)
- Functions: `evd_scratch`, `evd`
- For non-symmetric matrices
- Parameters: `EvdParams` (hessenberg, schur, evd_from_schur params)
- Separate functions for real vs complex matrices
- Complex eigenvalues possible (stored in S and S_im for real matrices)
- Left and right eigenvectors optional

**Generalized EVD** (lines 2459-2520)
- Functions: `generalized_evd_scratch`, `generalized_evd`
- Solves A*x = lambda*B*x
- Parameters: `GevdParams` (hessenberg, schur, gevd_from_schur params)
- Eigenvalues as alpha/beta pairs

#### 5. Singular Value Decomposition (SVD)

**SVD** (lines 2327-2366)
- Functions: `svd_scratch`, `svd`
- Computes A = U * S * V^H
- Parameters: `SvdParams` (bidiag, qr params, recursion_threshold, qr_ratio_threshold)
- Control vector computation: None, Thin, or Full
- Can skip computing U or V if not needed

### Specialized Operations

**Matrix Multiplication** (lines 854-894)
- `matmul`: General matrix multiplication
- `matmul_triangular`: Triangular matrix multiplication with block structure

**Triangular Solves** (lines 896-937)
- `solve_triangular_lower_in_place` / `solve_triangular_upper_in_place`
- `solve_unit_triangular_lower_in_place` / `solve_unit_triangular_upper_in_place`
- All support conjugation

**Triangular Inverse** (lines 939-982)
- `inverse_triangular_lower_in_place` / `inverse_triangular_upper_in_place`
- `inverse_unit_triangular_lower_in_place` / `inverse_unit_triangular_upper_in_place`

**Householder Operations** (lines 1422-1518)
- `apply_householder_on_the_left` / `apply_householder_on_the_right`
- `apply_householder_transpose_on_the_left` / `apply_householder_transpose_on_the_right`
- Used for QR-based operations

---

## Global Functions

**Parallelism Control** (lines 2524-2542)
- `libfaer_v0_23_get_global_par()`: Get global parallelism setting
- `libfaer_v0_23_set_global_par(par)`: Set global parallelism

**Memory Allocation** (lines 2544-2569)
- `libfaer_v0_23_alloc(size, align)`: Allocate aligned memory
- `libfaer_v0_23_dealloc(ptr, size, align)`: Deallocate memory
- Uses Rust's global allocator

---

## Type System

### Supported Numeric Types
1. **f32**: 32-bit float
2. **f64**: 64-bit float
3. **fx128**: 128-bit float (extended precision)
4. **c32**: 32-bit complex (Complex<f32>)
5. **c64**: 64-bit complex (Complex<f64>)
6. **cx128**: 128-bit complex (Complex<fx128>)

### Index Types
- **u32**: 32-bit unsigned (for matrices up to 4B elements)
- **u64**: 64-bit unsigned (for very large matrices)

---

## Design Patterns

### Memory Management
- Caller responsible for allocation/deallocation
- Scratch space requirements computed via `*_scratch()` functions
- `MemAlloc` wraps pre-allocated memory
- In-place operations to minimize allocations

### Error Handling
- C-compatible enum-based error types
- Status structs with discriminated unions
- `Unknown` variant for unexpected errors
- Most operations return status/info structs

### Type Safety
- Opaque pointers (`*const Scalar`, `*const Real`) read via helper functions
- Runtime type dispatch through separate function variants
- Compile-time type checking on Rust side

### Parallelism
- Thread-safe operations with `Par` parameter
- Global parallelism setting
- Rayon-based parallel execution
- Threshold-based parallelism decisions

---

## Key Implementation Details

1. **Unsafe Code**: Extensive use of `unsafe` for FFI boundary
2. **Stride Support**: All matrix/vector types support custom strides
3. **Conjugation**: Complex operations support optional conjugation
4. **In-Place Operations**: Most operations modify input matrices
5. **Block Algorithms**: Block-oriented algorithms for cache efficiency
6. **Pivoting Strategies**: Multiple pivoting options for different stability/performance tradeoffs

---

## Usage Pattern

Typical workflow for decomposition-based solve:
1. Call `*_scratch()` to get memory requirements
2. Allocate memory using `libfaer_v0_23_alloc()`
3. Create `MemAlloc` from allocated buffer
4. Call `*_factor_in_place()` to compute decomposition
5. Call `*_solve_in_place()` to solve linear system
6. Deallocate memory using `libfaer_v0_23_dealloc()`

---

## Notes

- All functions use `#[unsafe(no_mangle)]` for C linkage
- Functions use `extern "C"` calling convention
- Version in function names: `v0_23` corresponds to faer version
- Feature-gated: `linalg` module requires `linalg` feature flag
- Marker types: `Scalar`, `Index`, `Real` are zero-sized marker enums
