# Milestone 7 regular-grid contour kernel

## Scope and public API

Milestone 7 implements line contours over the regular two-dimensional ternary lattice only. For subdivision count `n`, `RegularTernaryScalarField` stores `(n+1)(n+2)/2` finite values at integer coordinates `i+j+k=n`. Canonical public ordering is row-major in `(i,j)`: `i=0..n`; for each `i`, `j=0..n-i`; and `k=n-i-j`. `GridVertexId`, `LatticeCoordinate`, `index_of`, `coordinate_of`, and `composition_at` provide checked conversions.

The public contour surface is:

```rust
RegularTernaryScalarField
ContourInterpolation::{Linear, CubicAlpha}
ContourOptions
CubicAlphaOptions
ContourSet { levels }
ContourLevel { value, paths }
ContourPath { points, closed }
TernaryContourSeries
```

The grid internally contains `n^2` elementary triangles and `3n(n+1)/2` unique edges. Arbitrary connectivity, irregular scattered data, Delaunay triangulation, Kuhn simplices, N-component grids, filled contours, and surfaces are deliberately absent.

## Feature policy

`cubic-alpha` gates cubic contour construction. The public option types remain available without the feature so configuration can be shared; requesting cubic computation then returns `ContourError::CubicFeatureUnavailable`. `spline1d` remains a normal dependency because the already-public `TernarySmoothSeries` also uses it. Thus making the dependency itself optional would be a breaking feature-policy change unrelated to this milestone.

## Linear contours

Linear mode is piecewise affine on every elementary triangle. A crossed edge uses

```text
t = (level - z0) / (z1 - z0)
```

with tolerance snapping at zero and one. Exact-level vertices are de-duplicated. A coincident edge has one deterministic owner based on the lowest triangle ID. A completely level-coincident triangle is reported as an ambiguous two-dimensional level region instead of producing an arbitrary line. Zero-length/tangent fragments are omitted. Segment endpoints are joined deterministically; degree greater than two, invalid loops, and non-manifold joins are errors.

## Directed alpha convention

Every unique grid edge is directed from its lower canonical `GridVertexId` to its higher ID. Each edge belongs to one of three deterministic lattice-line families. Lines with at least three samples use the appropriate `spline1d` left, middle, or right single-interval alpha API. A two-sample boundary line uses `CubicBoundaryPolicy::LinearFallback` by default or returns an error. Fallbacks are counted.

The normalized interval is:

```text
y(t) = y0*(1-t) + y1*t + (1-t)*t*(alpha0 + alpha1*t)
```

**`alpha1` multiplies the normalized coordinate `t` measured from the first directed edge endpoint toward the second.** Asymmetric numerical convention tests compare this form with `spline1d` direct coefficients for Akima, MAKIMA, PCHIP, and Steffen left/middle/right intervals.

Reversing an interval is centralized by `AlphaInterval::reversed`:

```text
(alpha0, alpha1) -> (alpha0 + alpha1, -alpha1)
```

Adjacent triangles reference the same canonical edge interval; they never independently fit oppositely directed coefficients.

## Binary extrapolation policies

For a directed pair `i -> j`, remaining component `k`, and `xi+xj+xk=1`, every policy retains the raw prefactor:

```text
Eij = xi*xj*(alpha0 + alpha1*tij)
```

Only `tij` changes:

```text
RawBarycentric: tij = xj
Muggianu:       tij = xj + xk/2
Kohler:         tij = xj/(xi+xj)
```

RawBarycentric extends the directed destination coordinate directly and is orientation-sensitive in the ternary interior. Canonical grid-edge direction therefore defines that model deterministically. It is not described as conventional Muggianu.

Muggianu symmetrically assigns half the remaining component fraction to each member of the binary pair. Kohler preserves their binary ratio by normalizing within `xi+xj`. Kohler returns exactly zero, with its finite limiting derivative, at the third vertex instead of evaluating `0/0`. Neither policy substitutes normalized `Xi*Xj` for the required raw `xi*xj` prefactor.

All three reduce exactly to the directed `spline1d` interval on the binary edge and to `alpha0*xi*xj` when `alpha1=0`. Ordinary alpha reversal preserves the complete Muggianu and Kohler interior contributions. It does not generally preserve RawBarycentric away from the binary edge; tests explicitly protect this distinction. No policy is claimed universally superior for every thermodynamic system.

## Local cubic-alpha field and gradients

