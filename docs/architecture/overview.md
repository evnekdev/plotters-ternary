# Architectural overview

## Purpose

`plotters-ternary` is a high-level extension layer for Plotters that exposes ternary composition coordinates while reusing Plotters for rendering, layout, backends, styles, captions, legends and annotations.

The crate is not intended to fork Plotters or introduce a separate drawing backend. It should translate ternary-domain concepts into ordinary two-dimensional Plotters operations.

## Core architectural rule

Every ternary primitive is defined in the full ternary domain, projected into a common two-dimensional plane, clipped against an optional rectangular viewport, and then delegated to Plotters for rendering.

```text
ternary composition
    -> geometry validation and normalisation
    -> ternary-to-plane projection
    -> triangle and viewport clipping
    -> Plotters Cartesian coordinates
    -> drawing backend
```

## Layers

### 1. Domain and geometry

Owns:

- `TernaryPoint` and component access;
- normalisation and validation policies;
- vertex order and triangle orientation;
- forward and reverse ternary projection;
- component isolines and triangle edges;
- point classification.

This layer must not depend on a Plotters backend.

### 2. Viewport and clipping

Owns:

- rectangular logical viewports in projected ternary space;
- aspect-preserving mapping into a pixel rectangle;
- line, polyline and polygon clipping;
- visibility tests;
- visible fragments of original triangle edges;
- reverse mapping for future interaction and hit testing.

The rectangular viewport is normally invisible. It is a logical clipping and zoom window, not a Cartesian axis frame.

### 3. Chart integration

Owns:

- `TernaryChartBuilder`;
- `TernaryChart` as a wrapper around a Cartesian Plotters chart context;
- allocation of space for captions, subtitles, legends and margins;
- access to the underlying Plotters context and plotting area;
- coordination of geometry, viewport and rendering configuration.

The initial implementation should use a normal Cartesian Plotters chart internally. A custom Plotters coordinate specification may be explored later, but should not be required for the first stable API.

### 4. Mesh and axes

Owns:

- triangle border rendering;
- per-component major and minor grid lines;
- per-axis tick generation and formatting;
- component axis names and A/B/C corner labels;
- visible-edge tick placement under cropping;
- policies for cropped charts where edges or corners are missing.

Composition axes, triangle boundary edges and viewport boundaries are separate concepts. Viewport sides must not automatically become visible axes.

### 5. Elements and series

Owns ternary-aware adapters for:

- line segments and polylines;
- markers and point series;
- polygons and phase regions;
- text and annotations;
- contour paths;
- generic Plotters elements anchored at ternary coordinates.

Series should integrate with Plotters' normal series annotation and legend machinery.

### 6. Scientific algorithms

Owns backend-independent numerical operations such as:

- marching-triangle contour extraction;
- joining contour segments into paths;
- regular or irregular triangular mesh representation;
- future filled-contour topology;
- optional interpolation helpers.

Contour calculation and contour drawing must remain separate.

### 7. Optional text extensions

Owns:

- arbitrary-angle text strategy;
- mathematical text abstraction;
- optional LaTeX, Typst or lightweight built-in math rendering;
- rendered-text caching.

Ordinary Unicode and Plotters text remain the baseline. External text engines must be optional features.

## Compatibility principles

1. Reuse Plotters types such as `ShapeStyle`, `TextStyle`, `FontDesc`, colours and series annotations.
2. Preserve ordinary Plotters captions, legends, surrounding drawing areas and backend choice.
3. Do not hide all Plotters internals: expose deliberate escape hatches.
4. Start with `f64` for public geometry to keep tolerances and clipping predictable.
5. Accept tuples and arrays through conversions, but use `TernaryPoint` as the principal domain type.
6. Treat full triangles as the default viewport, not as a separate rendering path.

## Suggested module boundaries

```text
src/
    lib.rs
    prelude.rs
    error.rs

    coord/
        point.rs
        geometry.rs
        transform.rs
        validation.rs

    viewport/
        viewport.rs
        mapping.rs
        clipping.rs

    chart/
        builder.rs
        context.rs
        layout.rs

    mesh/
        axis.rs
        ticks.rs
        labels.rs
        grid.rs

    element/
        line.rs
        point.rs
        polygon.rs
        text.rs
        mapped.rs

    series/
        line.rs
        points.rs
        contours.rs

    contour/
        field.rs
        triangulation.rs
        marching_triangles.rs
        path_joining.rs

    text/
        rotation.rs
        math.rs
```

This is a target organisation, not a requirement to create every module immediately.