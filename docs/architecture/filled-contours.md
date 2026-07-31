# Milestone 10: linear filled contours and scalar maps

ternary-contours owns ContourBandSet numerical regions and their
non-overlapping simple fragments. plotters-ternary projects immutable semantic
coordinates through TernaryGeometry, clips only for the invisible rectangular
viewport, and draws native vector polygons. It never resamples, smooths, or
changes a ContourBandSet.

Breaks are finite and strictly increasing. Scalar ownership is f < l0,
li <= f < li+1, and f >= lm. Adjacent band polygons may share the same
zero-area threshold boundary. This does not create a positive-area overlap.

The renderer fills core fragments, rather than filling a ContourRegion exterior
ring alone. Consequently holes are transparent cut-outs: they reveal the layer
below, including a scalar map, instead of a painted chart background. By
default bands have no borders. OuterRegions borders stroke each exterior and
hole boundary once; Fragments borders are an explicit diagnostic mode that can
show internal seams. Isolines are the preferred inter-band boundary style.

TernaryScalarMapSeries evaluates the exact piecewise-linear field at each
microtriangle centroid and flat-fills that primitive. It is therefore a
deterministic approximation of continuous colour shading, not a backend
Gouraud gradient. Fixed resolution emits n squared microtriangles per
elementary field triangle. Adaptive currently means bounded deterministic
uniform refinement from max_depth. Higher resolution reduces visible faceting
but increases SVG primitive count and file size. Automatic constant fields use
the colour map midpoint; explicit degenerate ranges are rejected.

Colours, opacity, scalar normalisation, microtriangle resolution, viewport
clipping, legends, and labels belong to plotters-ternary.
