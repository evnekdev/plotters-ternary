# ADR 0002: Treat trimmed triangles as rectangular logical viewports

- Status: Accepted
- Date: 2026-07-30

## Context

Scientific ternary diagrams often show only part of a Gibbs triangle: a left or right region, a corner with other parts trimmed, a top-trimmed view, or an interior zoom containing no original triangle edge.

The displayed region is always rectangular, but that rectangle should normally be invisible. Titles, subtitles, legends and other Plotters layout elements must remain outside the clipped ternary view.

A post-render image crop would produce incorrect labels and intersections, would interfere with vector output, and would not support ordinary Plotters layout.

## Decision

A trimmed ternary chart is represented by a rectangular `TernaryViewport` in the projected ternary plane.

All ternary geometry is defined in the complete ternary domain, projected, then clipped mathematically against the viewport before drawing. The viewport boundary is not drawn and is not treated as a Cartesian axis unless a user explicitly requests a debugging frame or manually anchors labels to its sides.

The default full-triangle chart is implemented through the same viewport pipeline.

Aspect ratio is preserved by default.

## Consequences

### Positive

- Full, side-cropped, corner-cropped and interior views share one rendering model.
- Off-screen line and contour segments intersect the visible window correctly.
- Vector output remains vector output.
- Plotters captions and legends are unaffected by ternary clipping.
- The original triangle edges, component axes and viewport boundaries remain conceptually distinct.
- Reverse mapping and future interaction can use the same transform.

### Negative

- Mathematical clipping is required for lines, polygons and eventually marker bounds.
- Cropped views introduce non-trivial policies for missing axis edges and corner labels.
- Interior views may have no natural place for ticks unless labels are manually or automatically relocated.
- Aspect-preserving fitting can leave unused pixels inside the allocated chart area.

## Axis policy

The initial default is `TriangleEdgesOnly`: ticks and axis labels appear only on visible fragments of the original triangle edges.

Automatic relocation of missing axes to invisible viewport sides is a later feature. Manual placement should be supported before automatic placement is considered stable.

## Alternatives considered

### Crop the final rendered bitmap or SVG

Rejected because it removes contextual layout, gives incorrect tick and label decisions, and cannot correctly clip all element types before rendering.

### Redefine the visible rectangle as a Cartesian plot frame

Rejected because its sides are not ternary axes and drawing them would misrepresent the Gibbs geometry.

### Construct a new smaller ternary triangle for every zoom

Rejected because a cropped portion is not generally a similar ternary domain and can contain no original corner at all.

## Revisit conditions

Revisit details of the clipping implementation if Plotters gains portable clipping regions that preserve vector output and can be scoped to a sub-area without affecting captions or legends. The logical viewport model itself should remain.