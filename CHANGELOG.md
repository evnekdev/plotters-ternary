# Changelog

All notable changes are documented here.

## 0.1.0 — 2026-07-31

First release of `plotters-ternary`.

- Backend-independent ternary composition, projection, viewport, and clipping kernels.
- Cartesian-backed Plotters charts with cropped ternary views, publication axes, captions, and native legends.
- Lines, smooth series, scientific markers, polygons, and composition-anchored text.
- Regular-grid piecewise-linear contours and optional cubic-alpha contours using Akima, MAKIMA, PCHIP, and Steffen edge intervals.
- Muggianu and Kohler binary extrapolation policies; RawBarycentric is experimental only.
- PNG geometry supersampling helper and vector SVG output.

Known limitations: no filled contours or contour labels; no irregular/scattered
triangulation, Kuhn simplices, or N-component fields; cubic-alpha fields are C0
rather than C1 across grid edges; adaptive cubic topology is bounded and
reports maximum-depth diagnostics.