One elementary triangle uses barycentric weights `x1,x2,x3`, vertex values `f1,f2,f3`, and one canonical directed interval for each pair:

```text
f = f1*x1 + f2*x2 + f3*x3 + E12 + E23 + E13
```

The centralized pair evaluator returns the value, chosen parameter, and unconstrained derivatives with respect to source, destination, and remaining barycentric coordinates. RawBarycentric, Muggianu, and Kohler have separate analytic parameter derivatives. Reduced derivatives use `x3=1-u-v`; global regular-grid projection transforms them analytically into semantic `(a,b)` coordinates. Centered finite-difference tests cover ordinary interior points, binary-edge neighborhoods, the third-vertex neighborhood, and local vertex permutation.

The field reproduces vertices and all three one-dimensional edge splines exactly, reduces to linear when all alpha values vanish, and reproduces the pairwise quadratic regular-solution form when all `alpha1` values vanish. It is C0 across grid edges because adjacent triangles share one edge interval. Cross-edge gradient continuity is not guaranteed.

## Cubic topology extraction

A cubic-alpha triangle may contain multiple crossings, tangencies, or closed curves. The implementation therefore does not classify it as one straight segment. It performs deterministic barycentric subdivision, evaluates the exact cubic-alpha field at subdivision vertices, edge midpoints, and cell centroids, and applies linear marching triangles only on leaf microtriangles.

Refinement considers sampled sign range and midpoint disagreement from linear edge values. Two initial refinement levels protect interior feature detection; subsequent refinement is adaptive up to `AdaptiveContourOptions::max_depth`. `maximum_depth_hits` reports leaf cells whose curvature remains above the configured flatness threshold. These cells are represented by the bounded microtriangle approximation rather than hidden. Stable endpoint cleanup and topology checks then assemble open and closed paths.

## Arc-length regularization and projection

Optional `ContourRegularization` computes chord length in the canonical equilateral ternary plane, discards provisional nonuniform vertices, redistributes open or periodic closed paths at uniform target arc positions, and projects each new interior point back to the requested implicit level. Open domain-boundary endpoints remain unchanged; closed paths do not duplicate their first point.

Projection uses the global piecewise field and its analytic semantic `(a,b)` gradient:

```text
delta = -F / dot(grad F, grad F) * grad F
```

The step is capped, backtracked until the residual decreases, validated in the simplex, and relocated into the containing elementary triangle after every accepted move. Crossing a small-triangle edge is therefore supported. Zero gradients, non-finite state, and non-convergence are explicit errors. Cubic regularization always projects onto the cubic policy-selected field, never onto the provisional linear microtriangles.

## Rendering adapter

`TernaryContourSeries::new(&contours)` supports one `ShapeStyle` or a level-style provider. It closes closed paths explicitly, then reuses `prepare_polyline` for semantic projection, mathematical viewport clipping, and visible-subpath splitting. Complete logical paths are computed before clipping. `TernaryChart::draw_series` returns Plotters' native `SeriesAnno`, so normal labels and legends remain intact.

PNG contour geometry participates in the existing geometry-only supersampling pass. Text remains final-resolution. SVG contours are ordinary vector polylines in the `ternary-geometry` group with no raster image; captions, axes, and legends remain native text.

## Examples and diagnostics

Permanent examples are:

```text
examples/output/{png,svg}/linear_contours.*
examples/output/{png,svg}/cubic_alpha_contours.*
examples/output/{png,svg}/cropped_contours.*
```

The coarse comparison shows linear, Akima+Muggianu, and regularized Steffen+Kohler paths from the same samples. The cropped example computes complete MAKIMA+Kohler paths first, then clips them to an invisible rectangular viewport. The nine-subdivision examples report 132 cubic edges, three two-sample linear fallback edges, zero projection failures, and explicit adaptive maximum-depth diagnostics.

## Known limitations

- Cubic topology is a bounded adaptive polyline approximation, not an analytic implicit-cubic solver.
- A max-depth diagnostic is not proof that topology is wrong; it records cells that reached the configured bound while still exceeding the flatness target.
- A completely flat level triangle is an ambiguous two-dimensional level set and returns an error.
- C0 edge continuity is guaranteed; C1 continuity is not.
- RawBarycentric depends on canonical direction in the ternary interior.
- Exact cubic edge-root acceleration and contour labels are deferred.
- Irregular triangulation, Kuhn simplices, filled contours, and N-component interpolation are outside this milestone.
