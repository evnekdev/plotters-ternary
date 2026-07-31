# Milestone implementation roadmap

This roadmap is structured for implementation with Codex or another coding agent. Each milestone should be independently reviewable, keep the crate compiling, and end with objective evidence: tests, examples, generated artefacts, or all three.

The implementation must not assume that all three triangle corners or edges are visible. Rectangular logical viewports and cropped Gibbs triangles are foundational rather than a later add-on.

## Working method

For every milestone:

1. Start from a dedicated branch named `milestone/<number>-<topic>`.
2. Ask Codex to read the architecture notes and relevant ADRs before editing.
3. Keep the change limited to the milestone acceptance criteria.
4. Require `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features` where supported.
5. Add or update examples whenever a public capability becomes visible.
6. Commit generated example outputs only when they are intentional reference artefacts.
7. Open a draft pull request and review API shape before merging.

Generated images are part of the acceptance evidence. Reference outputs should live under:

```text
examples/output/
    png/
    svg/
```

Example programs should live under `examples/` and generate both raster and vector output where the backend supports it.

## Milestone 0: Plotters integration spike

### Objective

Prove that the selected Plotters architecture works before fixing public signatures.

### Implementation

- Add the Plotters dependency and choose a supported version range.
- Add minimal PNG and SVG backend features.
- Create an experimental Cartesian chart containing an equilateral triangle.
- Verify wrapping or owning the required `ChartContext` and `DrawingArea` lifetimes.
- Verify ordinary Plotters captions, margins, series annotations and legends around the triangle.
- Test composable anchored elements.
- Record native text-rotation and clipping limitations.
- Decide whether the spike can be evolved directly or should remain private.

### Tests

- Crate compiles with the selected backend feature combinations.
- Minimal example executes successfully for PNG and SVG.

### Visual artefacts

No polished output is required, but retain temporary outputs during review if they expose integration constraints.

### Exit criteria

- The Cartesian-backed design is confirmed or ADR 0001 is revised.
- The intended ownership/lifetime pattern is documented.
- Caption and legend support is demonstrated.

## Milestone 1: Geometry, compositions and projection

### Objective

Implement a backend-independent ternary geometry kernel.

### Implementation

- `TernaryPoint`.
- validation and normalisation policies;
- `Component`, `VertexOrder` and triangle orientation types;
- forward ternary-to-Cartesian projection;
- reverse Cartesian-to-ternary projection;
- point classification for valid, invalid and outside-triangle points;
- tolerance policy used consistently by geometry operations.

### Tests

- all three pure corners;
- edge and centre compositions;
- unnormalised compositions under each policy;
- all supported vertex orders;
- forward/reverse round trips;
- invalid and near-boundary inputs.

Property-based tests are desirable for normalised random compositions.

### Visual artefacts

None required. This milestone is intentionally numerical.

### Exit criteria

- Geometry has no dependency on `DrawingBackend`.
- Reverse projection reproduces valid compositions within a documented tolerance.
- Public naming is stable enough for the viewport layer.

## Milestone 2: Rectangular viewport and clipping kernel

### Objective

Make full, cropped and interior ternary views first-class geometry operations.

### Implementation

- `TernaryViewport` with full and Cartesian constructors;
- viewport validation;
- aspect-preserving logical-to-pixel transform model;
- viewport fit and alignment policies;
- segment clipping against a rectangle, preferably Liang-Barsky;
- visible triangle-edge calculation;
- grid-isoline segment construction;
- reverse mapping from viewport coordinates;
- optional helper constructors such as viewport around a composition or fitted to points.

### Tests

- full triangle;
- right-side crop;
- left-side crop;
- top crop;
- crop containing one corner;
- crop containing no corners or edges;
- completely external viewport;
- horizontal, vertical and diagonal segment clipping;
- lines entering and leaving through every rectangle side;
- no distortion under aspect-preserving fitting.

### Visual artefacts

A diagnostic example may draw the viewport rectangle, triangle and clipped segments, but the rectangle must be explicitly enabled as a debug overlay rather than treated as a chart feature.

### Exit criteria

- Cropping is mathematical, not post-render image cropping.
- The viewport boundary is invisible by default in subsequent rendering milestones.
- The clipping kernel is backend-independent and well tested.

## Milestone 3: First rendered ternary chart

### Objective

Deliver the first user-visible chart abstraction and the first permanent generated images.

### Implementation

- `TernaryChartBuilder`;
- Cartesian-backed `TernaryChart`;
- full and cropped triangle boundary rendering;
- major ternary grid lines;
- basic A/B/C corner names;
- basic component-axis names;
- Plotters caption and margin support;
- escape hatch to the underlying Cartesian chart or plotting area;
- invisible rectangular viewport clipping for chart-owned primitives.

