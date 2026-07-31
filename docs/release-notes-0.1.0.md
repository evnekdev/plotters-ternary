# Draft 0.1.0 release notes

`plotters-ternary` 0.1.0 provides publication-oriented ternary charts for Plotters. It includes validated A/B/C composition geometry, invisible rectangular viewports with mathematical clipping, configurable axes, ordinary Plotters legends, line and point series, scientific markers, polygons, and composition-anchored text.

Regular-grid numerical contours are supplied by the companion `ternary-contours` crate. This crate renders those final paths without changing their numerical coordinates. Users can add per-level styles, legends, colour bars, tangent or curved labels, piecewise-linear filled bands, and continuous piecewise-linear scalar maps.

The `cubic-alpha` feature enables cubic-alpha line contours. It does not enable cubic-alpha filled bands. Irregular grids, filled cubic bands, N-component fields, and arbitrary-angle general text layout remain outside this release.

This is a draft for the GitHub release body only. No release, tag, or registry publication is created by this document.
