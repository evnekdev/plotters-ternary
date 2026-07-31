# Regularization and level projection

Optional regularization redistributes a provisional path at approximately equal
chord-length spacing in the canonical equilateral plane, then projects each
interior point back onto `f = level` with a damped normal/Newton correction.

Open endpoints are preserved. Closed paths are periodic and do not duplicate
the first point. Each accepted projection step is constrained to the simplex,
backtracked until it reduces residual, and re-located in the global piecewise
field; it may cross elementary-triangle boundaries. Zero gradients and
non-convergence are explicit errors.
