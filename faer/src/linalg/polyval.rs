//! Matrix polynomial evaluation using Horner's method.
//!
//! Given a square matrix `A` and coefficients `[c_0, c_1, ..., c_n]`,
//! computes `p(A) = c_0 * I + c_1 * A + c_2 * A^2 + ... + c_n * A^n`
//! using Horner's method: `p(A) = (...((c_n * A + c_{n-1}) * A + c_{n-2}) * A + ...) * A + c_0) * I`

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
}
