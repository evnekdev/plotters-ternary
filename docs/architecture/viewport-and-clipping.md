# Rectangular viewports and clipping

## Requirement

A chart may display only a rectangular portion of a much larger projected Gibbs triangle. Typical cases include:

- the right or left side of a triangle;
- a corner with the opposite edges trimmed;
- a top-trimmed triangle;
- an interior zoom containing no original corner or triangle edge.

The rectangular display window must normally remain invisible. Space outside it remains available to ordinary Plotters layout for captions, subtitles, legends and margins.

## Separate coordinate spaces

The implementation must distinguish:

1. **Ternary domain:** compositions `(a, b, c)` under a selected sum and normalisation policy.
2. **Projected plane:** the two-dimensional plane containing the complete Gibbs triangle.
3. **Logical viewport:** a rectangle in projected-plane coordinates.
4. **Pixel viewport:** the aspect-fitted rectangle allocated inside a Plotters drawing area.
5. **Outer layout:** captions, legends and other Plotters content outside the clipped ternary region.

A full triangle is simply the default logical viewport.

## Viewport type

```rust
pub struct TernaryViewport { /* private x_min/x_max/y_min/y_max */ }
```

Implemented constructors:

```rust
TernaryViewport::new(x_min, x_max, y_min, y_max)
TernaryViewport::new_with_tolerance(x_min, x_max, y_min, y_max, tolerance)
TernaryViewport::full(geometry)
```

The raw scalar constructor provides exact control. Composition-oriented convenience constructors remain deferred until their padding and degenerate-input policies are required.

## Mapping and aspect ratio

The default must preserve equal x/y scale so that the Gibbs triangle is not distorted.

```rust
pub enum ViewportFit {
    PreserveAspect,
    Stretch,
}
```

`PreserveAspect` may leave unused pixels in the allocated chart rectangle. Alignment should be configurable. `Stretch` is available only as an explicit opt-in.

`PixelRect` uses a top-left origin with pixel Y increasing downward. `ViewportTransform` stores floating fitted bounds, applies one of nine alignments to unused space, and reversibly maps logical and floating pixel coordinates. Integer rounding is deferred to rendering.

## Fundamental rendering rule

All primitives are defined in the full ternary domain. They are not pre-filtered by point visibility. The correct pipeline is:

```text
project complete primitive
    -> intersect with full Gibbs triangle where required
    -> clip against rectangular viewport
    -> map visible result to pixels
    -> draw through Plotters
```

Discarding off-screen vertices before clipping is incorrect because an off-screen segment may cross the visible window.

## Clipping algorithms

### Segments and polylines

Use a standard rectangle-segment clipping algorithm such as Liang-Barsky. For a polyline:

1. Project all source vertices.
2. Clip each consecutive segment.
3. Join adjacent surviving segments into visible subpaths.
4. Draw each subpath independently.

### Polygons

Polygon clipping is deferred to Milestone 6. Sutherland-Hodgman against the four sides remains the intended algorithm because the clipping window is convex.

### Grid lines

Generate a finite component isoline across the full triangle, then clip it against the viewport. For component A at value `k`, the full segment joins:

```rust
TernaryPoint::new(k, 1.0 - k, 0.0)
TernaryPoint::new(k, 0.0, 1.0 - k)
```

The same construction applies cyclically to B and C.

### Markers

Marker clipping needs an explicit policy:

```rust
pub enum MarkerClipMode {
    Centre,
    Bounds,
    None,
}
```

`Centre` is a practical initial implementation. `Bounds` is the publication-quality target but may require drawing into a clipped sub-area or clipping the marker geometry itself.

### Text

Text should have a similar policy because annotations may intentionally extend beyond the viewport. Suggested modes:

```rust
pub enum AnnotationClipMode {
    Anchor,
    Bounds,
    None,
}
```

## Triangle edges and ticks under cropping

The original triangle edges remain geometric entities even when partly or completely invisible.

```rust
pub enum TriangleEdge {
    LeftRight,
    RightApex,
    ApexLeft,
}
```

Edges use geometric slot names so their identities remain stable across component vertex orders. For each directed edge, calculate its intersection with the viewport. `VisibleTriangleEdge` retains the original edge identity and the clipped parameter range in `[0, 1]`.

Ticks are generated in composition space and drawn only when their projected positions lie on a visible original triangle-edge fragment. The viewport boundary itself is not automatically drawn and is not automatically treated as an axis.

## Tick range modes

```rust
pub enum TickRangeMode {
    FullCompositionRange,
    VisibleRange,
}
```

- `FullCompositionRange` preserves a global sequence such as 0.0, 0.1, ..., 1.0 and filters invisible ticks.
- `VisibleRange` resolves ticks over the component interval visible inside the viewport and is useful for close zooms.

The global mode should be the conservative default because cropping should not silently change requested tick semantics.

## Cropped axis policies

```rust
pub enum CroppedAxisPolicy {
    TriangleEdgesOnly,
    RelocateMissingAxes,
    Manual,
}
```

### TriangleEdgesOnly

Ticks, tick labels and axis names are drawn only where original triangle edges remain visible. This is the default and most geometrically honest behaviour.

### RelocateMissingAxes

Missing labels may be anchored to selected viewport sides while the viewport frame remains invisible. This is useful for interior zooms but requires careful automatic placement and should follow the initial implementation.

### Manual

The user selects visibility and placement for every axis and corner label.

## Corner labels

Corner labels and axis names are separate concepts. A corner label should normally be hidden when its projected vertex is outside the viewport.

Suggested policy:

```rust
pub enum CornerLabelVisibility {
    VisibleCornersOnly,
    HiddenWhenCropped,
    Always,
    Custom,
}
```

Manual placement should allow a missing corner name to be pinned to a viewport side or projected coordinate without drawing a rectangular frame.

## Point classification

```rust
pub enum ViewportPointLocation {
    Inside,
    Boundary,
    Outside,
}
```

Viewport classification is deliberately separate from `TrianglePointLocation` and composition validation. A later chart may combine those results explicitly, but the geometry kernel does not introduce a conflated `Visible` status.

## Testing requirements

Viewport tests should include:

- complete triangle;
- left and right crops;
- top-trimmed and bottom-trimmed views;
- a crop containing exactly one corner;
- a crop containing no corners but intersecting edges;
- a fully interior crop containing no edges;
- lines entering and leaving through every viewport side;
- polygons surrounding the viewport;
- aspect-preserving mapping into wide, square and tall pixel areas;
- reverse transformation near viewport boundaries.

## Milestone 3 rendering integration

Rendering uses the fitted-subarea strategy. After Plotters allocates the caption
and the chart applies its outer margin, `ViewportTransform` calculates the
fitted bounds. Those floating bounds are rounded once to an integer
`DrawingArea::shrink` subarea, and the Cartesian chart uses the requested
viewport ranges exactly. `PreserveAspect` therefore keeps equal logical X/Y
scale to within one pixel of layout rounding; `ViewportAlignment` places unused
space around that subarea. `Stretch` uses the full post-layout allocation.

The requested viewport remains both the mathematical clipping rectangle and
the internal Cartesian logical range. It is never rendered. Triangle edges and
component isolines are clipped with `visible_edges` and
`visible_component_isoline` before becoming Plotters paths; the plotting-area
boundary is not used as a substitute for mathematical clipping.
