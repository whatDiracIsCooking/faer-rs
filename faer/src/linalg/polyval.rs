//! Matrix polynomial evaluation.
//!
//! Given a square matrix `A` and coefficients `[c_0, c_1, ..., c_n]`,
//! computes `p(A) = c_0 * I + c_1 * A + c_2 * A^2 + ... + c_n * A^n`.
//!
//! Two algorithms are provided:
//! - [`polyval`]: Horner's method, using `n` matrix multiplications.
//! - [`polyval_ps`]: Paterson-Stockmeyer algorithm, using ~`2√n` matrix multiplications.

use crate::assert;
use crate::internal_prelude::*;
use crate::{Accum, Scale, get_global_parallelism};
use faer_traits::ComplexField;

/// Evaluates a polynomial with the given coefficients at the matrix `matrix`.
///
/// The coefficients are ordered from lowest degree to highest:
/// `coeffs = [c_0, c_1, ..., c_n]` represents the polynomial
/// `p(x) = c_0 + c_1 * x + c_2 * x^2 + ... + c_n * x^n`.
///
/// The input array has size `n + 1` for a polynomial of degree `n`.
///
/// # Panics
///
/// Panics if the matrix is not square.
/// Panics if `coeffs` is empty.
pub fn polyval<T: ComplexField>(matrix: MatRef<'_, T>, coeffs: &[T]) -> Mat<T> {
	assert!(!coeffs.is_empty());
	let n = matrix.nrows();
	assert!(n == matrix.ncols());

	let par = get_global_parallelism();

	// Horner's method: p(A) = (...((c_n * A + c_{n-1} * I) * A + c_{n-2} * I) ... ) * A + c_0 * I
	// Start with result = c_n * I, then iterate:
	//   result = result * A + c_{k} * I

	let degree = coeffs.len() - 1;

	// Start: result = c_n * I
	let mut result = Mat::<T>::zeros(n, n);
	for i in 0..n {
		result[(i, i)] = coeffs[degree].clone();
	}

	// Iterate from c_{n-1} down to c_0
	for k in (0..degree).rev() {
		// result = result * A + c_k * I
		let mut new_result = Mat::<T>::zeros(n, n);
		crate::linalg::matmul::matmul(
			new_result.as_mut(),
			Accum::Replace,
			result.as_ref(),
			matrix,
			T::one_impl(),
			par,
		);
		// Add c_k * I
		for i in 0..n {
			new_result[(i, i)] = new_result[(i, i)].clone() + coeffs[k].clone();
		}
		result = new_result;
	}

	result
}