### Examples

Create:

- `examples/full_triangle.rs`;
- `examples/cropped_right.rs`;
- `examples/interior_view.rs`.

Each example should generate PNG and SVG outputs.

### Required reference artefacts

```text
examples/output/png/full_triangle.png
examples/output/svg/full_triangle.svg
examples/output/png/cropped_right.png
examples/output/svg/cropped_right.svg
examples/output/png/interior_view.png
examples/output/svg/interior_view.svg
```

The interior example must contain no visible triangular edge and no visible rectangular frame, only clipped internal grid lines.

### Tests

- mesh geometry and visible boundary segments;
- viewport boundary remains absent from normal drawing commands;
- examples execute without panic;
- SVG output contains expected primitive categories or stable identifying text.

### Exit criteria

- A full triangle can be produced using only the high-level ternary API.
- Captions remain outside the clipped ternary viewport.
- Cropped and interior views are visually demonstrated in raster and vector formats.

## Milestone 4: Lines, points and Plotters legends

### Objective

Support the main data-series workflows while retaining ordinary Plotters annotations and legends.

### Implementation

- ternary polyline projection;
- segment clipping without prematurely discarding off-screen vertices;
- `TernaryLineSeries`;
- visible-subpath splitting;
- `TernaryPointSeries`;
- closure-based marker elements;
- marker clip modes, initially `Centre` and `None`;
- integration with Plotters `SeriesAnno` and `configure_series_labels`;
- Plotters-native `ShapeStyle`, colours and marker styles.

### Examples

Create or extend examples to demonstrate:

- several styled curves;
- a line whose endpoints are outside the viewport but whose middle crosses it;
- circles, crosses, triangles and custom marker closures;
- normal Plotters legends next to a full triangle;
- a legend next to a cropped triangle.

### Required reference artefacts

At minimum:

```text
examples/output/png/lines_points_legend.png
examples/output/svg/lines_points_legend.svg
examples/output/png/cropped_crossing_series.png
examples/output/svg/cropped_crossing_series.svg
```

### Tests

- off-screen endpoints yield correct visible intersections;
- multiple visible subpaths are retained;
- invalid compositions produce documented errors or omissions;
- marker closures receive the expected anchor coordinate;
- series annotation is returned and legends render.

### Exit criteria

- Users can draw scientifically useful datasets.
- No separate ternary legend API is introduced.
- Raster and vector outputs demonstrate parity of content.

## Scientific marker extension: custom scientific markers

### Objective

Extend `TernaryPointSeries` with portable fillable and partitioned scientific
markers while preserving Plotters-native `SeriesAnno` annotations and legends.

### Implementation

- centred local marker geometry for ellipses, polygonal shapes, stars, four
  triangle directions, and stroke-only plus/cross/asterisk symbols;
- independent fill and edge styling, empty contours, solid fills, and common
  outer-edge draw order;
- weighted radial, two-way linear/diagonal, and four-quadrant partitions;
- concrete `MarkerElement` usable in point series and ordinary legend closures;
- source-indexed per-point marker-style providers for experimental phase
  combinations;
- permanent PNG/SVG marker gallery generated through the existing optional
  bitmap-quality helper and native SVG path.

### Exit criteria

- the legacy `.style(...).marker(MarkerShape::Circle)` API remains valid;
- normal Plotters legend configuration is retained, with every marker centred
  in the shared fixed symbol slot;
- PNG and SVG receive equivalent local marker geometry, with no SVG raster
  images;
- marker geometry remains separate from interpolation, ternary coordinates,
  viewport clipping, phase regions, and annotations.

## Milestone 5: Publication-quality axes and mesh

### Objective

Provide independently configurable ternary axes suitable for publications.

### Implementation

- independent A/B/C axis configuration;
- `TickSpec` supporting count, step and explicit values;
- major and minor ticks;
- major and minor grid styles;
- decimal, percentage and custom label formatters;
- tick direction, length and label offset;
- axis and corner font/style configuration;
- visible-edge tick filtering;
- `TickRangeMode::FullCompositionRange` and `VisibleRange`;
- corner-label visibility policies;
- `CroppedAxisPolicy::TriangleEdgesOnly`;
- manual axis-name and label placement for cropped views;
- collision-avoidance rules for duplicate endpoint labels.

Automatic relocation of missing axes should remain deferred until manual placement is stable.

### Examples

Create:

- a full triangle with dense minor grid lines;
- an asymmetric axis configuration;
- a percentage-labelled composition chart;
- a right-side crop with only visible-edge ticks;
- an interior view with manually placed component labels.

