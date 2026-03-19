pub use super::super::no_pivoting::factor::recommended_block_size;

use crate::assert;
use crate::internal_prelude::*;
use crate::perm::swap_cols_idx;

/// Computes the layout of required workspace for performing a sketched QR
/// decomposition.
pub fn qr_in_place_scratch<I: Index, T: ComplexField>(
	nrows: usize,
	ncols: usize,
	sketch_nrows: usize,
	block_size: usize,
	par: Par,
	params: Spec<super::super::col_pivoting::factor::ColPivQrParams, T>,
) -> StackReq {
	let sketch_size = Ord::min(sketch_nrows, ncols);
	let sketch_bs = Ord::min(block_size, sketch_size);
	StackReq::all_of(&[
		// sketch matrix B = S * A
		linalg::temp_mat_scratch::<T>(sketch_nrows, ncols),
		// householder coefficients for CPQR on sketch
		linalg::temp_mat_scratch::<T>(sketch_bs, sketch_size),
		// max of CPQR workspace and no-pivoting QR workspace
		StackReq::any_of(&[
			super::super::col_pivoting::factor::qr_in_place_scratch::<I, T>(
				sketch_nrows,
				ncols,
				sketch_bs,
				par,
				params,
			),
			super::super::no_pivoting::factor::qr_in_place_scratch::<T>(
				nrows, ncols, block_size, par, Default::default(),
			),
		]),
	])
}

/// Information about the resulting sketched $QR$ factorization.
#[derive(Copy, Clone, Debug)]
pub struct SketchedQrInfo {
	/// Number of transpositions in the column permutation.
	pub transposition_count: usize,
}

