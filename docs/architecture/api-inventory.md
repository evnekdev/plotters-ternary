# API design inventory

This document inventories the intended public concepts. Names and signatures are provisional until validated against Plotters' actual type and lifetime constraints.

## Coordinates and validation

```rust
pub struct TernaryPoint { /* private A/B/C fields */ }

pub enum Component {
    A,
    B,
    C,
}

pub enum Normalization {
    RequireUnitSum,
    Normalize,
    RequireSum(f64),
}

pub struct Tolerance {
    pub absolute: f64,
    pub relative: f64,
}
```

Expected operations:

```rust
TernaryPoint::new(a, b, c)
point.sum()
point.validate(policy, tolerance)
point.component(Component::A)
point.as_array()
```

Tuple and array conversions should be supported without making raw tuples the primary API.

## Geometry

```rust
pub struct TernaryGeometry { /* private orientation and vertex order */ }

pub enum TriangleOrientation {
    Up,
    Down,
}

pub struct VertexOrder { /* validated private left/right/apex fields */ }

// Canonical default: left=B, right=C, apex=A.

pub struct TernaryCartesian {
    pub x: f64,
    pub y: f64,
}
```

Expected operations:

```rust
geometry.project(point, normalization, tolerance)
geometry.unproject(cartesian, tolerance)
geometry.vertex(component)
geometry.vertices()
geometry.classify(cartesian, tolerance)
geometry.triangle_edge(edge)
geometry.visible_edges(viewport, tolerance)
geometry.component_isoline(component, value, tolerance)
geometry.visible_component_isoline(component, value, viewport, tolerance)
```

## Viewport

```rust
pub struct TernaryViewport { /* private scalar bounds */ }

pub struct PixelRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub struct PixelPoint {
    pub x: f64,
    pub y: f64,
}

pub struct PixelBounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

pub enum ViewportFit {
    PreserveAspect,
    Stretch,
}

pub enum ViewportAlignment {
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

pub enum ViewportPointLocation {
    Inside,
    Boundary,
    Outside,
}

pub struct ViewportTransform { /* viewport, allocation, fitted bounds */ }

pub struct CartesianSegment {
    pub start: TernaryCartesian,
    pub end: TernaryCartesian,
}

pub enum TriangleEdge {
    LeftRight,
    RightApex,
    ApexLeft,
}
```

Implemented operations:

```rust
TernaryViewport::new(x_min, x_max, y_min, y_max)
TernaryViewport::new_with_tolerance(x_min, x_max, y_min, y_max, tolerance)
TernaryViewport::full(geometry)
viewport.classify(cartesian, tolerance)
viewport.contains(cartesian, tolerance)

ViewportTransform::new(viewport, pixel_rect, fit, alignment)
transform.logical_to_pixel(cartesian)
transform.pixel_to_logical(pixel)
transform.pixel_to_logical_checked(pixel)

clip_segment(segment, viewport, tolerance)
clip_segment_with_parameters(segment, viewport, tolerance)
```

## Chart

```rust
pub struct TernaryChartBuilder<'root, DB> { /* drawing-area layout inputs */ }
pub struct TernaryChart<'series, DB> { /* owned Cartesian ChartContext */ }
pub struct TernaryMeshConfig<'chart, 'series, 'axis, 'corner, DB> { /* borrowed config */ }
pub enum TernaryChartError<E> { /* geometry, drawing, grid, layout */ }
```

Implemented Milestone 3 builder operations:

```rust
TernaryChartBuilder::on(&root)
    .caption(...)
    .margin(...)
    .geometry(...)
    .viewport(...)
    .viewport_fit(...)
    .viewport_alignment(...)
    .build()
```

The implicit viewport is the full viewport of the final geometry. Once an
explicit viewport is selected, later geometry changes do not replace it.
Caption and margins are resolved before a fitted Plotters subarea is created.

Implemented chart operations:

```rust
chart.configure_mesh()
chart.plotting_area()
chart.cartesian_chart()
chart.cartesian_chart_mut()
chart.geometry()
chart.viewport()
chart.viewport_fit()
chart.viewport_alignment()
chart.draw_series(series)
chart.draw_point_series(series, marker)
chart.configure_series_labels()
```

