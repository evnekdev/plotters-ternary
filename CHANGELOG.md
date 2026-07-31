# Changelog

All notable changes are documented here.

## 0.1.0 — 2026-07-31

First release of `plotters-ternary`.

- Backend-independent ternary composition, projection, viewport, and clipping kernels.
- Cartesian-backed Plotters charts with cropped ternary views, publication axes, captions, and native legends.
- Lines, smooth series, scientific markers, polygons, and composition-anchored text.
- Regular-grid piecewise-linear contours and optional cubic-alpha contours using Akima, MAKIMA, PCHIP, and Steffen edge intervals.
- Muggianu and Kohler binary extrapolation policies; RawBarycentric is experimental only.
- Per-level contour styling, discrete legends, continuous colour bars, and tangent, curved, repeated, and manually placed contour labels.
- Piecewise-linear filled contour bands with transparent holes, stepped colour bars, and continuous scalar-map rendering.
- Geometry-only PNG supersampling and vector SVG output.

Known limitations: cubic-alpha filled bands are not available; no irregular/scattered
triangulation, Kuhn simplices, or N-component fields are provided. Cubic-alpha fields
are C0 rather than C1 across grid edges, and adaptive cubic topology is bounded and
reports maximum-depth diagnostics.
