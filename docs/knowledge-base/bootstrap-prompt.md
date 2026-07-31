# Bootstrap prompt

When reviewing or extending the contour kernel, keep these invariants intact:

1. Use only the regular two-dimensional ternary lattice in this crate.
2. Linear contours contain no alpha terms.
3. `alpha1` multiplies directed `t`; reversal is `(alpha0+alpha1, -alpha1)`.
4. The pair prefactor is always raw `xi*xj`.
5. Muggianu and Kohler are distinct interior policies but identical on binary edges.
6. Use shared canonical directed edge intervals and analytic gradients.
7. Extract cubic topology adaptively; never silently discard unresolved cells.
8. Redistribute then project back to the same global field.
9. Keep numerical code independent of Plotters; render and viewport-clip only afterwards.
