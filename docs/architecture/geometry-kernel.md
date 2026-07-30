# Milestone 1 geometry kernel

## Public kernel

Milestone 1 introduces a backend-independent `coord` module, with its stable
kernel types re-exported from the crate root:

- `Component::{A, B, C}` has stable A/B/C array indexes.
- `TernaryPoint` stores private `f64` components and provides `new`,
  `component`, `sum`, `as_array`, and array/tuple conversions.
- `Normalization::{RequireUnitSum, Normalize, RequireSum(f64)}` selects an
  explicit validation policy.
- `Tolerance { absolute, relative }` and the focused crate `Error` type cover
  numerical validation and full-triangle inverse projection.
- `TriangleOrientation::{Up, Down}`, validated `VertexOrder`,
  `TernaryCartesian`, `TernaryGeometry`, and `TrianglePointLocation` define
  projection geometry without depending on Plotters.

`TernaryPoint::new` never normalises. Users validate explicitly with
`point.validate(policy, tolerance)`, and `TernaryGeometry::project` validates
with the same explicit inputs.

## Validation and tolerance

`Tolerance::default()` is absolute and relative `1e-12`. Both terms must be
finite and strictly positive. Comparisons use
`absolute + relative * max(|left|, |right|)`; near-zero checks use the
absolute term.

Every component must be finite. A component below `-absolute` is rejected.
A finite value in `[-absolute, 0.0)` is deterministically converted to exactly
zero before calculating the sum. All policies reject a finite sum less than or
equal to `absolute`; `Normalize` then divides by that non-zero sum, while the
two `Require*` policies compare the cleaned sum to their target.

## Canonical geometry and projection

The base slots are always left `(0, 0)` and right `(1, 0)`. The orientation
apex is `(0.5, sqrt(3)/2)` for `Up` and `(0.5, -sqrt(3)/2)` for `Down`, where
`sqrt(3)/2` is represented by the single `EQUILATERAL_TRIANGLE_HEIGHT`
constant. The `apex` slot replaces the provisional `top` name because it stays
unambiguous under downward orientation.

`VertexOrder` maps semantic components to the left/right/apex slots and can
contain each component only once. Projection first produces unit weights, then
calculates the barycentric sum `a * vertex(A) + b * vertex(B) + c * vertex(C)`.
Component semantics remain A/B/C regardless of slot placement.

Reverse projection solves the two-dimensional affine barycentric system using
2D cross products; it does not require a linear-algebra dependency. Weights are
mapped from slots back into A/B/C order. Points with a weight below
`-absolute` fail as outside the full triangle. For a point classified on the
boundary, weights within the absolute tolerance of zero are set to zero and
the remaining weights are rescaled to unit sum. Thus round-off immediately
outside an edge is admitted only inside tolerance; materially external points
are never silently clamped.

## Difference from the provisional inventory

The inventory now reflects the Milestone 1 decisions: `TernaryPoint` and
`TernaryGeometry` fields are private, `VertexOrder` exposes validated
left/right/apex accessors, and projection takes an explicit normalisation
policy and tolerance. Edge, isoline, viewport, pixel-mapping, and rendering
operations remain intentionally absent.

## Carry-forward to Milestone 2

Milestone 2 must add a separate rectangular `TernaryViewport`, aspect-aware
logical-to-pixel fitting, and mathematical segment/polygon clipping. Its
visibility statuses must remain distinct from `TrianglePointLocation`, which
only concerns the complete Gibbs triangle. Later Plotters adapters must keep
this kernel free of backend and drawing-area types, in line with ADR 0001 and
the Milestone 0 clipping finding.
