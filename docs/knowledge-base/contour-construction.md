# Contour construction

Linear contours are conventional piecewise-affine marching triangles with
explicit handling for level vertices, coincident edges, and fully level
triangles. Paths are joined deterministically; non-manifold degree greater than
two is an error.

Cubic-alpha contours evaluate the exact local field on an adaptively subdivided
barycentric microtriangle mesh. Two initial subdivision levels guard against
interior features. Refinement is bounded by `AdaptiveContourOptions`; cells
that still exceed the flatness target at the limit are retained in the
approximation and counted in `maximum_depth_hits` diagnostics.

Complete semantic paths are constructed before chart projection and invisible
rectangular viewport clipping. Rendering reuses ordinary line-series clipping
and native Plotters legends.
