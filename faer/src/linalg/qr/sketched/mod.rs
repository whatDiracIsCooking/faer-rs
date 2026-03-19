//! Randomized rank-revealing $QR$ decomposition via Gaussian sketching.
//!
//! Computes $AP^T = QR$ by first sketching $A$ with a Gaussian random matrix
//! to determine the column permutation, and then performing unpivoted $QR$ on
//! the permuted matrix.
//!
//! The sketch matrix $S \in \mathbb{R}^{s \times m}$ has i.i.d. $\mathcal{N}(0,1)$
//! entries. The product $B = SA$ preserves the number of columns and is used to
//! determine column ordering via column-pivoting $QR$ on $B$.

pub mod factor;