/// Computes the sketched column-pivoted QR factorization of matrix $A$.
///
/// The algorithm:
/// 1. Generates a Gaussian sketch matrix $S$ of size `sketch_nrows × nrows`
///    with i.i.d. $\mathcal{N}(0,1)$ entries.
/// 2. Computes $B = SA$ (size `sketch_nrows × ncols`).
/// 3. Runs column-pivoting QR on $B$ to determine the column permutation $P$.
/// 4. Applies $P$ to $A$ and runs standard (no-pivoting) QR on $AP^T$.
///
/// On output, `A` contains the QR factors of $AP^T$, and `col_perm` /
/// `col_perm_inv` describe the column permutation.
///
/// # Panics
///
/// Panics if `col_perm.len() != ncols` or `col_perm_inv.len() != ncols`.
#[track_caller]
pub fn qr_in_place<'out, I: Index, T: ComplexField>(
	A: MatMut<'_, T>,
	Q_coeff: MatMut<'_, T>,
	col_perm: &'out mut [I],
	col_perm_inv: &'out mut [I],
	sketch_nrows: usize,
	rng: &mut (impl ?Sized + rand::Rng),
	par: Par,
	stack: &mut MemStack,
	params: Spec<super::super::col_pivoting::factor::ColPivQrParams, T>,
) -> (SketchedQrInfo, PermRef<'out, I>)
where
	rand_distr::StandardNormal: rand::distr::Distribution<T::Real>,
{
	let m = A.nrows();
	let n = A.ncols();
	let size = Ord::min(m, n);
	let block_size = Q_coeff.nrows();

	assert!(col_perm.len() == n);
	assert!(col_perm_inv.len() == n);
	assert!(Q_coeff.ncols() == size);
	assert!(sketch_nrows > 0);

	let mut A = A;
	let mut Q_coeff = Q_coeff;

	// Step 1: Generate sketch matrix S (sketch_nrows × m) and compute B = S*A.
	let (mut B, stack) =
		linalg::temp_mat_zeroed::<T, _, _>(sketch_nrows, n, stack);
	let mut B = B.as_mat_mut();

	use rand::distr::Distribution;
	let normal = rand_distr::StandardNormal;
	for j in 0..m {
		let a_row = A.rb().row(j);
		for i in 0..sketch_nrows {
			let s_ij: T::Real = normal.sample(rng);
			let s_val = from_real::<T>(&s_ij);
			z!(B.rb_mut().row_mut(i), a_row).for_each(|uz!(b, a)| {
				*b += &s_val * a;
			});
		}
	}

	// Step 2: Run CPQR on sketch B to get column permutation.
	let sketch_size = Ord::min(sketch_nrows, n);
	let sketch_bs = Ord::min(block_size, sketch_size);
	let (mut sketch_H, mut stack) =
		linalg::temp_mat_zeroed::<T, _, _>(sketch_bs, sketch_size, stack);

	let (cpqr_info, _) = super::super::col_pivoting::factor::qr_in_place(
		B.rb_mut(),
		sketch_H.as_mat_mut(),
		col_perm,
		col_perm_inv,
		par,
		stack.rb_mut(),
		params,
	);
	let n_trans = cpqr_info.transposition_count;

	// Step 3: Apply permutation to columns of A using cycle decomposition.
	// col_perm[j] = k means original column k goes to position j.
	// We walk through cycles and rotate columns in-place.
	// col_perm_inv is preserved; col_perm is temporarily modified then restored.
	{
		for i in 0..n {
			if col_perm[i].zx() == i {
				continue;
			}
			let mut j = i;
			loop {
				let next = col_perm[j].zx();
				if next == i {
					col_perm[j] = I::truncate(j);
					break;
				}
				swap_cols_idx(A.rb_mut(), j, next);
				col_perm[j] = I::truncate(j); // mark visited
				j = next;
			}
		}

		// Restore col_perm from col_perm_inv.
		for k in 0..n {
			col_perm[col_perm_inv[k].zx()] = I::truncate(k);
		}
	}

	// Step 4: Run no-pivoting QR on the permuted A.
	super::super::no_pivoting::factor::qr_in_place(
		A.rb_mut(),
		Q_coeff.rb_mut(),
		par,
		stack.rb_mut(),
		Default::default(),
	);

	(
		SketchedQrInfo {
			transposition_count: n_trans,
		},
		unsafe { PermRef::new_unchecked(col_perm, col_perm_inv, n) },
	)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::stats::prelude::*;
	use crate::utils::approx::*;
	use crate::{Mat, assert, c64};
	use dyn_stack::MemBuffer;
	use linalg::householder;

	/// Helper: build an m×n matrix with orthonormal columns scaled so that
	/// column j has norm `2^{scale_fn(j)}`.
	fn build_scaled_matrix(
		m: usize,
		n: usize,
		bs: usize,
		rng: &mut StdRng,
		scale_fn: impl Fn(usize) -> i32,
	) -> Mat<c64> {
		let size = Ord::min(m, n);

		let Q_rand = CwiseMatDistribution {
			nrows: m,
			ncols: n,
			dist: ComplexDistribution::new(StandardNormal, StandardNormal),
		}
		.rand::<Mat<c64>>(rng);

		let mut QR_tmp = Q_rand.as_ref().cloned();
		let mut H_tmp = Mat::zeros(bs, size);
		crate::linalg::qr::no_pivoting::factor::qr_in_place(
			QR_tmp.as_mut(),
			H_tmp.as_mut(),
			Par::Seq,
			MemStack::new(&mut MemBuffer::new(
				crate::linalg::qr::no_pivoting::factor::qr_in_place_scratch::<
					c64,
				>(m, n, bs, Par::Seq, Default::default()),
			)),
			Default::default(),
		);

		let mut Q_orth = Mat::<c64>::zeros(m, n);
		for j in 0..n {
			Q_orth[(j, j)] = c64::ONE;
		}
		householder::apply_block_householder_sequence_on_the_left_in_place_with_conj(
			QR_tmp.as_ref().subcols(0, size),
			H_tmp.as_ref(),
			Conj::No,
			Q_orth.as_mut(),
			Par::Seq,
			MemStack::new(&mut MemBuffer::new(
				householder::apply_block_householder_sequence_on_the_left_in_place_scratch::<c64>(m, bs, n),
			)),
		);

		for j in 0..n {
			let scale = c64::new(f64::powi(2.0, scale_fn(j)), 0.0);
			z!(Q_orth.as_mut().col_mut(j))
				.for_each(|uz!(a)| *a = &*a * &scale);
		}
		Q_orth
	}

	/// Helper: verify that Q * R * P ≈ A_orig for a completed factorization.
	fn verify_qrp(
		A_qr: MatRef<'_, c64>,
		H: MatRef<'_, c64>,
		perm: PermRef<'_, usize>,
		A_orig: MatRef<'_, c64>,
		m: usize,
		n: usize,
		bs: usize,
	) {
		let size = Ord::min(m, n);
		let approx_eq = CwiseMat(ApproxEq {
			abs_tol: 1e-10,
			rel_tol: 1e-10,
		});

		let mut Q = Mat::<c64>::zeros(m, m);
		let mut R = A_qr.cloned();
		for j in 0..m {
			Q[(j, j)] = c64::ONE;
		}
		householder::apply_block_householder_sequence_on_the_left_in_place_with_conj(
			A_qr.subcols(0, size),
			H,
			Conj::No,
			Q.as_mut(),
			Par::Seq,
			MemStack::new(&mut MemBuffer::new(
				householder::apply_block_householder_sequence_on_the_left_in_place_scratch::<c64>(m, bs, m),
			)),
		);
		for j in 0..n {
			for i in j + 1..m {
				R[(i, j)] = c64::ZERO;
			}
		}

		assert!(Q * R * perm ~ A_orig);
	}

	/// Test 1: A matrix that needs no pivoting.
	///
	/// Columns have geometrically decreasing norms (column j has norm 2^{-j}),
	/// so CPQR should discover the identity permutation.
	#[test]
	fn test_sketched_qr_no_pivoting_needed() {
		let rng = &mut StdRng::seed_from_u64(42);
		let n = 20;
		let m = 60;
		let bs = 8;
		let sketch_nrows = 40;
		let size = Ord::min(m, n);

		let mut A =
			build_scaled_matrix(m, n, bs, rng, |j| -(j as i32));
		let A_orig = A.as_ref().cloned();

		let col_perm = &mut *vec![0usize; n];
		let col_perm_inv = &mut *vec![0usize; n];
		let mut H = Mat::zeros(bs, size);
		let mut mem = MemBuffer::new(qr_in_place_scratch::<usize, c64>(
			m,
			n,
			sketch_nrows,
			bs,
			Par::Seq,
			default(),
		));

		let (_, perm) = qr_in_place(
			A.as_mut(),
			H.as_mut(),
			col_perm,
			col_perm_inv,
			sketch_nrows,
			rng,
			Par::Seq,
			MemStack::new(&mut mem),
			default(),
		);

		// Check identity permutation (no pivoting needed).
		let (fwd, _) = perm.arrays();
		for j in 0..n {
			std::assert_eq!(fwd[j].zx(), j, "expected identity perm at {j}");
		}

		verify_qrp(A.as_ref(), H.as_ref(), perm, A_orig.as_ref(), m, n, bs);
	}

	/// Test 2: A matrix that requires pivoting.
	///
	/// Columns have *increasing* norms (column j has norm 2^j), the opposite
	/// of what CPQR wants. We verify the sketched method recovers the same
	/// permutation as full CPQR.
	#[test]
	fn test_sketched_qr_pivoting_needed() {
		let rng = &mut StdRng::seed_from_u64(123);
		let n = 15;
		let m = 40;
		let bs = 8;
		let sketch_nrows = 30;
		let size = Ord::min(m, n);

		let mut A = build_scaled_matrix(m, n, bs, rng, |j| j as i32);
		let A_orig = A.as_ref().cloned();

		// Run full CPQR to get the reference permutation.
		let mut A_cpqr = A_orig.as_ref().cloned();
		let cpqr_col_perm = &mut *vec![0usize; n];
		let cpqr_col_perm_inv = &mut *vec![0usize; n];
		let mut H_cpqr = Mat::zeros(bs, size);
		let (_, cpqr_perm) =
			crate::linalg::qr::col_pivoting::factor::qr_in_place(
				A_cpqr.as_mut(),
				H_cpqr.as_mut(),
				cpqr_col_perm,
				cpqr_col_perm_inv,
				Par::Seq,
				MemStack::new(&mut MemBuffer::new(
					crate::linalg::qr::col_pivoting::factor::qr_in_place_scratch::<usize, c64>(
						m, n, bs, Par::Seq, default(),
					),
				)),
				default(),
			);

		let (cpqr_fwd, _) = cpqr_perm.arrays();
		let cpqr_fwd: Vec<usize> =
			cpqr_fwd.iter().map(|x| x.zx()).collect();

		// Run sketched QR.
		let col_perm = &mut *vec![0usize; n];
		let col_perm_inv = &mut *vec![0usize; n];
		let mut H = Mat::zeros(bs, size);
		let mut mem = MemBuffer::new(qr_in_place_scratch::<usize, c64>(
			m,
			n,
			sketch_nrows,
			bs,
			Par::Seq,
			default(),
		));

		let (_, perm) = qr_in_place(
			A.as_mut(),
			H.as_mut(),
			col_perm,
			col_perm_inv,
			sketch_nrows,
			rng,
			Par::Seq,
			MemStack::new(&mut mem),
			default(),
		);

		// Verify same permutation as full CPQR.
		let (sketched_fwd, _) = perm.arrays();
		for j in 0..n {
			std::assert_eq!(
				sketched_fwd[j].zx(),
				cpqr_fwd[j],
				"perm mismatch at {j}: sketched={} vs cpqr={}",
				sketched_fwd[j].zx(),
				cpqr_fwd[j],
			);
		}

		verify_qrp(A.as_ref(), H.as_ref(), perm, A_orig.as_ref(), m, n, bs);
	}
}
