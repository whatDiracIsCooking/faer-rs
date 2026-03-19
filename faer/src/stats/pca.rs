use crate::internal_prelude::*;
use crate::linalg::solvers::{Svd, SvdError};
use crate::stats::{NanHandling, row_mean};

/// Principal Component Analysis (PCA) decomposition.
///
/// Given a data matrix $A$ of shape $(n, p)$ where $n$ is the number of
/// observations and $p$ is the number of features, PCA computes the principal
/// components via the thin SVD of the mean-centered data.
///
/// The data convention is **rows = observations, columns = features**.
#[derive(Clone, Debug)]
pub struct Pca<T> {
	/// Principal component directions (columns of V from SVD), shape (p, k)
	components: Mat<T>,
	/// Singular values of the centered data matrix, length k
	singular_values: Col<T>,
	/// Feature means used for centering, length p (stored as a Row)
	mean: Row<T>,
	/// Number of observations (n)
	n_samples: usize,
}

impl<T: ComplexField> Pca<T> {
	/// Computes PCA on the data matrix $A$ with shape $(n, p)$.
	///
	/// Rows are observations and columns are features.
	/// If `n_components` is `None`, all `min(n, p)` components are kept.
	///
	/// # Errors
	/// Returns `SvdError` if the underlying SVD fails to converge.
	#[track_caller]
	pub fn new<C: Conjugate<Canonical = T>>(
		A: MatRef<'_, C>,
		n_components: Option<usize>,
	) -> Result<Self, SvdError> {
		Self::new_imp(A.canonical(), Conj::get::<C>(), n_components)
	}

	#[track_caller]
	fn new_imp(
		A: MatRef<'_, T>,
		conj: Conj,
		n_components: Option<usize>,
	) -> Result<Self, SvdError> {
		let (n, p) = A.shape();
		let k_max = Ord::min(n, p);
		let k = match n_components {
			Some(k) => {
				core::assert!(
					k <= k_max,
					"n_components ({k}) exceeds min(n, p) ({k_max})"
				);
				k
			},
			None => k_max,
		};

		// Compute feature means (mean of each column across rows)
		let mut mean = Row::<T>::zeros(p);
		row_mean(mean.as_mut(), A, NanHandling::Propagate);

		// Build mean-centered matrix
		let mut centered = Mat::<T>::zeros(n, p);
		for j in 0..p {
			for i in 0..n {
				centered[(i, j)] = A[(i, j)].copy().sub(&mean[j]);
			}
		}

		// Compute thin SVD of centered data
		let svd = if conj == Conj::Yes {
			Svd::new_thin(centered.conjugate())?
		} else {
			Svd::new_thin(centered.as_ref())?
		};

		// Extract first k components
		let V = svd.V();
		let s_full = svd.S().column_vector();

		let mut components = Mat::<T>::zeros(p, k);
		let mut singular_values = Col::<T>::zeros(k);

		for j in 0..k {
			for i in 0..p {
				components[(i, j)] = V[(i, j)].copy();
			}
			singular_values[j] = s_full[j].copy();
		}

		Ok(Self {
			components,
			singular_values,
			mean,
			n_samples: n,
		})
	}

