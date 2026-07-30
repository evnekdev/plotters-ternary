# Milestone 4 series and legend kernel

## Public series API

Milestone 4 adds backend-neutral preparation and Plotters-backed drawing through:

- `TernaryLineSeries<I>`;
- `TernaryPointSeries<I>`;
- `TernarySeries<DB>` as the dispatch bound for `TernaryChart::draw_series`;
- `MarkerShape::{Circle, Cross, Triangle}`;
- `MarkerClipMode::{Centre, None}`;
- `InvalidPointPolicy::{Error, Break}`;
- `SeriesError`;
- `prepare_polyline` and `prepare_points`.

The normal line path is:

```rust,no_run
use plotters::prelude::*;
use plotters_ternary::{TernaryLineSeries, TernaryPoint};
# fn draw<DB: DrawingBackend>(chart: &mut plotters_ternary::TernaryChart<'_, DB>)
# -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>>
# where DB: 'static {
let points = [
    TernaryPoint::new(0.7, 0.2, 0.1),
    TernaryPoint::new(0.2, 0.6, 0.2),
];
chart
    .draw_series(TernaryLineSeries::new(points, BLUE.stroke_width(2)))?
    .label("Liquidus boundary")
    .legend(|(x, y)| {
        PathElement::new([(x, y), (x + 20, y)], BLUE.stroke_width(2))
    });
# Ok(())
# }
```

`TernaryLineSeries::new` stores its input `IntoIterator` without eagerly
collecting it. It accepts any item convertible into `TernaryPoint`, including
the crate's array and tuple conversions. Configuration methods select
`normalization`, `tolerance`, and `invalid_point_policy`; the line style is a
normal Plotters `ShapeStyle`.

## Validation and invalid points

Both line and point series default to `Normalization::RequireUnitSum`,
`Tolerance::default()`, and `InvalidPointPolicy::Error`. No normalization is
silent. Callers opt into `Normalization::Normalize` or `RequireSum` on the
series.

`Error` stops preparation and returns
`SeriesError::InvalidPoint { index, source }`, retaining the original source
index and the underlying `coord::Error`. For a line, `Break` finalizes the
current source run and starts a new run after the invalid point; neighbours are
never joined across it. For independent markers, `Break` omits the invalid
point. An empty line or point series is a valid no-op and Plotters still
allocates a native annotation, so it can participate in an explicitly desired
legend entry.

## Polyline preparation and clipping

`prepare_polyline` first projects each valid source run through the chart's
`TernaryGeometry`. It then processes every consecutive complete source segment
with the directed Liang-Barsky viewport clipper. It does not filter endpoints
before clipping, so a segment with two outside endpoints can still cross the
viewport.

Surviving segments are appended only when the prior visible endpoint and the
new clipped start agree within tolerance. A segment ending before its original
source endpoint closes the current visible subpath; a later re-entry starts a
new one. Fully invisible segments and invalid-point breaks also close the
subpath. Adjacent duplicates introduced by shared clipped endpoints or repeated
source points are removed within tolerance. Segment direction is retained.
Corner tangencies and visible zero-length segments are represented as one-point
subpaths. A single visible source point is likewise retained, though its
`PathElement` has no visible stroke.

The result is `Vec<Vec<TernaryCartesian>>` and remains independent of every
Plotters backend. `TernaryLineSeries` converts each subpath into a separate
owned `PathElement`, so Plotters never connects materially separated geometry.

## Explicit smooth series

`TernaryLineSeries` remains an exact polyline: it never interpolates or adds
cosmetic subdivision points. Smooth rendering is selected explicitly with
`TernarySmoothSeries<I>` and `TernaryInterpolation::{Pchip, Akima, Makima,
Steffen}`. The implementation uses `spline1d` 0.1.0 with its allocation-backed
API; no cubic algorithm is copied into this crate.

The ordered source index is the strictly increasing parameter `t`. The kernel
validates and converts source compositions to semantic unit A/B/C values, fits
`A(t)` and `B(t)` with the selected `spline1d::Spline`, and derives
`C(t) = 1 - A(t) - B(t)`. It never splines projected Cartesian X/Y, so vertex
order and triangle orientation cannot change the semantic composition curve.
Every generated sample is validated. Negatives no larger than the configured
absolute tolerance are cleaned to zero and the generated point is
renormalized; material simplex violations return
`SeriesError::InvalidInterpolatedPoint` under `Error` or split the curve under
`Break`.

The initial sampling policy is a documented bounded fallback of 24 samples per
source interval, configurable through `samples_per_interval`. Values must be in
`1..=4096`, and a complete curve is capped at 100,000 samples. Adaptive
pixel-deviation sampling is deferred. Source knots are included exactly in the
sample schedule. Samples are private rendering details: they are not returned
as observations and never create markers.

The complete composition curve is sampled before projection. Its sampled
logical segments then use the same directed clipping and visible-subpath
splitting as exact lines. One backend-neutral preparation path feeds PNG and
SVG. Bitmap rendering may additionally supersample geometry; SVG receives the
same sampled points as vector polylines without further interpolation or
resampling.
## Point series and marker closures

The built-in API is:

```rust,no_run
# use plotters::prelude::*;
# use plotters_ternary::{MarkerShape, TernaryPoint, TernaryPointSeries};
let series = TernaryPointSeries::new([
    TernaryPoint::new(0.2, 0.3, 0.5),
])
.size(6)
.style(RED.filled())
.marker(MarkerShape::Circle);
```

`size` is a backend-pixel radius or half-size and must be nonzero. Built-in
markers delegate to Plotters `Circle`, `Cross`, and `TriangleMarker` elements.
Styles are ordinary `ShapeStyle` values.

Custom composition works through `TernaryChart::draw_point_series(series,
closure)`. The closure receives `((x, y), size, style)`, where `(x, y)` is the
projected logical anchor. It runs only for prepared markers and returns one
owned concrete Plotters element. For example it may return
`EmptyElement::at(anchor) + Cross::new((0, 0), ...) + Circle::new((0, 0), ...)`.
The closure and projected point vector are consumed during drawing and are not
stored in the series annotation. A closure must return one concrete element
type, but Plotters composable elements allow multiple local shapes without
dynamic `Any` storage or leaked allocations.

## Marker clipping

`MarkerClipMode::Centre` is the default. The marker is submitted only when its
projected centre is inside or on the logical viewport. Its pixel bounds may
extend past the viewport; this is explicitly not bounds clipping.

`MarkerClipMode::None` submits every valid projected centre. It is an escape
hatch, and an outside centre may be affected by Plotters' plotting-area
coordinate truncation. `Bounds` remains deferred because portable geometric
marker-bounds clipping requires a stronger backend-aware model.

## Native annotations and legends

```rust
pub fn TernaryChart::draw_series<'chart, S>(
    &'chart mut self,
    series: S,
) -> Result<
    &'chart mut SeriesAnno<'series, DB>,
    TernaryChartError<DB::ErrorType>,
>
```

The return value is Plotters' native `SeriesAnno`, not a crate wrapper. The
reference borrow lasts for the call's mutable chart borrow (`'chart`), while
legend closures are stored with the chart context's existing `'series`
lifetime. Prepared paths and marker elements are owned and exhausted before the
annotation is returned, so they cannot be borrowed by stored legend closures.
Legend closures should capture owned or sufficiently long-lived styles exactly
as they do with ordinary Plotters.

`TernaryChart::configure_series_labels()` forwards and returns Plotters'
`SeriesLabelStyle<'series, 'chart, DB, Cartesian2d<...>>` unchanged. All normal
background, border, font, position, margin, and legend-area methods therefore
remain available. The crate introduces no ternary legend type.

### Reference legend alignment

The shared series-example renderer keeps native Plotters legends, but adapts
the callback coordinate through a private `LegendRowLayout`. Plotters supplies
the left edge of its legend area; the adapter reserves a 34-pixel symbol slot
and a 12-pixel label gap, then supplies the slot centre to every line, circle,
triangle, cross, and custom-symbol closure. The configured native legend area
is therefore 46 pixels wide, with 12 pixels of outer padding. Plotters still
measures label widths and row heights when it sizes the box. At final
1000-by-800 resolution the full-series legend uses centre X = 644 and label
start X = 673; the cropped legend uses centre X = 253 and label start X = 282.
The nominal row height is rendered with normal integer-pixel rounding, so
adjacent SVG row centres differ by at most one pixel.

This is shared example/output infrastructure, not a replacement for the
public native `SeriesAnno::legend` API. In the adapter's custom-symbol contract
its closure coordinate is the physical centre of the allocated symbol slot.
PNG geometry and SVG use the same adapter; the final-resolution PNG text pass
uses the same unscaled label starts and font size.

## Error strategy

`TernaryChartError<E>` now has a `Series(SeriesError)` variant alongside the
existing geometry, layout, and drawing variants. `SeriesError` preserves
invalid source indexes and nested coordinate errors, and reports a zero marker
size. Drawing still uses `DrawingAreaErrorKind<DB::ErrorType>` without adding
backend types to the coordinate or preparation kernels.

## Permanent examples

- `examples/lines_points_legend.rs` generates
  `examples/output/{png,svg}/lines_points_legend.*`;
- `examples/cropped_crossing_series.rs` generates
  `examples/output/{png,svg}/cropped_crossing_series.*`.

The first contains two scientific synthetic lines, circle and triangle point
series, a closure-composed marker, and five ordinary Plotters legend entries.
The cropped example contains an outside-to-outside crossing, a line split into
two visible subpaths after an outside excursion, centre-clipped markers, and
three legend entries. Both formats have final dimensions of 1000 by 800. The full series example includes one exact liquidus polyline and one explicit PCHIP solvus curve. SVG uses separate geometry/text vector groups and preserves the sampled Plotters polylines unchanged. PNG uses the optional two-pass helper: geometry is rendered at 3x and Lanczos3-downsampled, while text is rendered once at final resolution. This development-only fallback remains outside the crate API and can be switched to `BitmapQuality::Native`. Logical compositions and viewports are unchanged.

## Carry-forward

Milestone 5 must add publication-quality axes, independent tick/grid policy,
and cropped-label placement without changing the native annotation path.
Milestone 6 must reuse visible-subpath concepts for polygon and region
clipping. Marker-bounds clipping, explicit optional/missing-value inputs beyond
invalid compositions, dashed-path semantics across split subpaths, and
legend/layout reservation outside the fitted plot remain future design work.
