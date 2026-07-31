# Milestone 9 contour rendering, legends and labels

Milestone 9 is a rendering-only layer over the final numerical `ContourSet`
owned by `ternary-contours`. It never changes a `ContourPath`, resamples a path,
projects a point back to an isolevel, reconnects components, or changes an
open/closed flag. Projection, pixel-space label placement and rectangular
viewport clipping happen only while drawing.

This gives the central invariant:

> Given one `ContourSet`, changing the backend, output dimensions, viewport,
> line style, legend policy, colour bar or PNG supersampling cannot change its
> stored numerical coordinates.

## Per-level styles and legends

`ContourStylePolicy` supports a uniform `ShapeStyle`, a cycling ordered palette,
a callback from exact scalar level to `ShapeStyle`, and a continuous normalized
RGBA colour map. `TernaryContourSeries::style` and `style_for_level` remain
source-compatible. `ordered_styles`, `color_map`, and `style_policy` add the new
paths.

`ContourLegendPolicy` can register no automatic entries, every level, selected
levels, or every nth level. Selected entries are still ordinary Plotters series
annotations, so `configure_series_labels` remains the legend mechanism. A
level formatter controls the registered label text. The final no-op annotation
returned by an automatically expanded multi-level series preserves the existing
`draw_series` return contract.

For dense level sets, `ContourColorBar` is a chart-space layout object. It
supports horizontal and vertical orientations, four plotting-area corners,
explicit ticks, a formatter, title, text style, border style and a user colour
map. Geometry and text have separate draw methods so the two-pass PNG helper can
supersample only colour-bar geometry and draw its text at final resolution.

## Portable labels

`ContourLabelConfig` separates placement, appearance and formatting.
`ContourLabelPlacement` supports one automatic label per eligible numerical
component, repeated labels at a final-pixel spacing, and manual anchors by
level, path index and complete-path arc-length fraction. `ContourLabelMode`
chooses a whole tangent-aligned label or per-glyph curved layout.

Placement is calculated after projection and display clipping because glyph
size, visible length, pixel curvature and viewport clearance are layout facts.
The implementation:

1. projects each complete semantic path without modifying it;
2. clips a temporary display polyline with the established series pipeline;
3. builds cumulative projected chord length;
4. samples deterministic candidate positions;
5. rejects insufficient length, endpoint clearance, viewport clearance and
   excessive local curvature;
6. normalizes tangents to the readable `-90 deg..=90 deg` interval;
7. scores surviving candidates by centrality and curvature;
8. reserves padded axis-aligned label envelopes to reject collisions.

Manual anchors are stable semantic anchors: `(level, path_index, arc_fraction)`
refers to the unmodified complete numerical path. They are still rejected when
the resulting label would be unreadable or outside the plotting viewport.

A white halo is enabled by default and may be recoloured or disabled. It masks
the line locally without breaking the numerical or rendered contour polyline.
The first implementation deliberately does not split the line underneath the
label. It also does not inspect arbitrary user annotations, so collision
avoidance is currently among contour labels plus deterministic viewport,
endpoint and curvature guards rather than a general chart-wide collision
solver.

## Curved text and backends

Curved mode measures each Unicode scalar with Plotters' available text metrics,
places glyph centres along projected arc length, evaluates a local tangent for
each glyph, and draws glyphs in deterministic order. Plotters does not expose
full shaping or grapheme-cluster metrics through every backend; complex scripts
and combining sequences therefore use a documented per-character approximation.

PNG uses the existing high-quality arbitrary-angle coverage-mask renderer in
the final-resolution text pass. Geometry supersampling does not change label
font size, anchor or spacing. SVG capture converts the same prepared rotated
text commands to native transformed `<text>` elements inside `ternary-text`;
labels stay selectable, searchable UTF-8 vector text and no `<image>` element is
introduced. This is an isolated enhancement of the existing SVG output helper;
ordinary Plotters backends use the portable drawing element.

## Draw ordering and limitations

Applications normally draw mesh geometry, contour series, contour labels,
colour-bar geometry/text and then Plotters legends. The exact ordering remains
under caller control. A halo is the supported line-gap strategy in this
milestone. General text-on-path, filled contours, contour labels that participate
in a chart-wide annotation collision system, interactive dragging and external
math typesetting remain out of scope.

Permanent examples are:

```text
examples/output/{png,svg}/contour_level_legend.*
examples/output/{png,svg}/contour_color_bar.*
examples/output/{png,svg}/contour_labels.*
examples/output/{png,svg}/curved_contour_labels.*
examples/output/{png,svg}/cropped_contour_labels.*
examples/output/{png,svg}/manual_contour_labels.*
examples/output/{png,svg}/repeated_contour_labels.*
```
