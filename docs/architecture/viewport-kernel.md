# Milestone 2 viewport and clipping kernel

## Public types

Milestone 2 extends the backend-independent `coord` kernel with:

- `TernaryViewport` and `ViewportPointLocation`;
- `PixelRect`, `PixelPoint`, and `PixelBounds`;
- `ViewportFit`, `ViewportAlignment`, and `ViewportTransform`;
- `CartesianSegment`, `ClippedSegment`, `clip_segment`, and
  `clip_segment_with_parameters`;
- `TriangleEdge` and `VisibleTriangleEdge`;
- full, visible, and component-isoline helpers on `TernaryGeometry`.

All are re-exported from the crate root. None depends on Plotters, and none draws
or exposes the viewport rectangle.

## Logical viewport

`TernaryViewport::new(x_min, x_max, y_min, y_max)` validates with
`Tolerance::default()`. `new_with_tolerance` permits an explicit minimum span.
All bounds must be finite; both ranges must be ordered and have a span strictly
greater than the absolute tolerance. A viewport may be partly or wholly
outside the Gibbs triangle. `TernaryViewport::full(geometry)` is the tight
axis-aligned rectangle around that geometry's complete triangle, including the
negative Y range of downward geometry.

`viewport.classify(point, tolerance)` returns `Inside`, `Boundary`, or
`Outside`. `contains` is the corresponding boundary-inclusive boolean helper.
These results intentionally do not include triangle containment or composition
validity.

## Pixel coordinates and fitting

`PixelRect { x, y, width, height }` uses a top-left origin. Pixel X increases to
the right and pixel Y increases downward. Width and height are continuous
coordinate extents: a rectangle at `(10, 20)` with size `200 x 100` has
floating bounds `10..210` and `20..120`. Zero dimensions are invalid.

`ViewportTransform` retains the logical viewport, allocated integer rectangle,
and floating fitted bounds. `PreserveAspect` uses one X/Y scale and is the
default. `Stretch` independently fills both allocated dimensions. When
preserving aspect leaves space, `ViewportAlignment` applies horizontal and
vertical factors of zero, one half, or one for the nine named positions.

`logical_to_pixel` inverts logical Y so `y_max` maps to the fitted top.
`pixel_to_logical` is affine and accepts pixels outside the fitted rectangle;
`pixel_to_logical_checked` returns `None` outside. Integer rounding is deferred
to rendering integration.

## Segment clipping

`clip_segment` uses directed Liang-Barsky parameter clipping against the four
viewport sides. It returns `None` for no intersection and otherwise preserves
the source direction. A fully contained segment is returned verbatim. A
zero-length segment survives exactly when its point is viewport-contained.
Boundary-coincident segments and corner tangencies remain valid; materially
external parallel segments do not collapse onto a viewport boundary.

The calculation clips to exact viewport bounds. Tolerance is used only to
classify near-boundary parallel constraints and numerically coincident entering
and leaving parameters. `clip_segment_with_parameters` additionally returns the
retained source interval, normally within `[0, 1]`.

## Triangle edges

`TriangleEdge::{LeftRight, RightApex, ApexLeft}` names geometric slots rather
than semantic components, so identity is stable across all vertex orders and
both orientations. Each edge is directed in boundary order.

`geometry.visible_edges(viewport, tolerance)` returns zero to three
`VisibleTriangleEdge` values. Every fragment retains the edge identity, clipped
segment, and `parameter_start..parameter_end` on the original directed edge.
This is sufficient for Milestone 5 to test whether a tick lies on a visible
original edge.

## Component isolines

`geometry.component_isoline(component, value, tolerance)` builds the finite
full-triangle segment by interpolating from both other component vertices
toward the selected component vertex. At zero it is the opposite edge; at one
it degenerates to the selected vertex. Values within tolerance of zero or one
are snapped to the boundary, while materially external and non-finite values
return `Error::InvalidIsolineValue`.

The construction is semantic A/B/C geometry and therefore works under all six
vertex orders and both orientations. `visible_component_isoline` applies the
same generic segment clipper without generating tick sequences.

## Differences from the provisional inventory

The implemented viewport stores four private scalar bounds rather than public
`Range<f64>` fields. Construction is `new`/`new_with_tolerance`, and
`full` takes a `TernaryGeometry`. Floating pixel types and the checked reverse
mapping are explicit. The combined provisional `PointStatus` is not introduced;
triangle and viewport classification remain separate.

## Carry-forward to Milestone 3

Milestone 3 must allocate a Plotters chart rectangle, build a
`ViewportTransform`, clip complete triangle edges and component isolines in
logical space, and only then map the surviving geometry to Cartesian/pixel
coordinates. The viewport rectangle remains invisible. Chart code must decide
how unused fitted pixel space participates in layout and must defer integer
rounding until Plotters elements are produced.
