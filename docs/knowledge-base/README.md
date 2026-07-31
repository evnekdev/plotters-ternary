# Ternary cubic-alpha contour knowledge base

This directory is the editable source for the contour-method knowledge base that
previously existed only as a ZIP archive. The binary archive is intentionally
not version-controlled or published in the crate package; a release workflow
may create one from these Markdown sources when needed.

Start with [formulas](formulas.md), then the
[interpolation model](interpolation-model.md),
[contour construction](contour-construction.md), and
[regularization](regularization.md). The original detailed bundle is preserved
under [`source/`](source/), with encoding corrections where needed.

The implemented scope is a regular two-dimensional ternary lattice. Irregular
triangulation, Kuhn simplices, filled contours, and N-component fields remain
explicitly out of scope for `plotters-ternary` 0.1.0.

## Implementation ownership

The editable numerical method is implemented in `ternary-contours`. That crate
owns topology extraction, path assembly, regularization, and level projection.
`plotters-ternary` consumes final semantic paths and performs only chart
projection, display clipping, styling, legends, and backend rendering.