### Required reference artefacts

At minimum:

```text
examples/output/png/custom_axes.png
examples/output/svg/custom_axes.svg
examples/output/png/cropped_axes.png
examples/output/svg/cropped_axes.svg
```

### Tests

- tick resolution for count, step and explicit modes;
- formatter behaviour;
- visible-range calculation;
- no tick is emitted on an invisible triangle-edge fragment;
- custom labels and Unicode chemical formulae are retained in SVG.

### Exit criteria

- Axis configuration covers the originally requested major/minor steps, labels and A/B/C names.
- Cropped axes remain ternary axes; the viewport rectangle never becomes a Cartesian frame.
- Example outputs are acceptable as initial publication-style figures.

## Milestone 6: Polygons, regions and text annotations

### Objective

Support labelled phase regions and other common scientific annotations.

### Implementation

- rectangle clipping for polygons, preferably Sutherland-Hodgman;
- `TernaryPolygon` with fill and border styles;
- `TernaryText` with position, offset and anchor;
- quarter-turn text rotation through Plotters where supported;
- explicit capability reporting for unsupported arbitrary-angle rotation;
- annotation clipping modes;
- stricter marker-bounds clipping where practical;
- layout tests for long labels and chemical formulae.

### Examples

- filled phase fields;
- labelled regions;
- annotations near and beyond viewport boundaries;
- cropped filled polygons;
- Unicode scientific text and subscripts.

### Required reference artefacts

```text
examples/output/png/regions_annotations.png
examples/output/svg/regions_annotations.svg
examples/output/png/cropped_regions.png
examples/output/svg/cropped_regions.svg
```

### Tests

- polygon clipping across every viewport side;
- concave input behaviour is either supported and tested or rejected explicitly;
- annotation policies behave consistently;
- SVG retains vector polygons and text rather than rasterising the whole chart.

### Exit criteria

- Phase-region diagrams can be produced without dropping to raw Cartesian coordinates.
- Text limitations are documented rather than silently approximated.

### Implemented first release

Milestone 6 implements simple convex and concave `TernaryPolygon` loops with
strict self-intersection rejection, Sutherland–Hodgman viewport clipping,
independent fill/border styles, and native Plotters annotations/legends.
`TernaryText` supports owned Unicode content, composition anchors, final-pixel
offsets, portable alignment, `Anchor`/`None` clipping, and native quarter turns.
Bounds clipping, callouts, general arbitrary-angle annotations, holes, and
filled contours remain explicitly deferred.

## Milestone 7: Regular-grid line contours

Phase 2 extraction moved the complete backend-independent Milestone 7 pipeline
to `ternary-contours`. `plotters-ternary` directly re-exports the primary core
API and retains only projection, visual clipping, Plotters elements, legends,
and backend rendering. The local path dependency is intentional until both
crates complete publication compatibility review.

### Objective

Add backend-independent isolines on a regular two-dimensional ternary lattice, with linear and modular cubic-alpha interpolation.

### Implemented scope

- canonical `i+j+k=n` scalar-field ordering and checked conversions;
- internally generated upward/downward elementary triangles and unique directed edges;
- deterministic linear marching triangles and path joining;
- `spline1d` Akima, MAKIMA, PCHIP, and Steffen single-interval alpha coefficients;
- RawBarycentric, conventional symmetric Muggianu, and Kohler binary extrapolation policies;
- analytic local and global reduced gradients;
- bounded adaptive barycentric topology extraction with diagnostics;
- optional arc-length redistribution and implicit-level projection;
- `ContourSet`, `ContourLevel`, `ContourPath`, and native-legend `TernaryContourSeries`;
- complete-path construction before existing rectangular viewport clipping.

Irregular triangulation, Delaunay, arbitrary meshes, Kuhn simplices, N-component grids, filled contours, and surfaces are explicitly excluded.

### Reference artefacts

```text
examples/output/png/linear_contours.png
examples/output/svg/linear_contours.svg
examples/output/png/cubic_alpha_contours.png
examples/output/svg/cubic_alpha_contours.svg
examples/output/png/cropped_contours.png
examples/output/svg/cropped_contours.svg
```

### Exit criteria

- Numerical interpolation and topology remain independent of Plotters.
- Alpha convention and reversal are proven behaviorally against `spline1d`.
- All three extrapolation policies are explicit and tested.
- Full and cropped PNG/SVG examples preserve native legends and vector geometry.
- Limitations and diagnostics are documented in [contour-kernel.md](contour-kernel.md).

## Milestone 8: Release-quality API and documentation

### Objective

Prepare the first publishable crate version around the stable core.

