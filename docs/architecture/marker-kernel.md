# Scientific marker kernel

## Purpose and boundary

The scientific-marker extension adds portable marker geometry beneath
`TernaryPointSeries`; it does not change ternary coordinates, interpolation,
viewports, or Plotters' legend system. The crate keeps using
`TernaryChart::draw_series`, which returns Plotters' native `SeriesAnno`, and
callers configure legends through the normal `configure_series_labels` path.

The core marker model is independent of a concrete Plotters backend until a
`MarkerElement<Coord>` is drawn. It uses Plotters colour and `ShapeStyle` types
for interoperability, but has no bitmap-quality or `image` dependency.

## Public model

```rust
use plotters::prelude::*;
use plotters_ternary::{MarkerShape, MarkerStyle, TernaryPoint, TernaryPointSeries};

let style = MarkerStyle::solid(
    MarkerShape::Diamond,
    RGBColor(225, 95, 50),
    BLACK.stroke_width(2),
)?;
let points = TernaryPointSeries::new([TernaryPoint::new(0.2, 0.3, 0.5)])
    .size(9)
    .marker_style(style);
# Ok::<(), plotters_ternary::MarkerError>(())
```

`MarkerGeometry` stores a validated `MarkerShape` and optional extra rotation.
`MarkerStyle` contains a geometry, a `MarkerFill`, and one optional outer
`ShapeStyle`. `MarkerFill` is `Empty`, `Solid { color }`, or `Partitioned`.
`MarkerSlice` stores finite positive relative weights and an `RGBAColor`.
Constructor colours implement Plotters' `Color` trait and are converted to
RGBA immediately.

The compatibility path remains unchanged:

```rust
# use plotters::prelude::*;
# use plotters_ternary::{MarkerShape, TernaryPoint, TernaryPointSeries};
let points = TernaryPointSeries::new([TernaryPoint::new(0.2, 0.3, 0.5)])
    .size(6)
    .style(RED.filled())
    .marker(MarkerShape::Circle);
```

`MarkerShape::Triangle` is retained as the historical spelling for
`TriangleUp`. `Plus` is the orthogonal `+`; `Cross` is the diagonal `?`.
Fillable shapes are circle/ellipse, square/rectangle/rounded square, diamond,
the four triangle directions, regular polygons, and stars. `Plus`, `Cross`,
and `Asterisk` are intentionally stroke-only and reject non-empty fills.
Convenience constructors cover pentagons, hexagons, octagons, and 4/5/6/8
point stars.

## Centre and tessellation contract

Every outline is generated in local floating-point coordinates and normalized
to a centred `[-1, 1] ? [-1, 1]` bounding box. A `MarkerElement` anchor is
always that visual centre, both in a plot and in a legend. The requested size
is the backend-pixel half-extent. This prevents circles, triangles, stars, and
stroke-only symbols from shifting when mixed in a native Plotters legend.

Circles and ellipses use a fixed 32-vertex closed outline. The fixed bounded
policy means the exact same logical polygon topology reaches PNG and SVG;
bitmap supersampling only changes the rasterisation of that geometry.

## Fills and partitions

`MarkerPartition::Radial` accepts one to four slices. Angles begin at
`start_angle_deg`; `0?` points right and positive angles rotate
counter-clockwise in ordinary mathematical space (visually toward the top in
backend pixel coordinates). `SweepDirection` selects positive or negative
sweep. Slices follow source-vector order and their positive finite weights are
normalized internally.

`Horizontal`, `Vertical`, `DiagonalForward`, and `DiagonalBackward` require
exactly two slices. Their first slice is respectively upper, left, the
upper/right side of `/`, and the upper/left side of `\`. `Quadrants` requires
four slices in upper-right, lower-right, lower-left, upper-left order before
the supplied counter-clockwise rotation.

Radial fills are built by intersecting wedge half-planes with a fan of the
outer marker polygon. Linear and quadrant fills use the same reusable convex
half-plane clipping. Concave stars are decomposed into centre-to-outline fans,
so each partition remains inside the visible star. Draw order is always:

```text
partition fills -> optional divider segments -> one common outer edge
```

The outer edge is never redrawn per partition. Empty markers require an edge;
solid and partitioned markers may deliberately omit one. `MarkerStyle::fact_sage`
creates the common same-colour fill-and-edge form.

## Per-point phase combinations

A uniform `marker_style` is the common case. For experiment-specific phase
mixtures, `point_style_provider` receives `(original_source_index,
normalized_abc_composition)` and returns a complete `MarkerStyle`:

```rust
# use plotters::prelude::*;
# use plotters_ternary::{MarkerShape, MarkerStyle, TernaryPoint, TernaryPointSeries};
let points = TernaryPointSeries::new([
    TernaryPoint::new(0.2, 0.3, 0.5),
    TernaryPoint::new(0.3, 0.4, 0.3),
])
.marker_style(MarkerStyle::solid(MarkerShape::Circle, RED, BLACK)?)
.point_style_provider(|index, _composition| {
    if index == 0 {
        MarkerStyle::solid(MarkerShape::Diamond, BLUE, BLACK).unwrap()
    } else {
        MarkerStyle::solid(MarkerShape::Circle, GREEN, BLACK).unwrap()
    }
});
# Ok::<(), plotters_ternary::MarkerError>(())
```

The callback is stored directly in the series generic parameter rather than a
boxed `'static` trait object. It is evaluated during drawing, preserves source
indexes after centre clipping, and does not require a separate Plotters series
for every phase combination.

## Legend, PNG and SVG behaviour

`MarkerElement<(i32, i32)>` is a concrete ordinary Plotters element, so a
legend can use it directly in `SeriesAnno::legend`. The examples adapt the
callback anchor through one shared fixed-width symbol slot: all line and
marker symbols use the same centre and label start. This remains example
layout infrastructure, not a ternary-specific legend API.

The example PNG helper's optional supersampled geometry pass calls
`MarkerStyle::scaled` and scales marker half-size, edge widths, and dividers.
Its final-resolution text pass is unchanged. SVG receives the same unscaled
local polygon geometry in the vector `ternary-geometry` group and never
contains raster marker images.

## Validation and deferred work

Regular polygons, stars, and asterisks accept 3 through 16 sides/points/arms.
Stars require `0 < inner_ratio < 1`; ellipses and rectangles require a finite
positive aspect ratio; rounded-square corner ratio is finite in `0..=0.5`;
all rotations are finite. Partitions reject empty input, more than four
slices, incorrect fixed partition counts, and non-finite or non-positive
weights. `MarkerError` is nested by `SeriesError::Marker` with an optional
source index.

Marker-bounds clipping remains deferred: `MarkerClipMode::Centre` clips only
the anchor, while `None` is the documented unrestricted escape hatch. General
phase-region polygons, annotations, and contour work remain separate future
milestones.

## Permanent gallery

`examples/custom_markers.rs` writes:

- `examples/output/png/custom_markers.png`;
- `examples/output/svg/custom_markers.svg`.

It includes all built-in scientific shape families, empty and solid forms,
independent fill/edge colours, all supported split modes, equal and weighted
radial sectors, four experimental phase combinations selected through the
per-point style callback, and native Plotters legend rows.