/// Evaluates a matrix polynomial using the Paterson-Stockmeyer algorithm.
///
/// This is more efficient than Horner's method ([`polyval`]) for polynomials of
/// degree >= 4, reducing the number of matrix multiplications from `d` to
/// approximately `2√d`.
///
/// The algorithm works by choosing a block size `s ≈ √d`, precomputing
/// `A, A^2, ..., A^s`, then rewriting the polynomial in base-`s` blocks
/// and evaluating with Horner's method on those blocks.
///
/// The coefficients are ordered from lowest degree to highest:
/// `coeffs = [c_0, c_1, ..., c_n]` represents the polynomial
/// `p(x) = c_0 + c_1 * x + c_2 * x^2 + ... + c_n * x^n`.
///
/// # Panics
///
/// Panics if the matrix is not square.
/// Panics if `coeffs` is empty.
pub fn polyval_ps<T: ComplexField>(matrix: MatRef<'_, T>, coeffs: &[T]) -> Mat<T> {
	assert!(!coeffs.is_empty());
	let n = matrix.nrows();
	assert!(n == matrix.ncols());

	let par = get_global_parallelism();
	let degree = coeffs.len() - 1;

	// For small polynomials, fall back to Horner's method
	if degree <= 3 {
		return polyval(matrix, coeffs);
	}

	// Choose block size s ≈ √degree
	let s = (degree as f64).sqrt().ceil() as usize;

	// Precompute powers: powers[i] = A^(i+1) for i = 0..s-1
	// So powers[0] = A, powers[1] = A^2, ..., powers[s-1] = A^s
	let mut powers = Vec::with_capacity(s);
	powers.push(matrix.to_owned());
	for i in 1..s {
		let mut power = Mat::<T>::zeros(n, n);
		crate::linalg::matmul::matmul(
			power.as_mut(),
			Accum::Replace,
			powers[i - 1].as_ref(),
			matrix,
			T::one_impl(),
			par,
		);
		powers.push(power);
	}

	// Number of blocks: q+1 blocks where q = ceil((degree+1)/s) - 1
	let num_blocks = (coeffs.len() + s - 1) / s;

	// Evaluate block B_j = c_{j*s} * I + c_{j*s+1} * A + ... + c_{j*s+s-1} * A^{s-1}
	let eval_block = |block_idx: usize| -> Mat<T> {
		let start = block_idx * s;
		let mut block = Mat::<T>::zeros(n, n);

		// Add c_{start} * I (the constant term of this block)
		if start < coeffs.len() {
			for i in 0..n {
				block[(i, i)] = coeffs[start].clone();
			}
		}

		// Add c_{start+k} * A^k for k = 1..s-1
		for k in 1..s {
			if start + k < coeffs.len() {
				// block += c_{start+k} * powers[k-1]  (powers[k-1] = A^k)
				crate::linalg::matmul::matmul(
					block.as_mut(),
					Accum::Add,
					powers[k - 1].as_ref(),
					Mat::<T>::identity(n, n).as_ref(),
					coeffs[start + k].clone(),
					par,
				);
			}
		}

		block
	};

	// Horner evaluation on blocks using A^s as the variable:
	// result = (...((B_{q} * A^s + B_{q-1}) * A^s + B_{q-2}) ...) * A^s + B_0
	let a_s = &powers[s - 1]; // A^s

	let mut result = eval_block(num_blocks - 1);

	for j in (0..num_blocks - 1).rev() {
		// result = result * A^s + B_j
		let mut new_result = eval_block(j);
		crate::linalg::matmul::matmul(
			new_result.as_mut(),
			Accum::Add,
			result.as_ref(),
			a_s.as_ref(),
			T::one_impl(),
			par,
		);
		result = new_result;
	}

	result
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{Mat, assert};

	#[test]
	fn test_identity_matrix() {
		// If the input matrix is the identity, then A^k = I for all k,
		// so p(I) = (c_0 + c_1 + ... + c_n) * I
		let coeffs = [1.0, 2.0, 3.0, 4.0];
		let expected_sum: f64 = coeffs.iter().sum();

		for size in [1, 2, 5, 10] {
			let identity = Mat::<f64>::identity(size, size);
			let result = polyval(identity.as_ref(), &coeffs);
			let expected = Mat::from_fn(size, size, |i, j| {
				if i == j { expected_sum } else { 0.0 }
			});
			assert!(result == expected);
		}
	}

	#[test]
	fn test_zero_matrix() {
		// If the input matrix is zero, then A^k = 0 for k >= 1,
		// so p(0) = c_0 * I
		let coeffs = [5.0, 2.0, 3.0, 4.0];

		for size in [1, 2, 5, 10] {
			let zero = Mat::<f64>::zeros(size, size);
			let result = polyval(zero.as_ref(), &coeffs);
			let expected = Mat::from_fn(size, size, |i, j| {
				if i == j { coeffs[0] } else { 0.0 }
			});
			assert!(result == expected);
		}
	}

	#[test]
	fn test_constant_polynomial() {
		// p(A) = c_0 * I for a constant polynomial (degree 0)
		let mat = Mat::from_fn(3, 3, |i, j| (i + j) as f64);
		let result = polyval(mat.as_ref(), &[7.0]);
		let expected = Mat::from_fn(3, 3, |i, j| if i == j { 7.0 } else { 0.0 });
		assert!(result == expected);
	}

	#[test]
	fn test_linear_polynomial() {
		// p(A) = c_0 * I + c_1 * A
		let mat = Mat::from_fn(3, 3, |i, j| (i * 3 + j + 1) as f64);
		let result = polyval(mat.as_ref(), &[2.0, 3.0]);
		let expected = &Mat::<f64>::identity(3, 3) * Scale(2.0) + &mat * Scale(3.0);
		assert!(result == expected);
	}

	#[test]
	fn test_quadratic_polynomial() {
		// p(A) = c_0 * I + c_1 * A + c_2 * A^2
		let mat = Mat::from_fn(3, 3, |i, j| (i * 3 + j + 1) as f64);
		let a_squared = &mat * &mat;
		let result = polyval(mat.as_ref(), &[1.0, 2.0, 3.0]);
		let expected = &Mat::<f64>::identity(3, 3) * Scale(1.0)
			+ &(&mat * Scale(2.0))
			+ &(&a_squared * Scale(3.0));
		assert!(result == expected);
	}

	#[test]
	#[should_panic]
	fn test_empty_coefficients_panics() {
		let mat = Mat::<f64>::zeros(2, 2);
		polyval(mat.as_ref(), &[]);
	}

	#[test]
	#[should_panic]
	fn test_non_square_matrix_panics() {
		let mat = Mat::<f64>::zeros(2, 3);
		polyval(mat.as_ref(), &[1.0]);
	}

	// Paterson-Stockmeyer tests

	#[test]
	fn test_ps_matches_horner_degree4() {
		// Degree 4: first degree where PS doesn't fall back to Horner
		let mat = Mat::from_fn(4, 4, |i, j| (i * 4 + j + 1) as f64 * 0.1);
		let coeffs = [1.0, -2.0, 3.0, -4.0, 5.0];
		let horner = polyval(mat.as_ref(), &coeffs);
		let ps = polyval_ps(mat.as_ref(), &coeffs);
		for i in 0..4 {
			for j in 0..4 {
				assert!((horner[(i, j)] - ps[(i, j)]).abs() < 1e-10);
			}
		}
	}

	#[test]
	fn test_ps_matches_horner_degree7() {
		let mat = Mat::from_fn(3, 3, |i, j| (i * 3 + j + 1) as f64 * 0.05);
		let coeffs = [1.0, 2.0, -1.0, 0.5, -0.3, 0.7, -0.2, 0.1];
		let horner = polyval(mat.as_ref(), &coeffs);
		let ps = polyval_ps(mat.as_ref(), &coeffs);
		for i in 0..3 {
			for j in 0..3 {
				assert!((horner[(i, j)] - ps[(i, j)]).abs() < 1e-10);
			}
		}
	}

	#[test]
	fn test_ps_matches_horner_degree10() {
		let mat = Mat::from_fn(3, 3, |i, j| (i * 3 + j + 1) as f64 * 0.02);
		let coeffs: Vec<f64> = (0..11).map(|i| (i as f64 - 5.0) * 0.3).collect();
		let horner = polyval(mat.as_ref(), &coeffs);
		let ps = polyval_ps(mat.as_ref(), &coeffs);
		for i in 0..3 {
			for j in 0..3 {
				assert!((horner[(i, j)] - ps[(i, j)]).abs() < 1e-9);
			}
		}
	}

	#[test]
	fn test_ps_identity_matrix() {
		let coeffs: Vec<f64> = (1..=8).map(|i| i as f64).collect();
		let expected_sum: f64 = coeffs.iter().sum();
		let identity = Mat::<f64>::identity(4, 4);
		let result = polyval_ps(identity.as_ref(), &coeffs);
		for i in 0..4 {
			for j in 0..4 {
				let expected = if i == j { expected_sum } else { 0.0 };
				assert!((result[(i, j)] - expected).abs() < 1e-10);
			}
		}
	}

	#[test]
	fn test_ps_zero_matrix() {
		let coeffs = [5.0, 2.0, 3.0, 4.0, 1.0];
		let zero = Mat::<f64>::zeros(3, 3);
		let result = polyval_ps(zero.as_ref(), &coeffs);
		for i in 0..3 {
			for j in 0..3 {
				let expected = if i == j { coeffs[0] } else { 0.0 };
				assert!((result[(i, j)] - expected).abs() < 1e-10);
			}
		}
	}

	#[test]
	fn test_ps_fallback_small_degree() {
		// Degree <= 3 should fall back to Horner
		let mat = Mat::from_fn(3, 3, |i, j| (i * 3 + j + 1) as f64);
		let coeffs = [1.0, 2.0, 3.0];
		let horner = polyval(mat.as_ref(), &coeffs);
		let ps = polyval_ps(mat.as_ref(), &coeffs);
		assert!(horner == ps);
	}

	#[test]
	#[should_panic]
	fn test_ps_empty_coefficients_panics() {
		let mat = Mat::<f64>::zeros(2, 2);
		polyval_ps(mat.as_ref(), &[]);
	}
}
