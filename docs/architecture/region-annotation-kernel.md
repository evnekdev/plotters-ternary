# Milestone 6 region and annotation kernel

## Public API

Milestone 6 adds `TernaryPolygon<I>` and `TernaryText` as ordinary
`TernarySeries` implementations. They are submitted through the existing
`TernaryChart::draw_series` method and return Plotters' native mutable
`SeriesAnno`; phase regions therefore use the same `.label(...).legend(...)`
and `configure_series_labels()` workflow as line and point series.

`TernaryPolygon::new(points)` accepts compositions convertible to
`TernaryPoint`. Its independent optional styles are selected with
`.fill_style(...)` and `.border_style(...)`. No fill and/or no border is valid.
Input loops may be open or explicitly closed; a repeated final first vertex is
removed before validation.

`TernaryText::new(point, text)` owns its composition, string, and
`AxisTextStyle`. It offers `TextAnchor` with `HorizontalAnchor` and
`VerticalAnchor`, a final-output-pixel `.offset((x, y))`, explicit
normalisation/tolerance, native `TextRotation::{None, Rotate90, Rotate180,
Rotate270}`, and `AnnotationClipMode::{Anchor, None}`. `Anchor` is the default
and only submits text whose projected anchor is in or on the logical viewport.
`None` is an intentional escape hatch. Portable measured text-bounds clipping,
callouts, backgrounds, and general arbitrary-angle annotation rotation remain
deferred. `VerticalAnchor::Baseline` currently maps to Plotters' portable
bottom anchor because Plotters exposes no backend-neutral baseline metric.

## Polygon preparation and clipping

`prepare_polygon` is the backend-independent kernel. It validates and
normalises every source composition while retaining the original source index
in `PolygonError::InvalidPoint`, projects the cleaned loop, rejects fewer than
three distinct vertices, checks signed area, and rejects intersections between
non-adjacent edges. Simple convex and concave subjects are supported.
Self-intersecting subjects are rejected rather than delegated to backend fill
rules.

The final loop is clipped against the four logical `TernaryViewport` sides
with Sutherland–Hodgman clipping. The clipping window is convex, so clipping a
simple subject produces one simple output loop or no positive-area polygon.
Intersections are calculated before outside vertices are discarded; a polygon
that encloses the viewport can therefore remain visible even when none of its
source vertices is visible. Adjacent near-duplicates are removed after every
clip pass. The invisible viewport is never emitted as a border.

The chart converts each prepared loop into one owned Plotters element. Draw
order for one region is always:

```text
fill -> one closed outer border
```

The border is not repeated per clipping edge or per fill fragment. Callers'
`draw_series` order remains the region layer order.

## Rendering phases and backends

Polygon fills and borders belong to the geometry pass. The shared example PNG
helper renders that pass at the selected bitmap supersampling factor and
downsamples it, while `TernaryText`, captions, axis text, and legend text are
drawn only in the final-resolution text pass. Annotation offsets and font sizes
are final output pixels and do not change with the geometry scale.

SVG uses the same prepared polygon vertices as native vector polygons and
normal Plotters text. The shared output helper keeps geometry in
`ternary-geometry` and text in `ternary-text`; it does not rasterise phase
regions or annotations. Quarter-turn annotation text remains native Plotters
SVG text. The richer arbitrary-angle SVG text path is intentionally limited to
the prepared axis-name facility documented in `axis-kernel.md`.

## Reference examples

- `examples/regions_annotations.rs` writes
  `examples/output/png/regions_annotations.png` and
  `examples/output/svg/regions_annotations.svg`. It combines alpha-bearing
  convex and concave regions, independent fill/border colours, Unicode region
  names, a line, scientific partitioned markers, axes, and native legends.
- `examples/cropped_regions.rs` writes
  `examples/output/png/cropped_regions.png` and
  `examples/output/svg/cropped_regions.svg`. It demonstrates a region with no
  original vertex inside the viewport, clipping across several viewport sides,
  an omitted `Anchor` annotation, an unrestricted offset annotation, and
  true-triangle-edge axis policy without a viewport frame.

## Carry-forward

Milestone 7 can reuse the simple-polygon clipping kernel for independently
supplied filled regions, but filled contours still require topology for bands,
holes, and adjacent level ownership. Polygon holes, general polygon-set
booleans, marker-bounds clipping, annotation bounds clipping, collision
avoidance, callouts, and arbitrary-angle general annotation text remain outside
this milestone.
