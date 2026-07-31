# Milestone 10: linear filled contours and scalar maps

ternary-contours owns ContourBandSet numerical regions. Plotters rendering
takes immutable semantic rings, projects them through TernaryGeometry, clips
only for the invisible rectangular viewport, and draws native vector polygons.
It never resamples, smooths, or changes a ContourBandSet.

Bands use lower-inclusive, upper-exclusive scalar ownership; exteriors are CCW
and holes CW in semantic (a,b) coordinates. The current renderer draws
exterior rings directly; complex hole compositing is deferred until Plotters
provides a portable even-odd polygon fill path.

TernaryScalarMapSeries is rendering-only. It evaluates the exact
piecewise-linear field on deterministic microtriangles and flat-fills each
microtriangle. This avoids a mandatory raster image in SVG, at the cost of
larger SVG output at higher resolutions. Colours, opacity, scalar
normalisation, and microtriangle resolution belong to plotters-ternary.