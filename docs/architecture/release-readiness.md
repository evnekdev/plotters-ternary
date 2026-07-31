# Milestone 8 release-readiness audit

## Scope

This audit stabilizes the 0.1.0 API and release process; it deliberately adds
no rendering or numerical feature. The verified numerical scope remains regular
ternary-grid line contours only.

## Contour audit result

The implementation matches `contour-kernel.md`:

- values are row-major by `(i,j)` for `i+j+k=n`;
- the grid has `n^2` elementary triangles and `3n(n+1)/2` directed unique edges;
- every edge direction is lower to higher `GridVertexId`;
- alpha intervals use `y0*(1-t)+y1*t+(1-t)*t*(alpha0+alpha1*t)` and reverse as
  `(alpha0+alpha1, -alpha1)`;
- Muggianu uses `xj+xk/2`, Kohler uses `xj/(xi+xj)`, and both retain raw `xi*xj`;
- shared edge values are C0, not C1;
- adaptive extraction terminates at a bounded depth and reports depth hits;
- path joining, endpoint preservation, periodic closed paths, cell-crossing
  projection, and viewport clipping have focused tests.

Linear-plane tests assert exact residuals. Nonlinear closed-loop and saddle
benchmarks assert path class, finite points, deterministic output, and bounded
approximation residuals; this is evidence of robustness, not a formal
convergence-order claim.

## RawBarycentric decision

[ADR 0003](../decisions/0003-raw-barycentric-experimental.md) retains
`RawBarycentric` as explicitly experimental and non-recommended. It is neither
linear interpolation nor conventional Muggianu. Stable code should select the
default Muggianu policy or Kohler.

## Packaging and features

`0.1.0` has MSRV Rust 1.89, uses edition 2024, and is dual licensed as
`MIT OR Apache-2.0`. `cubic-alpha` enables cubic contour computation. It does
not remove `spline1d` when disabled because smooth series already depend on it.
The generated example-output directory and legacy ZIP bundle are intentionally
excluded from the crate package; editable knowledge-base Markdown sources stay
in `docs/knowledge-base/`.

## CI

GitHub Actions checks formatting, clippy, default/all/no-default tests, rustdoc,
and `cargo package` on stable Linux; all-feature tests run on Linux, Windows,
and macOS; an MSRV 1.89 job runs `cargo check --all-targets` and `cargo test`.

## Known release limits

No filled contours, contour labels, irregular/scattered data, arbitrary meshes,
Kuhn simplices, N-component grids, or C1 guarantees are claimed. Cubic topology
is a bounded adaptive polyline approximation and diagnostics must be reviewed
when maximum-depth hits are nonzero.
## Phase 2 extraction publication blocker

During two-repository development, `ternary-contours` is intentionally a local
path dependency. `cargo package --list` remains useful, but full
`plotters-ternary` package verification cannot resolve that unpublished crate
from crates.io. Publish and verify `ternary-contours` first, then replace the
path dependency with the released version before restoring full package
verification in CI. Neither crate is published or tagged by the extraction
work.