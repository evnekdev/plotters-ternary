# plotters-ternary

`plotters-ternary` adds publication-quality ternary composition diagrams to
[Plotters](https://crates.io/crates/plotters). It keeps A/B/C compositions,
logical viewport clipping, regular-grid contours, and scientific drawing
primitives separate from backend rendering, while preserving ordinary Plotters
captions, styles, `SeriesAnno`, and legends.

## Capabilities

- validated ternary compositions, configurable vertex order, upward/downward geometry;
- full, cropped, and interior logical viewports with invisible mathematical clipping;
- publication-oriented A/B/C axes, grids, ticks, labels, and native SVG rotated axis names;
- exact/smooth lines, scientific markers, polygons, annotations, and normal Plotters legends;
- regular-grid line contours: piecewise-linear or optional cubic-alpha;
- PNG output with optional geometry-only supersampling and native vector SVG.

## Installation

```toml
[dependencies]
plotters-ternary = "0.1.0"
plotters = "0.3.7"
```

Enable cubic-alpha contour construction when needed:

```toml
plotters-ternary = { version = "0.1.0", features = ["cubic-alpha"] }
```

The feature enables cubic contour construction only. `spline1d` remains a
normal transitive dependency because smooth line series also use it.

## Minimal chart

```rust,no_run
use plotters::prelude::*;
use plotters_ternary::prelude::*;

let root = BitMapBackend::new("ternary.png", (1000, 800)).into_drawing_area();
root.fill(&WHITE)?;
let mut chart = TernaryChartBuilder::on(&root)
    .caption("Ternary diagram", ("sans-serif", 32))
    .margin(60)
    .build()?;
chart.configure_mesh()
    .corner_a_name("A")
    .corner_b_name("B")
    .corner_c_name("C")
    .draw()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

For a cropped diagram, select a logical viewport; it never becomes a visible
Cartesian frame:

```rust,no_run
# use plotters_ternary::{TernaryChartBuilder, TernaryViewport};
# use plotters::prelude::*;
# let root = BitMapBackend::new("cropped.png", (1000, 800)).into_drawing_area();
let mut chart = TernaryChartBuilder::on(&root)
    .viewport(TernaryViewport::new(0.42, 0.90, 0.05, 0.58)?)
    .build()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Series, markers, axes, and annotations

Use `TernaryLineSeries`, `TernaryPointSeries`, `TernaryPolygon`, and
`TernaryText` through `chart.draw_series(...)`. The return value is Plotters'
native annotation, so legends remain ordinary Plotters legends:

```rust,no_run
# use plotters::prelude::*;
# use plotters_ternary::{TernaryChartBuilder, TernaryLineSeries, TernaryPoint};
# let root = BitMapBackend::new("series.png", (1000, 800)).into_drawing_area();
# let mut chart = TernaryChartBuilder::on(&root).build()?;
chart.draw_series(TernaryLineSeries::new(
    [TernaryPoint::new(0.7, 0.2, 0.1), TernaryPoint::new(0.2, 0.5, 0.3)],
    BLUE.stroke_width(2),
))?
.label("Measured boundary")
.legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], BLUE.stroke_width(2)));
chart.configure_series_labels().draw()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

See the source examples for [markers and legends](examples/lines_points_legend.rs),
[custom axes](examples/custom_axes.rs), and
[phase regions and annotations](examples/regions_annotations.rs).

## Contours

The backend-independent `ternary-contours` dependency constructs complete
semantic A/B/C contour paths. This crate only projects and visually clips those
final paths for Plotters. Backend, viewport, output-size, style, and
supersampling choices cannot change the numerical `ContourSet`.

`ContourInterpolation::Linear` is the exact piecewise-affine baseline on a
regular ternary grid. Cubic-alpha contours reuse directed one-dimensional
`spline1d` intervals along every shared grid edge and use adaptive topology
extraction plus optional level-preserving regularization.

```rust,no_run
# use plotters_ternary::{ContourOptions, ContourSet, RegularTernaryScalarField};
# let field = RegularTernaryScalarField::from_fn(1, |[a, b, c]| 2.0 * a - 3.0 * b + 5.0 * c)?;
let contours = ContourSet::compute(&field, &[0.5, 1.0], ContourOptions::linear())?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

For cubic-alpha use `ContourOptions::cubic_alpha(CubicAlphaOptions { .. })`.
`Muggianu` is the default symmetric binary projection:
`xj + xk/2`. `Kohler` preserves the binary ratio: `xj/(xi+xj)`. Both retain
the required raw `xi*xj` pair prefactor and reproduce the same source spline
on the binary edge. `RawBarycentric` is explicitly experimental and should not
be described as Muggianu.

## Gallery

| Full chart | Axes and markers | Contours |
| --- | --- | --- |
| ![Full triangle](examples/output/png/full_triangle.png) | ![Custom axes](examples/output/png/custom_axes.png) | ![Cubic-alpha contours](examples/output/png/cubic_alpha_contours.png) |

More references: [cropped axes](examples/output/png/cropped_axes.png),
[markers](examples/output/png/custom_markers.png),
[regions](examples/output/png/regions_annotations.png), and
[cropped contours](examples/output/png/cropped_contours.png). SVG counterparts
live beside every PNG under `examples/output/svg/`.

## Features and limits

- `default`: geometry, Plotters charting, all standard series, linear contours.
- `cubic-alpha`: cubic-alpha contour field construction and adaptive contouring.

The crate does not provide filled contours, contour labels, irregular or
scattered-data triangulation, Kuhn simplices, N-component grids, or C1 cubic
field continuity. SVG is the preferred publication-quality output. PNG
supersampling is an example/output helper, not a permanent chart API.

## Architecture and contribution

Architecture notes are under [docs/architecture](docs/architecture/README.md).
The editable contour knowledge base is under
[docs/knowledge-base](docs/knowledge-base/README.md). Contributions are
welcome; see [CONTRIBUTING.md](CONTRIBUTING.md). The crate is dual licensed
under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
