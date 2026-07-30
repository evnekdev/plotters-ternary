# Development roadmap

The implementation should be staged so that early code does not assume all three triangle corners or edges are visible.

## Phase 0: dependency and Plotters spike

Before fixing public signatures:

- add the Plotters dependency and select a supported version range;
- build a minimal Cartesian-backed triangle example for SVG and bitmap output;
- verify how `ChartContext`, `DrawingArea`, `draw_series` and `SeriesAnno` lifetimes compose in a wrapper;
- verify access to caption and legend layout;
- test composable anchored elements;
- document native text rotation limits;
- determine whether Plotters provides usable clipping primitives or whether mathematical clipping is required throughout.

Deliverable: a private experimental module or examples proving the integration approach.

## Phase 1: geometry and viewport foundation

Implement:

- `TernaryPoint`;
- validation and normalisation;
- component and vertex-order types;
- forward and reverse projection;
- `TernaryViewport`;
- aspect-preserving logical-to-pixel mapping;
- point classification;
- segment clipping;
- unit tests for all transforms and crop positions.

Acceptance criteria:

- full triangle maps correctly;
- arbitrary vertex order is tested;
- right, left, top and interior viewports map without distortion;
- reverse projection reproduces valid compositions within tolerance.

## Phase 2: chart and basic mesh

Implement:

- `TernaryChartBuilder`;
- Cartesian-backed `TernaryChart`;
- full and cropped triangle boundary rendering;
- major component grid lines;
- basic axis and corner labels;
- space for ordinary Plotters captions, margins and legends;
- SVG and bitmap examples.

Acceptance criteria:

- viewport boundary is invisible by default;
- captions and legends remain outside the clipped ternary region;
- a fully interior viewport can display grid lines without drawing a frame;
- a full-triangle example requires only high-level ternary API calls.

## Phase 3: line and point series

Implement:

- ternary polyline projection and clipping;
- `TernaryLineSeries`;
- `TernaryPointSeries` and closure-based marker elements;
- Plotters series annotation and legend integration;
- marker clip modes, initially at least `Centre` and `None`;
- examples with off-screen lines crossing the viewport.

Acceptance criteria:

- off-screen endpoints produce correct visible intersections;
- multiple visible subpaths are handled;
- normal Plotters legends work without a separate legend API;
- marker styles use ordinary Plotters types.

## Phase 4: publication-quality axes

Implement:

- independent A/B/C axis configuration;
- major and minor tick counts, steps and explicit values;
- major and minor grid styles;
- decimal, percentage and custom formatters;
- tick direction, length and label offsets;
- visible-edge tick filtering;
- `TickRangeMode`;
- corner-label visibility policies;
- `CroppedAxisPolicy::TriangleEdgesOnly` and manual placement.

Automatic relocation of missing axes can follow after manual placement is stable.

## Phase 5: polygons, annotations and stricter clipping

Implement:

- polygon clipping;
- `TernaryPolygon` for phase regions;
- text annotations with offsets and anchors;
- annotation clipping modes;
- stricter marker bounds clipping where feasible;
- layout tests for long labels and chemical formulas.

## Phase 6: line contours

Implement:

- triangular field representation;
- marching-triangle isolines;
- degeneracy rules;
- path joining;
- `ContourSet` and `TernaryContourSeries`;
- contour level styling and optional labels.

Acceptance criteria:

- regular and irregular triangular meshes are supported;
- contours cross viewport boundaries correctly;
- contour calculation is usable without Plotters.

## Phase 7: advanced text and scientific helpers

Explore and optionally implement:

- arbitrary-angle text;
- lightweight mathematical text;
- optional LaTeX or Typst renderer;
- tie-line and phase-field helpers;
- interpolation adapters;
- filled contours;
- reverse-coordinate interaction and hit testing.

## API stability strategy

- Keep version 0.1 focused on geometry, chart, mesh, lines and points.
- Mark experimental contour and math-text modules clearly until their topology and backend contracts stabilise.
- Prefer additive extension over premature generic abstractions.
- Reserve public names only when their semantics are clear.
- Add examples for every major public workflow before publishing.

## Documentation deliverables

The first publishable release should include:

- crate-level overview;
- full triangle example;
- right-side cropped example;
- interior zoom example;
- custom axes and tick example;
- lines, points, text and legend example;
- explanation of component ordering and normalisation;
- documented clipping and unsupported text capabilities.