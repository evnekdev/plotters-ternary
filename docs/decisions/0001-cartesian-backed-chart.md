# ADR 0001: Use a Cartesian-backed Plotters chart

- Status: Accepted for initial implementation
- Date: 2026-07-30

## Context

`plotters-ternary` needs a ternary-coordinate API while retaining ordinary Plotters functionality such as drawing backends, captions, legends, styles, series annotations and surrounding layout.

Plotters supports custom coordinate translation, but its most mature high-level chart and mesh workflows are centred on Cartesian chart contexts. Beginning with a fully custom coordinate specification risks coupling the project to complicated Plotters generic and lifetime behaviour before the public ternary API is validated.

## Decision

The initial high-level `TernaryChart` will wrap an ordinary Cartesian Plotters `ChartContext`.

Public users provide ternary compositions. The crate projects them into a two-dimensional plane and delegates the resulting elements to Plotters.

The wrapper will expose deliberate access to the underlying Cartesian chart and plotting area for advanced use.

## Consequences

### Positive

- Immediate compatibility with existing Plotters backends and styles.
- Ordinary captions, legends and series annotations can be preserved.
- Ternary geometry and clipping remain testable independently of Plotters.
- The project can deliver a useful API before implementing a custom Plotters coordinate system.
- Advanced users retain an escape hatch for unsupported annotations.

### Negative

- The wrapper must carefully manage Plotters generic types and lifetimes.
- Some ternary-specific elements need adapters before they can participate in `draw_series`.
- The internal Cartesian coordinate range is an implementation detail that must not leak unnecessarily.
- A custom coordinate specification may still be desirable for future direct `DrawingArea` integration or interaction.

## Alternatives considered

### Fully custom Plotters coordinate specification from the start

Rejected for the initial implementation because it raises integration risk without proving a better high-level API.

### Independent rendering framework

Rejected because it would duplicate Plotters backends, styles, layout and legend functionality.

### Pre-render the ternary diagram as an image

Rejected because it would lose vector quality, element composition and ordinary Plotters series behaviour.

## Revisit conditions

Reconsider this decision if:

- the Cartesian wrapper prevents normal Plotters legends or series annotations;
- a custom coordinate implementation materially simplifies the public API;
- interactive backends require direct reverse-coordinate integration unavailable through the wrapper;
- Plotters adds stronger first-class support for non-Cartesian chart contexts.