### Implementation

- crate-level documentation;
- `prelude` module;
- comprehensive examples linked from docs;
- error type review;
- feature-flag review;
- MSRV decision and CI matrix;
- rustdoc links and examples;
- README with generated image gallery;
- changelog and dual licence files if selected;
- API naming review for consistency with Plotters;
- removal or sealing of experimental internals.

### Visual artefacts

Use the generated PNG images in the README gallery and link SVG versions for inspection. The gallery should include at least:

- full triangle;
- trimmed side view;
- interior viewport;
- lines, points and legend;
- custom axes;
- regions and annotations;
- contours if Milestone 7 is included in the first release.

### Exit criteria

- all documented examples build and execute;
- generated outputs are reproducible;
- public API has no known milestone-blocking lifetime or ownership issues;
- first release scope is explicitly declared.

## Milestone 9: Advanced contour styling and labels

### Objective

Add publication-oriented contour styling and chart-space labels without changing the final numerical geometry supplied by `ternary-contours`.

### Implemented scope

- uniform, ordered, callback and continuous per-level `ShapeStyle` policies;
- native Plotters legend entries for all, selected or every nth contour level;
- horizontal and vertical continuous colour bars with ticks, labels and units;
- automatic one-per-component tangent labels;
- repeated and semantic manual label anchors;
- per-glyph curved labels along projected arc length;
- deterministic endpoint, viewport, curvature and label-collision rejection;
- configurable final-resolution text style, normal offset and halo;
- native transformed SVG text and final-resolution antialiased PNG text.

Label placement is deliberately a final chart-space calculation. It may project and display-clip temporary paths, but it never mutates, resamples, reconnects or projects the numerical `ContourSet` back to an isolevel. Filled contours, chart-wide annotation collision solving, interactive label dragging and general text-on-path remain out of scope. See [contour-rendering.md](contour-rendering.md).

### Reference artefacts

```text
examples/output/{png,svg}/contour_level_legend.*
examples/output/{png,svg}/contour_color_bar.*
examples/output/{png,svg}/contour_labels.*
examples/output/{png,svg}/curved_contour_labels.*
examples/output/{png,svg}/cropped_contour_labels.*
examples/output/{png,svg}/manual_contour_labels.*
examples/output/{png,svg}/repeated_contour_labels.*
```

## Milestone 10 and later: advanced capabilities

These should be implemented only after the core API has real usage experience:

- automatic relocation of missing cropped axes;
- arbitrary-angle vector text;
- lightweight built-in mathematical text;
- optional LaTeX or Typst renderer;
- tie-line and phase-field helpers;
- interpolation adapters;
- filled contours;
- reverse-coordinate interaction and hit testing;
- GUI/backend-specific interaction helpers.

Each advanced feature should have its own ADR when it introduces an external process, backend-specific behaviour or a significant public abstraction.

## Suggested release cuts

### Internal prototype

After Milestone 3:

- geometry;
- rectangular viewport;
- first full and cropped images.

### Usable alpha (`0.1.0-alpha`)

After Milestone 5:

- chart, mesh, lines, points, legends and publication-quality axes;
- full, side-cropped and interior examples in PNG and SVG.

### First broadly useful release (`0.1.0`)

After Milestone 6 or 7, depending on whether contours are considered mandatory for the initial audience.

### Experimental modules

Contours, math text and interactive helpers may be feature-gated or clearly marked experimental until their contracts stabilise.

## Codex task template

Each Codex implementation request should include:

```text
Read docs/architecture/README.md and all linked architecture notes and ADRs.
Implement Milestone N from docs/architecture/roadmap.md only.
Do not expand scope beyond its acceptance criteria without documenting the reason.
Keep numerical geometry independent of Plotters backends.
Use Plotters-native styles and legend machinery.
Add unit tests and required examples.
Generate the specified PNG and SVG reference outputs.
Run formatting, clippy and tests.
Summarise public API changes, assumptions, generated artefacts and unresolved risks.
```

## Documentation baseline for the first release

The first publishable release should explain:

- component ordering and normalisation;
- full versus rectangularly trimmed viewports;
- invisible viewport clipping;
- axis and corner naming;
- tick count versus tick step configuration;
- lines, points, polygons, text and legends;
- supported raster and vector backends;
- clipping semantics;
- text rotation and mathematical-text limitations;
- how to reproduce every gallery image.

### Milestone 5 implementation status

Milestone 5 is implemented with independent semantic axis configuration,
true-edge-only cropped ticks, minor grids, deterministic endpoint ownership,
and split geometry/text draw phases. Advanced cropped-axis relocation and
general text collision resolution remain deferred.