	/// Returns the principal component directions as columns.
	///
	/// Shape: $(p, k)$ where $p$ is the number of features and $k$ is the
	/// number of components.
	pub fn components(&self) -> MatRef<'_, T> {
		self.components.as_ref()
	}

	/// Returns the singular values of the centered data matrix.
	///
	/// Length: $k$ (the number of components).
	pub fn singular_values(&self) -> ColRef<'_, T> {
		self.singular_values.as_ref()
	}

	/// Returns the feature means that were used for centering.
	///
	/// Length: $p$ (the number of features).
	pub fn mean(&self) -> RowRef<'_, T> {
		self.mean.as_ref()
	}

	/// Returns the explained variance for each component.
	///
	/// Computed as $\sigma_i^2 / (n - 1)$ where $\sigma_i$ are the singular
	/// values and $n$ is the number of observations.
	///
	/// Length: $k$ (the number of components).
	pub fn explained_variance(&self) -> Col<T::Real> {
		let n = self.n_samples;
		let denom = if n > 1 {
			from_f64::<T::Real>((n - 1) as f64)
		} else {
			from_f64::<T::Real>(1.0)
		};
		Col::from_fn(self.singular_values.nrows(), |i| {
			self.singular_values[i].abs2() / denom.copy()
		})
	}

	/// Returns the ratio of variance explained by each component.
	///
	/// Each entry is the explained variance of that component divided by the
	/// total explained variance. Values sum to 1.0 (when all components are
	/// kept).
	///
	/// Length: $k$ (the number of components).
	pub fn explained_variance_ratio(&self) -> Col<T::Real> {
		let var = self.explained_variance();
		let total: T::Real =
			var.iter().fold(zero::<T::Real>(), |acc, v| acc + v.copy());
		let total_inv = total.recip();
		Col::from_fn(var.nrows(), |i| var[i].copy() * total_inv.copy())
	}

	/// Projects data onto the principal components.
	///
	/// Given data $X$ of shape $(m, p)$, returns the centered data projected
	/// onto the components, of shape $(m, k)$.
	pub fn transform<C: Conjugate<Canonical = T>>(
		&self,
		X: MatRef<'_, C>,
	) -> Mat<T> {
		self.transform_imp(X.canonical(), Conj::get::<C>())
	}

	fn transform_imp(&self, X: MatRef<'_, T>, conj: Conj) -> Mat<T> {
		let (m, p) = X.shape();
		core::assert!(
			p == self.components.nrows(),
			"input has {p} features, expected {}",
			self.components.nrows()
		);
		// Center the data
		let mut centered = Mat::<T>::zeros(m, p);
		for j in 0..p {
			for i in 0..m {
				let val = if conj == Conj::Yes {
					X[(i, j)].conj()
				} else {
					X[(i, j)].copy()
				};
				centered[(i, j)] = val.sub(&self.mean[j]);
			}
		}
		// Project: centered * components
		&centered * &self.components
	}

	/// Reconstructs data from the reduced representation.
	///
	/// Given scores $Z$ of shape $(m, k)$, returns the reconstruction
	/// $Z \cdot V^H + \mu$ of shape $(m, p)$.
	pub fn inverse_transform<C: Conjugate<Canonical = T>>(
		&self,
		Z: MatRef<'_, C>,
	) -> Mat<T> {
		self.inverse_transform_imp(Z.canonical(), Conj::get::<C>())
	}

	fn inverse_transform_imp(
		&self,
		Z: MatRef<'_, T>,
		conj: Conj,
	) -> Mat<T> {
		let (m, k) = Z.shape();
		let p = self.components.nrows();
		core::assert!(
			k == self.components.ncols(),
			"input has {k} components, expected {}",
			self.components.ncols()
		);
		let Z = if conj == Conj::Yes {
			let mut z = Mat::<T>::zeros(m, k);
			for j in 0..k {
				for i in 0..m {
					z[(i, j)] = Z[(i, j)].conj();
				}
			}
			z
		} else {
			let mut z = Mat::<T>::zeros(m, k);
			z.copy_from(Z);
			z
		};
		// Reconstruct: Z * V^H + mean
		let mut result = &Z * self.components.adjoint();
		for j in 0..p {
			for i in 0..m {
				result[(i, j)] = result[(i, j)].copy().add(&self.mean[j]);
			}
		}
		result
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assert;
	use crate::utils::approx::*;

	#[test]
	fn test_pca_basic_f64() {
		// 4 observations, 3 features
		#[rustfmt::skip]
		let A = crate::mat![
			[1.0, 2.0, 3.0],
			[4.0, 5.0, 6.0],
			[7.0, 8.0, 9.0],
			[10.0, 11.0, 12.0_f64],
		];

		let pca = Pca::new(A.as_ref(), None).unwrap();

		// Check mean is correct: [5.5, 6.5, 7.5]
		let mean = pca.mean();
		equator::assert!((mean[0] - 5.5).abs() < 1e-10);
		equator::assert!((mean[1] - 6.5).abs() < 1e-10);
		equator::assert!((mean[2] - 7.5).abs() < 1e-10);

		// Components should be orthonormal columns
		let V = pca.components();
		let VtV = V.adjoint() * V;
		let approx_eq = CwiseMat(ApproxEq::eps() * 64.0);
		let eye = Mat::<f64>::identity(VtV.nrows(), VtV.ncols());
		assert!(VtV ~ eye);

		// Explained variance ratios should sum to 1
		let ratios = pca.explained_variance_ratio();
		let sum: f64 = ratios.iter().copied().sum();
		equator::assert!((sum - 1.0).abs() < 1e-10);
	}

	#[test]
	fn test_pca_n_components() {
		// 5 observations, 3 features
		#[rustfmt::skip]
		let A = crate::mat![
			[2.5, 2.4, 1.0],
			[0.5, 0.7, 0.3],
			[2.2, 2.9, 1.1],
			[1.9, 2.2, 0.9],
			[3.1, 3.0, 1.5_f64],
		];

		let pca = Pca::new(A.as_ref(), Some(2)).unwrap();
		assert!(pca.components().ncols() == 2);
		assert!(pca.singular_values().nrows() == 2);
		assert!(pca.explained_variance().nrows() == 2);
	}

	#[test]
	fn test_pca_transform_inverse_roundtrip() {
		// With all components, transform + inverse_transform should recover
		// the original data
		#[rustfmt::skip]
		let A = crate::mat![
			[2.5, 2.4, 1.0],
			[0.5, 0.7, 0.3],
			[2.2, 2.9, 1.1],
			[1.9, 2.2, 0.9],
			[3.1, 3.0, 1.5_f64],
		];

		let pca = Pca::new(A.as_ref(), None).unwrap();
		let scores = pca.transform(A.as_ref());
		let reconstructed = pca.inverse_transform(scores.as_ref());

		let approx_eq = CwiseMat(ApproxEq::eps() * 256.0);
		assert!(reconstructed ~ A);
	}

	#[test]
	fn test_pca_via_matref_convenience() {
		#[rustfmt::skip]
		let A = crate::mat![
			[1.0, 2.0],
			[3.0, 4.0],
			[5.0, 6.0_f64],
		];
		let pca = A.as_ref().pca(Some(1)).unwrap();
		assert!(pca.components().ncols() == 1);
	}

	#[test]
	fn test_pca_singular_values_nonnegative_and_sorted() {
		#[rustfmt::skip]
		let A = crate::mat![
			[2.5, 2.4, 1.0],
			[0.5, 0.7, 0.3],
			[2.2, 2.9, 1.1],
			[1.9, 2.2, 0.9],
			[3.1, 3.0, 1.5_f64],
		];

		let pca = Pca::new(A.as_ref(), None).unwrap();
		let sv = pca.singular_values();
		for i in 0..sv.nrows() {
			equator::assert!(sv[i].real() >= 0.0);
			if i > 0 {
				equator::assert!(sv[i - 1].real() >= sv[i].real());
			}
		}
	}
}
