# Milestone 3 chart kernel

## Public chart types

Milestone 3 adds the following Plotters-backed types at the crate root:

- `TernaryChartBuilder<'root, DB>`;
- `TernaryChart<'series, DB>`;
- `TernaryMeshConfig<'chart, 'series, 'font, DB>`;
- `TernaryChartError<E>`;
- `CartesianChartContext<'series, DB>` and `CartesianPlottingArea<DB>` aliases for escape-hatch return types.

The compact construction path is:

```rust,no_run
use plotters::prelude::*;
use plotters_ternary::{TernaryChartBuilder, TernaryGeometry, TernaryViewport};

let root = SVGBackend::new("chart.svg", (1000, 800)).into_drawing_area();
root.fill(&WHITE)?;
let geometry = TernaryGeometry::default();
let mut chart = TernaryChartBuilder::on(&root)
    .caption("Ternary diagram", ("sans-serif", 28))
    .margin(40)
    .geometry(geometry)
    .viewport(TernaryViewport::full(geometry))
    .build()?;
chart.configure_mesh()
    .major_step(0.1)
    .axis_a_name("A")
    .axis_b_name("B")
    .axis_c_name("C")
    .corner_a_name("Pure A")
    .corner_b_name("Pure B")
    .corner_c_name("Pure C")
    .draw()?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

Builder defaults are conventional upward geometry, an implicit full viewport resolved from the final geometry at build time, `PreserveAspect`, centred alignment, a 20-pixel margin, and no viewport frame. Calling `viewport` makes the selection explicit; a later `geometry` call never replaces it.

## Ownership and lifetimes

`TernaryChart` owns
`ChartContext<'series, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>`.
Plotters' builder clones the fitted drawing area into the context, so the returned chart does not store a reference to the root or to a temporary layout area. `TernaryChartBuilder::build` chooses the chart's independent `'series` lifetime (subject to `DB: 'series`); that lifetime is reserved for Plotters series-annotation closures in Milestone 4. No allocation is leaked, no unsafe lifetime extension is used, and the chart does not store a duplicate plotting area.

`cartesian_chart`, `cartesian_chart_mut`, and `plotting_area` provide narrow access to ordinary Plotters operations. The internal coordinate type is made readable through public aliases without exposing layout implementation structs.

## Layout and aspect preservation

Milestone 3 uses the fitted plotting-subarea strategy. Caption layout is performed by Plotters' `DrawingArea::titled`, then equal margins are applied. `ViewportTransform` fits the requested logical viewport into the remaining pixel allocation. Its floating bounds are rounded once at the rendering boundary, and `ChartBuilder` is created on that fitted `DrawingArea::shrink` subarea.

Consequently, the Cartesian X and Y ranges remain exactly
`viewport.x_min()..viewport.x_max()` and
`viewport.y_min()..viewport.y_max()`. The requested viewport is therefore both the internal logical range and the mathematical clipping window; no expanded display range is introduced. Under `PreserveAspect`, equal logical X/Y scales are retained to within at most one pixel of integer layout rounding. Alignment controls where unused allocated space is placed. `Stretch` deliberately uses the full allocation and may distort the triangle.

The fitted and requested rectangles remain invisible. The only rectangle normally present in generated SVG examples is the explicitly filled outer background.

## Clipping and mesh generation

Every chart-owned line follows the kernel pipeline:

```text
full triangle edge or component isoline
    -> logical Cartesian geometry
    -> mathematical clipping to TernaryViewport
    -> Plotters PathElement in the surviving logical coordinates
```

Triangle boundaries use `geometry.visible_edges`; their identities remain geometric `TriangleEdge` identities. Major grid families use `geometry.visible_component_isoline` for semantic components A, B, and C. Plotters plotting-area coordinate truncation is not used as geometric clipping.

A common major step defaults to `0.1`. It must be finite and in `(0, 1]`, and it may generate at most 10,000 intervals. Values are calculated as `i * step` with an integer index. Zero and one are excluded, so grid isolines never duplicate the outer boundary and the degenerate `k = 1` isoline is not drawn.

`TernaryMeshConfig` accepts Plotters `ShapeStyle` values for boundary and grid lines, a Plotters-compatible shared text style, separate semantic A/B/C axis names and corner names, and independent hide switches for boundary, grid, axis names, and corner names.

## Basic name policy

Corner names belong to semantic components and use `geometry.vertex(component)`, so custom `VertexOrder` values remain correct. A corner name is drawn only when that vertex is contained by the requested viewport within the chart tolerance.

An axis name uses the midpoint of the full edge opposite its semantic component. It is drawn only when that normal anchor is in the requested viewport. Names are horizontal. Missing labels are not relocated to viewport sides or to clipped edge fragments. Advanced tick, rotation, and cropped-axis placement policy remains Milestone 5 work.

## Error strategy

`TernaryChartError<DB::ErrorType>` keeps backend-independent `coord::Error` intact while representing Plotters' `DrawingAreaErrorKind`, an invalid major step, and a drawing allocation made unusable by caption/margins. It implements `From` for geometry and drawing errors, so normal `?` propagation remains concise without putting backend types into `coord::Error`.

## Reference examples

Three examples share one backend-generic renderer and generate each format directly:

- `examples/full_triangle.rs` -> `examples/output/{png,svg}/full_triangle.*`;
- `examples/cropped_right.rs` -> `examples/output/{png,svg}/cropped_right.*`;
- `examples/interior_view.rs` -> `examples/output/{png,svg}/interior_view.*`.

All use deterministic 1000 by 800 dimensions. The interior viewport lies wholly inside the Gibbs triangle and therefore emits no triangle boundary or automatic names, only clipped internal grid lines and its external caption.

## Carry-forward

Milestone 4 must add ternary-aware data-series adapters that project and mathematically clip into owned Plotters elements, delegate to this owned context, and return native `SeriesAnno` so normal legends remain available. Milestone 5 must add independent axis steps, ticks and labels, visible-edge tick filtering, better label offsets, and explicit cropped-axis policies. Text remains horizontally rendered because Plotters' portable vector text rotations are limited to quarter turns.
