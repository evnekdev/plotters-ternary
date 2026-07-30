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
pub struct TernaryChartBuilder<'a, DB> { /* provisional */ }
pub struct TernaryChart<'a, DB> { /* provisional */ }
```

Intended builder operations:

```rust
TernaryChartBuilder::on(&root)
    .caption(...)
    .subtitle(...)
    .margin(...)
    .viewport(...)
    .viewport_fit(...)
    .viewport_alignment(...)
    .normalization(...)
    .build()
```

Intended chart operations:

```rust
chart.configure_mesh()
chart.draw_series(...)
chart.draw_text(...)
chart.configure_series_labels()
chart.plotting_area()
chart.cartesian_chart()
chart.cartesian_chart_mut()
chart.project(point)
chart.unproject(pixel_or_cartesian)
chart.classify(point)
```

The exact return type from `draw_series` should preserve ordinary Plotters series annotation and legend support.

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

Proposed high-level types:

```rust
TernarySegment
TernaryPolyline
TernaryLineSeries
TernaryPointSeries
TernaryPolygon
TernaryText
TernaryElement
TernaryContourSeries
```

### Lines

Features:

- Plotters `ShapeStyle`;
- open and closed paths;
- clipping while preserving off-screen segments needed for intersections;
- missing-value segmentation;
- optional dashed or dotted strategies;
- future interpolation helpers kept separate from basic rendering.

### Markers

Built-in conveniences may include circles, squares, triangles, diamonds, crosses and plus signs. Advanced users must also be able to provide Plotters element closures.

```rust
pub enum MarkerClipMode {
    Centre,
    Bounds,
    None,
}
```

### Polygons

Support independent fill and border styles. Polygon clipping must occur against the rectangular viewport.

### Text

```rust
pub enum TextRotation {
    None,
    Deg90,
    Deg180,
    Deg270,
    Angle(f64),
}
```

Arbitrary angles may initially be unsupported or implemented through an optional renderer. The API should reserve the concept without claiming backend-independent vector support prematurely.

Text annotations should support:

- ternary anchor point;
- pixel offset;
- Plotters text style;
- anchor/alignment;
- rotation;
- optional background and border;
- optional mathematical-text renderer.

## Contours

```rust
pub struct TernaryScalarField { /* samples and triangulation */ }
pub struct TernaryTriangulation { /* vertices and cells */ }
pub struct ContourSet {
    pub levels: Vec<ContourLevel>,
}
pub struct ContourLevel {
    pub value: f64,
    pub paths: Vec<Vec<TernaryPoint>>,
}
```

Contour generation should use marching triangles and remain independent of Plotters. Rendering accepts either internally generated or externally supplied contour paths.

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