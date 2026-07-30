# Rendering, series, contours and text

## Plotters integration

The high-level chart should wrap a normal Cartesian Plotters `ChartContext`. Ternary elements are projected and clipped before being handed to Plotters.

This preserves:

- bitmap, SVG and other Plotters backends;
- `ShapeStyle`, colours, fonts and text styles;
- captions and surrounding drawing-area layouts;
- `draw_series` conventions;
- series labels and legends;
- access to ordinary Plotters elements for advanced annotations.

The wrapper should provide controlled escape hatches to the underlying chart context and Cartesian plotting area.

## Mesh rendering

A ternary mesh consists of independently configurable concerns:

1. original triangular boundary;
2. A, B and C component isolines;
3. major and minor ticks;
4. tick labels;
5. axis names;
6. A/B/C corner names.

Each component axis should support its own tick specification, formatter and styles. Shared convenience methods may apply a setting to all three axes.

Grid lines are constructed geometrically in ternary coordinates and clipped through the same pipeline as data series. This ensures identical behaviour in full and trimmed views.

## Line and path series

`TernaryLineSeries` should accept an iterator of ternary points and a Plotters style. It should:

- validate or normalise points according to chart policy;
- preserve off-screen points for intersection calculations;
- split at explicit missing or invalid data according to a documented policy;
- project complete line segments;
- clip them to the triangle and rectangular viewport;
- emit one or more Plotters path elements;
- return Plotters-compatible series annotation for legends.

Basic rendering should not implicitly interpolate. Spline or smoothing helpers can generate a ternary polyline before drawing.

## Markers and points

`TernaryPointSeries` should provide built-in marker conveniences and an advanced closure-based element constructor analogous to Plotters' point series.

Likely built-ins:

- circle;
- square;
- triangle;
- diamond;
- cross;
- plus.

Users must be able to compose arbitrary Plotters elements around a projected ternary anchor.

## Polygons and phase regions

`TernaryPolygon` should support:

- fill style;
- border style;
- optional closure validation;
- clipping against the full triangle and viewport;
- transparent fills through normal Plotters colour support.

This type is the basis for phase fields and future filled contours.

## Generic anchored elements

A generic adapter should permit Plotters elements to be anchored at ternary coordinates without a dedicated wrapper for every element type.

Conceptually:

```rust
TernaryElement::at(point)
    + Circle::new((0, 0), 4, RED.filled())
    + Text::new("sample", (6, -4), style)
```

The precise implementation must be tested against Plotters' composable element traits and coordinate ownership.

## Text annotations

Baseline text should reuse Plotters text styles and support:

- ternary anchor;
- pixel offset;
- horizontal and vertical anchoring;
- colour, font and size;
- optional background and border;
- clipping policy;
- rotation.

Plotters' native font transforms should be used for quarter turns. Arbitrary angle support, especially the +/-60 degree orientations natural to triangular axes, needs an explicit capability strategy.

Possible strategies:

1. Initially expose only supported rotations and return an error for arbitrary angles.
2. Rasterise text to a transparent image and rotate it; portable but loses vector text.
3. Add backend-specific vector support where available.
4. Use an optional mathematical-text renderer that can emit SVG or bitmap fragments.

The public API may reserve `TextRotation::Angle(f64)` before every backend can honour it, provided failures are explicit rather than silently ignored.

## Mathematical and LaTeX-like text

Core operation must not require a TeX installation. Ordinary Unicode handles many scientific labels, including Greek symbols, subscripts, superscripts and chemical formulas.

An optional `MathTextRenderer` abstraction can later support:

- external LaTeX;
- Typst;
- a lightweight built-in subset;
- pre-rendered SVG or bitmap fragments.

The renderer should expose both measurement and rendering so layout can reserve correct space. Rendered expressions should be cached by expression, style and scale.

## Contour architecture

Contour generation and rendering are separate.

### Input field

A `TernaryScalarField` contains scalar samples plus a triangular connectivity description. It may represent:

- a regular composition grid;
- a custom triangulation;
- scattered points after external triangulation.

### Isoline generation

The first algorithm should be marching triangles:

