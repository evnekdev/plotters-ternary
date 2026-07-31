# ADR 0003: Keep RawBarycentric experimental

- Status: accepted
- Date: 2026-07-31

## Context

The cubic-alpha field has three possible ways to extend a directed binary
interval parameter into a ternary interior. Conventional Muggianu and Kohler
have established symmetric/reversal-invariant interior semantics. The direct
`RawBarycentric` extension uses `t = xj`, where `j` is the canonical directed
edge destination.

Unlike Muggianu and Kohler, reversing an ordinary binary interval does not
preserve RawBarycentric's interior pair value when the third component is
nonzero. Its output is deterministic only because the regular grid assigns one
canonical direction to every edge.

## Decision

`BinaryExtrapolation::RawBarycentric` remains public for 0.1.0 only as an
**experimental, non-recommended research comparison mode**. Its documentation
must state that it is neither linear interpolation nor conventional Muggianu,
and that canonical direction is part of its model. The default remains
`Muggianu`; stable applications should choose `Muggianu` or `Kohler`.

## Consequences

The variant is retained so users can reproduce and audit the direct extension,
but it is not promoted in the README, prelude examples, or recommended API
paths. Future evidence may justify a dedicated experimental feature or removal
in a breaking release.
