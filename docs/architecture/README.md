# plotters-ternary architecture knowledge base

This directory records the intended architecture and public API of `plotters-ternary` before and during implementation.

The documents are living design notes. They describe the current direction rather than a frozen specification. Major decisions that should retain their historical rationale are recorded separately as Architecture Decision Records (ADRs) under `docs/decisions/`.

## Goals

`plotters-ternary` should make publication-quality ternary diagrams straightforward in Rust while remaining compatible with ordinary Plotters backends, layouts, captions, legends, styles and drawing elements.

The crate should support:

- full and rectangularly trimmed Gibbs triangles;
- independently configurable ternary axes, ticks, labels, corner names and grid lines;
- lines, markers, polygons, contours, text and annotations in ternary coordinates;
- ordinary Plotters titles, subtitles, legends and surrounding layout;
- optional advanced mathematical text without making an external TeX installation a core dependency;
- reusable coordinate, clipping and contour algorithms that are independent of the drawing backend.

## Documents

- [`overview.md`](overview.md): architectural layers, responsibilities and development strategy.
- [`api-inventory.md`](api-inventory.md): proposed public types and operations.
- [`viewport-and-clipping.md`](viewport-and-clipping.md): rectangular viewports and trimmed triangles.
- [`rendering-and-series.md`](rendering-and-series.md): mesh, drawing elements, contours and text.
- [`roadmap.md`](roadmap.md): staged implementation plan and acceptance criteria.
- [`geometry-kernel.md`](geometry-kernel.md): implemented composition validation and projection.
- [`viewport-kernel.md`](viewport-kernel.md): implemented viewport fitting and clipping.
- [`chart-kernel.md`](chart-kernel.md): implemented Cartesian-backed chart and mesh.
- [`series-kernel.md`](series-kernel.md): implemented lines, points and native legends.
- [`marker-kernel.md`](marker-kernel.md): portable scientific-marker geometry, fills and partitions.
- [`bitmap-quality.md`](bitmap-quality.md): temporary optional supersampling for example PNGs.
- [`axis-kernel.md`](axis-kernel.md): implemented publication mesh, ticks, and axis labels.
- [`region-annotation-kernel.md`](region-annotation-kernel.md): implemented phase polygons and text annotations.

## Decision records

- [`0001-cartesian-backed-chart.md`](../decisions/0001-cartesian-backed-chart.md)
- [`0002-rectangular-ternary-viewports.md`](../decisions/0002-rectangular-ternary-viewports.md)

## Maintenance rules

1. Update these notes when public concepts or ownership boundaries change.
2. Add an ADR when changing a decision that affects several modules or the public API.
3. Keep numerical algorithms independent of `DrawingBackend` wherever practical.
4. Prefer Plotters-native style and annotation types over parallel wrapper types.
5. Preserve escape hatches to the underlying Plotters chart and plotting area.