`TernaryChart` owns a
`ChartContext<'series, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>`.
Its annotation lifetime is independent of the root borrow used during build.
`CartesianChartContext` and `CartesianPlottingArea` aliases describe the narrow
escape-hatch return types. `draw_series` projects and clips a `TernarySeries`,
delegates owned Plotters elements to the Cartesian context, and returns
Plotters' native `&mut SeriesAnno<'series, DB>`. `configure_series_labels`
forwards Plotters' native `SeriesLabelStyle`.

The initial mesh supports:

```rust
chart.configure_mesh()
    .major_step(0.1)
    .boundary_style(...)
    .major_grid_style(...)
    .text_style(...)
    .axis_label_offset(...)
    .corner_label_offset(...)
    .axis_a_name(...).axis_b_name(...).axis_c_name(...)
    .corner_a_name(...).corner_b_name(...).corner_c_name(...)
    .hide_axis_names()
    .hide_corner_names()
    .hide_grid_lines()
    .hide_triangle_boundary()
    .draw()
```

Independent axis tick specifications, tick labels, and cropped-axis placement
remain provisional Milestone 5 concepts below.

## Axes, ticks and mesh

```rust
pub enum TernaryAxis {
    A,
    B,
    C,
}

pub enum TickSpec {
    Count {
        major: usize,
        minor_per_major: usize,
    },
    Step {
        major: f64,
        minor: Option<f64>,
    },
    Values {
        major: Vec<f64>,
        minor: Vec<f64>,
    },
    None,
}

pub enum TickRangeMode {
    FullCompositionRange,
    VisibleRange,
}

pub enum AxisLabelFormatter<'a> {
    Decimal { precision: usize },
    Percentage { precision: usize },
    Custom(Box<dyn Fn(f64) -> String + 'a>),
}

pub enum CroppedAxisPolicy {
    TriangleEdgesOnly,
    RelocateMissingAxes,
    Manual,
}
```

Per-axis configuration should include:

- axis name;
- corresponding corner name or a reference to separate corner-label configuration;
- major and minor tick specification;
- major and minor grid styles;
- border and tick styles;
- tick direction and length;
- label formatter, offset and text style;
- axis-name placement and text style;
- visibility policy.

The high-level API should resemble Plotters:

```rust
chart
    .configure_mesh()
    .axis_a(|axis| {
        axis.name("CaO")
            .major_step(0.1)
            .minor_step(0.02)
            .labels_as_percent()
    })
    .axis_b(...)
    .axis_c(...)
    .border_style(BLACK.stroke_width(2))
    .draw()?;