1. For each requested level, inspect each triangular cell.
2. Classify cell vertices relative to the level.
3. Interpolate intersections on crossed edges.
4. Emit zero or one ordinary segment per non-degenerate triangle.
5. Apply a consistent rule for exact-level vertices and flat edges.
6. Join adjacent segments into ordered paths using tolerance-aware endpoint matching.

Output:

```rust
pub struct ContourSet {
    pub levels: Vec<ContourLevel>,
}

pub struct ContourLevel {
    pub value: f64,
    pub paths: Vec<Vec<TernaryPoint>>,
}
```

### Rendering

`TernaryContourSeries` receives paths and styles, then uses the normal line-series projection and clipping pipeline. Users may therefore render contours generated by this crate or another package.

### Filled contours

Filled contours are a later phase because they require robust polygon topology, holes, level bands and clipping. They should not block isoline support.

## Legend behaviour

Ternary series should participate in ordinary Plotters legend configuration. The crate should not invent a separate legend system.

A target usage pattern is:

```rust
chart
    .draw_series(TernaryLineSeries::new(points, RED))?
    .label("Liquidus")
    .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], RED));

chart.configure_series_labels().draw()?;
```

The permanent series examples use a shared internal legend-row adapter around
those ordinary Plotters closures. It reserves one fixed symbol slot and gives
line and marker renderers the same slot-centre coordinate; its layout controls
slot width, label gap, and outer padding in one place. This corrects visual
alignment without changing the public native legend API.

## Rendering tests

Rendering tests should combine numerical assertions and reference images:

- exact projected and clipped coordinates for geometry tests;
- SVG output inspection for paths and text placement;
- small deterministic PNG or SVG snapshots for visual regression;
- examples covering full, side-cropped and interior viewports;
- legends and captions outside the clipped viewport;
- backend parity checks where practical.

## Milestone 3 implemented chart integration

The first production chart now owns a Cartesian `ChartContext` and lays it out
on an aspect-fitted Plotters subarea after caption and margin allocation. The
internal Cartesian ranges remain exactly the requested logical viewport.
Boundary and major-grid paths are mathematically clipped by the coordinate
kernel before Plotters receives them. A generic `TernaryChartError<E>` keeps
Plotters backend errors separate from `coord::Error`.

The initial `TernaryMeshConfig` deliberately has one common major step. Corner
names are semantic, offset outward from visible vertices, and anchored away
from the boundary. Axis names use the midpoint of the semantic opposite edge,
are offset outward, and follow that edge's actual pixel-space angle. Horizontal
names remain native Plotters text. Sloping names use the isolated rotated-glyph
element documented in `chart-kernel.md`: PNG is raster, SVG remains vector-only
but the sloping glyphs are rectangle outlines rather than searchable text.
Names whose geometric anchors are cropped are omitted, never relocated to an
invisible viewport side.

`TernaryLineSeries` and `TernaryPointSeries` now carry explicit
`Normalization`, `Tolerance`, and `InvalidPointPolicy` configuration. Strict
unit-sum validation and indexed errors are defaults. `Break` terminates a line
run at an invalid point and never joins its neighbours. Line preparation clips
every complete source segment and assembles separate owned visible subpaths,
retaining outside-to-outside crossings and traversal direction.

`TernarySmoothSeries` is the explicit interpolation path; ordinary lines remain
exact. It uses `spline1d` 0.1.0 for PCHIP, Akima, MAKIMA, or Steffen interpolation
of semantic A(t) and B(t), derives C(t), validates every private rendering
sample, then projects and clips the complete sampled curve. The initial bounded
fallback uses 24 samples per source interval rather than an adaptive
pixel-deviation criterion. PNG and SVG share this backend-neutral sampled
logical path.
Point series provide circle, cross, and triangle markers plus an owned custom
Plotters-element closure. `Centre` clipping tests only the projected anchor;
`None` is the explicit unfiltered escape hatch. `TernaryChart::draw_series`
returns native `&mut SeriesAnno`, and `configure_series_labels` forwards native
`SeriesLabelStyle`, preserving ordinary Plotters legends without wrappers.