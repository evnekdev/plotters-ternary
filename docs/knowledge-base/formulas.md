# Formula reference

For a directed interval, with `t = 0` at the first endpoint and `t = 1` at the
second:

```text
y(t) = y0*(1-t) + y1*t + (1-t)*t*(alpha0 + alpha1*t)
```

`alpha1` multiplies `t`. Reversing direction uses:

```text
(alpha0, alpha1) -> (alpha0 + alpha1, -alpha1)
```

For a directed pair `i -> j` and remaining component `k`, the pair contribution
always keeps the raw prefactor:

```text
Eij = xi*xj*(alpha0 + alpha1*tij)
```

- Muggianu: `tij = xj + xk/2 = 1/2 + (xj-xi)/2`.
- Kohler: `tij = xj/(xi+xj)`; at `xi=xj=0`, the contribution is exactly zero.
- RawBarycentric (experimental): `tij = xj`; its interior value depends on the
  canonical direction and it is not conventional Muggianu.

Muggianu and Kohler both reproduce the source spline exactly on a binary edge
and are invariant under ordinary alpha reversal throughout the triangle.