```

## Drawing elements and series

Implemented Milestone 4 types:

```rust
pub struct TernaryLineSeries<I> { /* points and validation/style policy */ }
pub struct TernaryPointSeries<I, Provider = ()> { /* points and marker policy */ }
pub struct TernarySmoothSeries<I> { /* explicit composition spline policy */ }
pub enum TernaryInterpolation { Pchip, Akima, Makima, Steffen }
pub enum InvalidPointPolicy { Error, Break }
pub enum MarkerShape { /* fillable and stroke-only scientific shapes */ }
pub struct MarkerGeometry { /* validated shape plus rotation */ }
pub struct MarkerStyle { /* geometry, fill, one outer edge */ }
pub enum MarkerFill { Empty, Solid { .. }, Partitioned { .. } }
pub enum MarkerPartition { Radial { .. }, Horizontal, Vertical, /* ... */ }
pub struct MarkerSlice { pub weight: f64, pub color: RGBAColor }
pub struct MarkerElement<Coord> { /* concrete owned Plotters element */ }
pub enum MarkerClipMode { Centre, None }
pub enum SeriesError { /* indexed point validation, marker configuration */ }
pub trait TernarySeries<DB: DrawingBackend> { /* chart dispatch */ }
```

`prepare_polyline` and `prepare_points` expose backend-neutral preparation.
Remaining proposed types include `TernarySegment`, `TernaryPolyline`,
`TernaryPolygon`, `TernaryText`, `TernaryElement`, and
`TernaryContourSeries`.

### Lines

Features:

- Plotters `ShapeStyle`;
- open and closed paths;
- clipping while preserving off-screen segments needed for intersections;
- missing-value segmentation;
- optional dashed or dotted strategies;
- exact straight segments for TernaryLineSeries;
- explicit composition-space interpolation through TernarySmoothSeries;
- bounded private sampling followed by the normal clipping pipeline.

### Markers

The scientific-marker extension retains `Circle`, `Cross`, and `Triangle` compatibility while adding `Ellipse`, rectangles and rounded squares, diamonds, four triangle directions, regular polygons, stars, plus, diagonal cross, and asterisk shapes. `MarkerStyle` separates geometry, `MarkerFill`, and one common edge. It supports empty contours, solid independent fill/edge colours, and radial, horizontal, vertical, diagonal, or quadrant partitions through `MarkerSlice` values.

`TernaryPointSeries::marker_style` selects a uniform complete style;
`point_style_provider(|source_index, normalized_abc| ...)` selects one without
changing source observations or creating one Plotters series per variant. The
concrete `MarkerElement<Coord>` can be returned from custom marker code and a
native `SeriesAnno::legend` closure. `draw_point_series` continues to accept a
closure returning any owned ordinary Plotters element at the projected anchor.

```rust
pub enum MarkerClipMode {
    Centre,
    None,
}
```

`Centre` remains centre-only clipping rather than marker-bounds clipping.

### Polygons and text annotations

Milestone 6 implements backend-neutral `prepare_polygon` and public
`TernaryPolygon<I>`. A polygon accepts compositions convertible to
`TernaryPoint`, supports independently optional `.fill_style(...)` and
`.border_style(...)`, and is rendered through the normal
`chart.draw_series(...)` route with native Plotters `SeriesAnno` legends.
Open and explicitly closed simple loops are accepted. Fewer than three distinct
vertices, zero area, invalid source compositions, and self intersections are
reported through `PolygonError`; simple concave subjects are supported.
Sutherland?Hodgman clipping occurs against the logical rectangular viewport
before Plotters receives the final vertices.

`TernaryText` has an owned ternary anchor, UTF-8 text, owned `AxisTextStyle`,
`TextAnchor`, final-pixel offset, validation policy, `AnnotationClipMode`, and
native quarter-turn `TextRotation`. `AnnotationClipMode::Anchor` tests only the
logical anchor; `None` is unrestricted. Bounds-aware clipping and general
arbitrary-angle annotations are deliberately not claimed yet.

## Contours

Milestone 7 implements regular two-dimensional ternary grids only:

```rust
pub struct RegularTernaryScalarField { /* n and canonical ordered values */ }
pub enum ContourInterpolation { Linear, CubicAlpha(CubicAlphaOptions) }
pub enum BinaryExtrapolation { RawBarycentric, Muggianu, Kohler }
pub struct ContourSet { pub levels: Vec<ContourLevel>, /* diagnostics */ }
pub struct ContourLevel { pub value: f64, pub paths: Vec<ContourPath> }
pub struct ContourPath { pub points: Vec<TernaryPoint>, pub closed: bool }
pub struct TernaryContourSeries<'a> { /* native Plotters adapter */ }
```

`RegularTernaryScalarField` uses integer triples `i+j+k=n` in row-major `(i,j)` ordering and generates its elementary connectivity internally. Linear contours use deterministic marching triangles. Cubic-alpha contours use `spline1d` edge intervals, shared canonical edge direction, explicit RawBarycentric/Muggianu/Kohler interior extrapolation, adaptive barycentric microtriangles, and optional arc-length redistribution plus analytic-gradient level projection. See [contour-kernel.md](contour-kernel.md). Arbitrary triangulations, irregular data, Kuhn simplices, filled contours, and N-component grids are not part of this API.

## Optional mathematical text

```rust
pub trait MathTextRenderer {
    type Error;

    fn measure(&self, expression: &str, style: &MathTextStyle)
        -> Result<(u32, u32), Self::Error>;

    fn render(&self, expression: &str, style: &MathTextStyle)
        -> Result<RenderedText, Self::Error>;
}
```

Potential optional implementations include LaTeX, Typst or a restricted built-in math renderer. None should be mandatory for core use.

## Error model

The crate should expose an error type that distinguishes:

- invalid or non-finite composition;
- failed normalisation constraints;
- degenerate viewport;
- projection or reverse-projection failure;
- unsupported text rotation or renderer capability;
- contour topology errors;
- wrapped Plotters drawing errors where practical.

Exact generic error wrapping should be designed after experimenting with Plotters' backend error types.

## Milestone 5 implemented axis API

`TernaryAxis`, `TernaryAxisConfig`, `TickSpec`, `TickRangeMode`,
`CroppedAxisPolicy`, `TickDirection`, `AxisLabelFormat`,
`EndpointLabelPolicy`, `CornerLabelVisibility`, `AxisNamePosition`,
`AxisTextStyle`, and `TickStyle` are now exported. Configure a semantic axis
with `mesh.axis(TernaryAxis::A, |axis| { ... })`; compatibility shortcuts
remain available on `TernaryMeshConfig`. See [axis-kernel.md](axis-kernel.md)
for the implemented opposite-edge and clipping semantics.
