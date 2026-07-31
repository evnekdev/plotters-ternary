# Interpolation model

`RegularTernaryScalarField` stores finite values at lattice coordinates
`i + j + k = n`, ordered row-major by `(i, j)`. It provides checked index,
lattice-coordinate, and composition conversions. The lattice has `n^2`
elementary triangles and `3n(n+1)/2` unique edges.

Each unique edge is directed from lower to higher canonical `GridVertexId` and
has one shared alpha interval. The three deterministic lattice-line families
use `spline1d` left/middle/right interval APIs. Two-sample boundary lines use
an explicit linear fallback by default and increment diagnostics.

A cubic triangle is the vertex-linear field plus its three pairwise terms. It
reproduces vertex values and source edge intervals exactly, is C0 across shared
edges, and does not promise C1 continuity.
