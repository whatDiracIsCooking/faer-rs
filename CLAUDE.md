# CLAUDE.md - faer Development Guide for AI Assistants

This document provides comprehensive guidance for AI assistants working on the `faer` codebase, a high-performance linear algebra library for Rust.

## Project Overview

**faer** is a pure Rust linear algebra library focused on:
- **Portability**: Cross-platform support (x86-64, Aarch64, etc.)
- **Correctness**: Memory-safe implementation leveraging Rust's type system
- **Performance**: SIMD optimizations, explicit vectorization, competitive with BLAS libraries

**Key Information:**
- **Repository**: https://codeberg.org/sarah-quinones/faer
- **Documentation**: https://docs.rs/faer
- **Website**: https://faer.veganb.tw
- **Community**: [Zulip chat](https://faer.zulipchat.com)
- **License**: MIT
- **MSRV**: Rust 1.84.0

## Repository Structure

This is a Cargo workspace with multiple crates:

```
faer-rs/
├── faer/              # Main library (core functionality, decompositions)
│   ├── src/
│   │   ├── lib.rs     # Library entry point with comprehensive documentation
│   │   ├── mat/       # Matrix types (Mat, MatRef, MatMut)
│   │   ├── col/       # Column vector types
│   │   ├── row/       # Row vector types
│   │   ├── linalg/    # Linear algebra algorithms
│   │   │   ├── cholesky/     # LLT/LBLT/LDLT decompositions
│   │   │   ├── lu/           # LU with partial/full pivoting
│   │   │   ├── qr/           # QR decomposition (with/without pivoting)
│   │   │   ├── svd/          # Singular value decomposition
│   │   │   ├── evd/          # Eigenvalue decomposition
│   │   │   ├── gevd/         # Generalized eigenvalue decomposition
│   │   │   ├── matmul/       # Matrix multiplication kernels
│   │   │   └── reductions/   # Norms, determinants, etc.
│   │   ├── sparse/    # Sparse matrix support
│   │   ├── stats/     # Statistical functions (mean, variance, random generation)
│   │   ├── perm/      # Permutation matrices
│   │   ├── utils/     # Utility functions
│   │   ├── serde/     # Serialization support
│   │   └── io.rs      # I/O operations (NumPy format support)
│   └── examples/      # Example programs and benchmarks
├── faer-traits/       # Core trait definitions and type abstractions
├── faer-macros/       # Procedural macros
├── faer-ffi/          # C FFI bindings (cdylib)
├── faer-no-std-test/  # No-std compatibility testing
└── eigen-bench-setup/ # Eigen benchmark comparison setup (excluded from workspace)
```

### Workspace Members
- **faer**: Main library with all core implementations
- **faer-traits**: Low-level traits for numeric types and SIMD operations
- **faer-macros**: Procedural macros for code generation
- **faer-ffi**: Foreign function interface for C compatibility

### Excluded Projects
- **faer-no-std-test**: Tests for no-std environments (separate from main workspace)
- **eigen-bench-setup**: Benchmark comparison tooling

## Build System

### Cargo Configuration

**Workspace-level** (`/Cargo.toml`):
- Uses Cargo resolver v2
- Development profile: `opt-level = 3` (optimized builds even in dev mode)
- No LTO in development for faster compile times

**Main crate features** (`/faer/Cargo.toml`):
- `default`: Includes `std`, `rayon`, `sparse-linalg`, `rand`, `npy`
- `std`: Standard library support (required for most functionality)
- `nightly`: Enables x86-v4 SIMD instructions
- `rayon`: Parallel computation support
- `sparse`: Sparse matrix types
- `sparse-linalg`: Sparse linear algebra operations
- `linalg`: Dense linear algebra operations
- `unstable`: Experimental features
- `perf-warn`: Performance warning logs
- `rand`: Random number generation
- `serde`: Serialization/deserialization
- `npy`: NumPy file format support

### Key Dependencies
- `pulp`: SIMD abstraction library (v0.22.1)
- `dyn-stack`: Dynamic stack allocation (v0.13.2)
- `num-complex`, `num-traits`: Numeric type support
- `gemm`, `nano-gemm`: Matrix multiplication backends
- `reborrow`, `generativity`: Memory safety abstractions
- `rayon`: Data parallelism (optional)
- `bytemuck`: Safe type casting

## Code Style and Formatting

### rustfmt Configuration (`/rustfmt.toml`)

**Critical conventions:**
```toml
hard_tabs = true              # ALWAYS use tabs, not spaces
max_width = 80                # 80 character line limit
comment_width = 100           # Wrap comments at 100 characters
style_edition = "2024"        # Use latest Rust formatting conventions
imports_granularity = "Module" # Group imports by module
```

**Formatting rules:**
- Use hard tabs for indentation
- 80-character line width for code
- 100-character width for comments (wrap_comments = true)
- Format code in doc comments
- Normalize doc attributes
- Use field init shorthand
- Reorder impl items
- Match block trailing commas

### Code Quality Tools

**Before committing, ensure:**
1. `cargo fmt --all` passes (nightly toolchain required for some features)
2. `cargo clippy --all-targets` has no warnings
3. Tests pass: `cargo test` (or `cargo nextest run`)

## CI/CD Workflows

### GitHub Actions (`.github/workflows/`)

**1. run-tests.yml** (Triggered on: push to main/refactor-3, PRs)
- **MSRV check**: Verifies code builds on Rust 1.84.0 (ubuntu-latest, windows-latest)
- **no-std check**: Tests no-std compatibility with nightly-2025-01-08
- **Testing + Coverage**:
  - Runs on stable Rust (ubuntu-latest)
  - Uses nextest for test execution
  - Generates coverage with cargo-llvm-cov
  - Uploads to codecov
  - Tests with `unstable` feature flag

**2. code-quality.yml** (Triggered on: push to dev/main, PRs)
- Runs on nightly toolchain
- **Formatting**: `cargo fmt --all -- --check`
- **Linting**: `cargo clippy --all-targets`
- Both checks continue-on-error (informational)

### Test Commands
```bash
# Standard tests
cargo test

# With coverage (requires cargo-llvm-cov)
cargo llvm-cov nextest --features=unstable --lcov --output-path lcov.info

# MSRV check
cargo +1.84.0 check

# no-std compatibility
cd faer-no-std-test && cargo run --profile nostd
```

## Development Workflows

### Making Changes

1. **Understand the context**:
   - Check relevant documentation in `/faer/src/lib.rs`
   - Review similar implementations in the same module
   - Consult academic papers referenced in comments

2. **Code organization**:
   - Dense algorithms: `/faer/src/linalg/`
   - Sparse algorithms: `/faer/src/sparse/linalg/`
   - Type definitions: `/faer/src/mat/`, `/faer/src/col/`, `/faer/src/row/`
   - Utilities: `/faer/src/utils/`

3. **Testing**:
   - Unit tests typically in the same file (or `mod.rs`)
   - Integration tests in module-level tests
   - Benchmark examples in `/faer/examples/`

4. **Performance considerations**:
   - SIMD operations via `pulp` crate
   - Memory allocation through `dyn-stack`
   - Parallelization via `rayon` (when feature enabled)
   - Matrix multiplication through specialized kernels in `linalg/matmul/`

### Adding New Features

**For new matrix decompositions:**
1. Create module in `/faer/src/linalg/<name>/`
2. Implement low-level algorithm (consider SIMD opportunities)
3. Add high-level API to relevant types (Mat, MatRef, etc.)
4. Write comprehensive tests
5. Add documentation with mathematical notation
6. Consider sparse variant in `/faer/src/sparse/linalg/`

**For new numeric types:**
1. Implement required traits from `faer-traits`
2. Test with existing decompositions
3. Document limitations (e.g., iterative algorithms)

### Documentation Standards

**Use mathematical notation** (rendered with KaTeX in docs):
```rust
/// Computes the LLT decomposition such that $A = LL^H$,
/// where $L$ is lower triangular.
```

**Document complexity and stability**:
```rust
/// # Computational Complexity
/// O(n³) for an n×n matrix
///
/// # Numerical Stability
/// Backward stable for well-conditioned matrices
```

**Provide examples**:
```rust
/// # Example
/// ```
/// use faer::mat;
/// let a = mat![[1.0, 0.0], [0.0, 1.0]];
/// let llt = a.llt(Default::default());
/// ```
```

## Key Conventions

### Memory Management
- Use `dyn-stack` for temporary allocations in hot paths
- Avoid heap allocation in inner loops
- Prefer stack allocation for small matrices (via `Mat::from_fn`)
- Use views (`MatRef`, `MatMut`) to avoid copies

### Parallelism
- Respect `Parallelism` parameter in function signatures
- Don't assume parallelism is always available (it's optional)
- Use `rayon` only when feature is enabled
- Document parallel behavior in function docs

### Numeric Types
- Support `f32`, `f64`, `c32`, `c64` out of the box
- Be generic over `ComplexField` or `RealField` traits
- Handle edge cases: NaN, infinity, denormals
- Document precision requirements

### Error Handling
- Use `assert!` for invariant violations (programmer errors)
- Return `Result` for recoverable errors
- Panic with descriptive messages for dimension mismatches
- Document panic conditions

### API Design
- Owned types: `Mat`, `Col`, `Row`
- Immutable views: `MatRef`, `ColRef`, `RowRef`
- Mutable views: `MatMut`, `ColMut`, `RowMut`
- Use reborrowing for mutable views
- Provide both high-level and low-level APIs

## Testing Strategy

### Test Organization
- Unit tests: In same file as implementation
- Module tests: In `mod.rs` or `tests/` subdirectory
- Integration tests: In workspace `tests/` directory
- Benchmarks: In `/faer/examples/bench*.rs`

### Test Coverage
- Test numerical correctness against reference implementations
- Test edge cases: empty matrices, single element, very large
- Test all supported numeric types
- Test with and without parallelism
- Test no-std compatibility (in separate crate)

### Benchmark Comparisons
- Compare against Eigen (C++), OpenBLAS, MKL
- Use `diol` for microbenchmarking
- Configuration via `bench.toml`
- See https://faer.veganb.tw/benchmarks/

## Common Pitfalls

### DO:
- Use tabs for indentation (hard_tabs = true)
- Test on MSRV (1.84.0) before committing
- Document mathematical algorithms with references
- Consider SIMD opportunities for hot paths
- Use `dyn-stack` for temporary allocations
- Write doc tests for public APIs
- Check for dimension compatibility
- Handle singular/ill-conditioned matrices gracefully

### DON'T:
- Use spaces for indentation (will fail CI)
- Assume specific SIMD features (use runtime detection)
- Ignore overflow/underflow in computations
- Make breaking changes without discussion
- Skip clippy/formatting checks
- Add dependencies without justification
- Assume `std` is available (support `no_std`)
- Use `unwrap()` in library code (except in tests/examples)

## Platform-Specific Considerations

### SIMD Support
- x86-64: Automatic feature detection (SSE2, AVX, AVX2, FMA)
- x86-64 nightly: AVX-512 (x86-v4) with `nightly` feature
- Aarch64: NEON intrinsics
- Future: SVE/SME, RISC-V RVV (when stabilized)

### Target-Specific Dependencies
- `private-gemm-x86`: x86_64 only (optimized kernels)
- OpenBLAS/MKL/BLIS: Optional, for benchmarking
- Test against system libraries when available

## Resources

### Documentation
- API docs: https://docs.rs/faer/latest/faer
- Website: https://faer.veganb.tw
- Examples: `/faer/examples/`
- Academic paper: `/paper.md` (JOSS submission)

### Community
- Zulip chat: https://faer.zulipchat.com
- Issues: Good first issues tagged on GitHub
- Changelog: `/CHANGELOG.md`

### Related Projects
- `nalgebra`: Alternative Rust linear algebra (different design focus)
- BLAS/LAPACK: Reference implementations
- Eigen: C++ library (used for benchmarks)

## Version Information

- **Current Version**: 0.23.2
- **MSRV**: 1.84.0
- **Edition**: Rust 2021
- **API Stability**: Pre-1.0 (breaking changes possible between minor versions)

## Contributing Guidelines

1. **Ask questions**: Use Zulip or GitHub issues before large changes
2. **Start small**: Look for "good first issue" labels
3. **Follow conventions**: Read this document and existing code
4. **Test thoroughly**: Include tests with PRs
5. **Document clearly**: Explain algorithms and design decisions
6. **Be patient**: Performance-critical code requires careful review

---

**Last Updated**: 2025-11-15
**Document Version**: 1.0
**Maintainer**: AI-generated for AI assistant guidance
