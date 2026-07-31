#[cfg(feature = "cubic-alpha")]
use std::collections::BTreeMap;

use crate::coord::TernaryPoint;
#[cfg(feature = "cubic-alpha")]
use crate::interpolation::{AlphaInterval, CubicAlphaTriangle, DirectedAlphaInterval};

use super::{ContourError, RegularTernaryScalarField};
#[cfg(feature = "cubic-alpha")]
use super::{
    CubicAlphaMethod, CubicAlphaOptions, CubicBoundaryPolicy, CubicContourDiagnostics,
    regular_grid::{GridEdgeKey, GridTriangle, GridVertexId, LatticeCoordinate},
};

#[derive(Clone, Copy, Debug)]
pub(crate) struct LocatedValue {
    pub value: f64,
    /// Gradient with respect to global semantic `(a,b)`, with `c=1-a-b`.
    pub gradient_ab: [f64; 2],
}

pub(crate) fn locate_linear(
    field: &RegularTernaryScalarField,
    point: TernaryPoint,
) -> Result<LocatedValue, ContourError> {
    locate_with(field, point, |triangle, barycentric| {
        let [v0, v1, v2] = triangle.vertices;
        let values = [field.value(v0)?, field.value(v1)?, field.value(v2)?];
        let value = dot(values, barycentric);
        Ok((value, local_linear_gradient(values)))
    })
}

fn locate_with(
    field: &RegularTernaryScalarField,
    point: TernaryPoint,
    mut evaluate: impl FnMut(
        super::regular_grid::GridTriangle,
        [f64; 3],
    ) -> Result<(f64, [f64; 2]), ContourError>,
) -> Result<LocatedValue, ContourError> {
    for triangle in field.triangles()? {
        let [v0, v1, v2] = triangle.vertices;
        let vertices = [
            field.composition(v0)?,
            field.composition(v1)?,
            field.composition(v2)?,
        ];
        let Some(barycentric) = barycentric_in_triangle(point, vertices, 1.0e-10) else {
            continue;
        };
        let (value, local_gradient) = evaluate(triangle, barycentric)?;
        let gradient_ab = local_to_global_gradient(local_gradient, vertices);
        return Ok(LocatedValue { value, gradient_ab });
    }
    let [a, b, c] = point.as_array();
    Err(ContourError::PointOutsideGrid { a, b, c })
}

pub(crate) fn barycentric_in_triangle(
    point: TernaryPoint,
    vertices: [TernaryPoint; 3],
    tolerance: f64,
) -> Option<[f64; 3]> {
    let [a, b, _] = point.as_array();
    let [a0, b0, _] = vertices[0].as_array();
    let [a1, b1, _] = vertices[1].as_array();
    let [a2, b2, _] = vertices[2].as_array();
    let da0 = a0 - a2;
    let da1 = a1 - a2;
    let db0 = b0 - b2;
    let db1 = b1 - b2;
    let det = da0 * db1 - da1 * db0;
    if det == 0.0 {
        return None;
    }
    let pa = a - a2;
    let pb = b - b2;
    let u = (pa * db1 - da1 * pb) / det;
    let v = (da0 * pb - pa * db0) / det;
    let w = 1.0 - u - v;
    if [u, v, w]
        .into_iter()
        .all(|value| value >= -tolerance && value <= 1.0 + tolerance)
    {
        Some([snap(u, tolerance), snap(v, tolerance), snap(w, tolerance)])
    } else {
        None
    }
}

fn snap(value: f64, tolerance: f64) -> f64 {
    if value.abs() <= tolerance {
        0.0
    } else if (1.0 - value).abs() <= tolerance {
        1.0
    } else {
        value
    }
}
fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left[0] * right[0] + left[1] * right[1] + left[2] * right[2]
}
fn local_linear_gradient(values: [f64; 3]) -> [f64; 2] {
    [values[0] - values[2], values[1] - values[2]]
}
fn local_to_global_gradient(local: [f64; 2], vertices: [TernaryPoint; 3]) -> [f64; 2] {
    let [a0, b0, _] = vertices[0].as_array();
    let [a1, b1, _] = vertices[1].as_array();
    let [a2, b2, _] = vertices[2].as_array();
    let da0 = a0 - a2;
    let da1 = a1 - a2;
    let db0 = b0 - b2;
    let db1 = b1 - b2;
    let det = da0 * db1 - da1 * db0;
    [
        (local[0] * db1 - db0 * local[1]) / det,
        (da0 * local[1] - local[0] * da1) / det,
    ]
}

#[cfg(feature = "cubic-alpha")]
pub(crate) struct CubicGridField<'a> {
    field: &'a RegularTernaryScalarField,
    triangles: Vec<GridTriangle>,
    models: Vec<CubicAlphaTriangle>,
    diagnostics: CubicContourDiagnostics,
}

#[cfg(feature = "cubic-alpha")]
impl<'a> CubicGridField<'a> {
    pub fn new(
        field: &'a RegularTernaryScalarField,
        options: CubicAlphaOptions,
    ) -> Result<Self, ContourError> {
        let mut diagnostics = CubicContourDiagnostics::default();
        let intervals = build_edge_intervals(field, options, &mut diagnostics)?;
        let triangles = field.triangles()?;
        let mut models = Vec::with_capacity(triangles.len());
        for triangle in &triangles {
            let [v0, v1, v2] = triangle.vertices;
            let values = [field.value(v0)?, field.value(v1)?, field.value(v2)?];
            let pairs = [(0, 1), (1, 2), (0, 2)];
            let mut directed = Vec::with_capacity(3);
            for (left, right) in pairs {
                let key = GridEdgeKey::new(triangle.vertices[left], triangle.vertices[right]);
                let interval = *intervals
                    .get(&key)
                    .ok_or(crate::interpolation::InterpolationError::MissingEdgePair)?;
                let (start, end) = if key.start == triangle.vertices[left] {
                    (left, right)
                } else {
                    (right, left)
                };
                directed.push(DirectedAlphaInterval::new(start, end, interval)?);
            }
            models.push(CubicAlphaTriangle::new(
                values,
                directed.try_into().expect("three pairs"),
                options.extrapolation,
            )?);
        }
        Ok(Self {
            field,
            triangles,
            models,
            diagnostics,
        })
    }

    pub fn diagnostics(&self) -> &CubicContourDiagnostics {
        &self.diagnostics
    }
    pub fn diagnostics_mut(&mut self) -> &mut CubicContourDiagnostics {
        &mut self.diagnostics
    }
    pub fn triangles(&self) -> &[GridTriangle] {
        &self.triangles
    }
    pub fn triangle_vertices(&self, index: usize) -> Result<[TernaryPoint; 3], ContourError> {
        {
            let [v0, v1, v2] = self.triangles[index].vertices;
            Ok([
                self.field.composition(v0)?,
                self.field.composition(v1)?,
                self.field.composition(v2)?,
            ])
        }
    }
    pub fn value_in_triangle(&self, index: usize, barycentric: [f64; 3]) -> f64 {
        self.models[index].value(barycentric)
    }

    pub fn locate(&self, point: TernaryPoint) -> Result<LocatedValue, ContourError> {
        locate_with(self.field, point, |triangle, barycentric| {
            let model = &self.models[triangle.id];
            Ok((
                model.value(barycentric),
                model.gradient_reduced(barycentric[0], barycentric[1]),
            ))
        })
    }
}

#[cfg(feature = "cubic-alpha")]
fn build_edge_intervals(
    field: &RegularTernaryScalarField,
    options: CubicAlphaOptions,
    diagnostics: &mut CubicContourDiagnostics,
) -> Result<BTreeMap<GridEdgeKey, AlphaInterval>, ContourError> {
    let n = field.subdivisions();
    let mut lines: Vec<Vec<GridVertexId>> = Vec::new();
    for fixed in 0..=n {
        lines.push(
            (0..=n - fixed)
                .map(|i| {
                    field.vertex_id(LatticeCoordinate {
                        i,
                        j: n - fixed - i,
                        k: fixed,
                    })
                })
                .collect::<Result<_, _>>()?,
        );
        lines.push(
            (0..=n - fixed)
                .map(|i| {
                    field.vertex_id(LatticeCoordinate {
                        i,
                        j: fixed,
                        k: n - fixed - i,
                    })
                })
                .collect::<Result<_, _>>()?,
        );
        lines.push(
            (0..=n - fixed)
                .map(|j| {
                    field.vertex_id(LatticeCoordinate {
                        i: fixed,
                        j,
                        k: n - fixed - j,
                    })
                })
                .collect::<Result<_, _>>()?,
        );
    }
    let mut result = BTreeMap::new();
    for line in lines {
        if line.len() < 2 {
            continue;
        }
        let values = line
            .iter()
            .map(|id| field.value(*id))
            .collect::<Result<Vec<_>, _>>()?;
        for interval_index in 0..line.len() - 1 {
            let mut alpha = if line.len() < 3 {
                match options.boundary_policy {
                    CubicBoundaryPolicy::LinearFallback => {
                        diagnostics.linear_fallback_edges += 1;
                        AlphaInterval::default()
                    }
                    CubicBoundaryPolicy::Error => {
                        return Err(ContourError::InsufficientStencil {
                            samples: line.len(),
                        });
                    }
                }
            } else {
                diagnostics.cubic_edges += 1;
                alpha_for_interval(options.method, &values, interval_index)
            };
            let key = GridEdgeKey::new(line[interval_index], line[interval_index + 1]);
            if key.start != line[interval_index] {
                alpha = alpha.reversed();
            }
            result.insert(key, alpha);
        }
    }
    debug_assert_eq!(result.len(), field.edge_count()?);
    Ok(result)
}

#[cfg(feature = "cubic-alpha")]
fn alpha_for_interval(method: CubicAlphaMethod, values: &[f64], index: usize) -> AlphaInterval {
    use spline1d::{cubic_single_left_alpha, cubic_single_middle_alpha, cubic_single_right_alpha};
    let kind = method_kind(method);
    let alpha = if index == 0 {
        cubic_single_left_alpha(kind, 0.0, values[0], 1.0, values[1], 2.0, values[2])
    } else if index + 1 == values.len() - 1 {
        let base = index - 1;
        cubic_single_right_alpha(
            kind,
            base as f64,
            values[base],
            index as f64,
            values[index],
            (index + 1) as f64,
            values[index + 1],
        )
    } else {
        cubic_single_middle_alpha(
            kind,
            (index - 1) as f64,
            values[index - 1],
            index as f64,
            values[index],
            (index + 1) as f64,
            values[index + 1],
            (index + 2) as f64,
            values[index + 2],
        )
    };
    AlphaInterval::new(alpha[0], alpha[1])
}

#[cfg(feature = "cubic-alpha")]
fn method_kind(method: CubicAlphaMethod) -> spline1d::InterpolationType<f64> {
    match method {
        CubicAlphaMethod::Akima => spline1d::InterpolationType::AKIMA,
        CubicAlphaMethod::Makima => spline1d::InterpolationType::MAKIMA,
        CubicAlphaMethod::Pchip => spline1d::InterpolationType::PCHIP,
        CubicAlphaMethod::Steffen => spline1d::InterpolationType::STEFFEN,
    }
}

#[cfg(all(test, feature = "cubic-alpha"))]
mod tests {
    use super::*;
    use spline1d::{cubic_single_left, cubic_single_middle, cubic_single_right};

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "{a} != {b}");
    }
    fn direct(coeff: [f64; 4], dx: f64) -> f64 {
        ((coeff[0] * dx + coeff[1]) * dx + coeff[2]) * dx + coeff[3]
    }
    fn g(x: f64) -> f64 {
        0.3 * x.powi(3) - 0.8 * x * x + (0.7 * x).sin() + 2.0
    }

    #[test]
    fn every_method_direct_and_alpha_left_middle_right_are_equivalent() {
        let xs = [-1.7, -0.2, 1.1, 3.4, 5.2];
        let ys = xs.map(g);
        for method in [
            CubicAlphaMethod::Akima,
            CubicAlphaMethod::Makima,
            CubicAlphaMethod::Pchip,
            CubicAlphaMethod::Steffen,
        ] {
            let kind = method_kind(method);
            let cases = [
                (
                    cubic_single_left(kind, xs[0], ys[0], xs[1], ys[1], xs[2], ys[2]),
                    AlphaInterval::new(
                        {
                            let a = spline1d::cubic_single_left_alpha(
                                kind, xs[0], ys[0], xs[1], ys[1], xs[2], ys[2],
                            );
                            a[0]
                        },
                        {
                            let a = spline1d::cubic_single_left_alpha(
                                kind, xs[0], ys[0], xs[1], ys[1], xs[2], ys[2],
                            );
                            a[1]
                        },
                    ),
                    xs[0],
                    xs[1],
                    ys[0],
                    ys[1],
                ),
                (
                    cubic_single_middle(
                        kind, xs[0], ys[0], xs[1], ys[1], xs[2], ys[2], xs[3], ys[3],
                    ),
                    AlphaInterval::new(
                        {
                            let a = spline1d::cubic_single_middle_alpha(
                                kind, xs[0], ys[0], xs[1], ys[1], xs[2], ys[2], xs[3], ys[3],
                            );
                            a[0]
                        },
                        {
                            let a = spline1d::cubic_single_middle_alpha(
                                kind, xs[0], ys[0], xs[1], ys[1], xs[2], ys[2], xs[3], ys[3],
                            );
                            a[1]
                        },
                    ),
                    xs[1],
                    xs[2],
                    ys[1],
                    ys[2],
                ),
                (
                    cubic_single_right(kind, xs[2], ys[2], xs[3], ys[3], xs[4], ys[4]),
                    AlphaInterval::new(
                        {
                            let a = spline1d::cubic_single_right_alpha(
                                kind, xs[2], ys[2], xs[3], ys[3], xs[4], ys[4],
                            );
                            a[0]
                        },
                        {
                            let a = spline1d::cubic_single_right_alpha(
                                kind, xs[2], ys[2], xs[3], ys[3], xs[4], ys[4],
                            );
                            a[1]
                        },
                    ),
                    xs[3],
                    xs[4],
                    ys[3],
                    ys[4],
                ),
            ];
            for (coeff, alpha, x0, x1, y0, y1) in cases {
                for t in [0.0, 0.11, 0.37, 0.72, 1.0] {
                    close(direct(coeff, t * (x1 - x0)), alpha.value(y0, y1, t));
                }
            }
        }
    }

    #[test]
    fn every_edge_is_built_once_and_short_boundary_lines_report_fallbacks() {
        let n = 3;
        let count = (n + 1) * (n + 2) / 2;
        let field =
            RegularTernaryScalarField::new(n, (0..count).map(|i| g(i as f64)).collect()).unwrap();
        let mut diagnostics = CubicContourDiagnostics::default();
        let map =
            build_edge_intervals(&field, CubicAlphaOptions::default(), &mut diagnostics).unwrap();
        assert_eq!(map.len(), field.edge_count().unwrap());
        assert_eq!(diagnostics.linear_fallback_edges, 3);
        assert_eq!(
            diagnostics.cubic_edges + diagnostics.linear_fallback_edges,
            map.len()
        );
    }

    #[test]
    fn shared_edges_are_value_continuous() {
        let n = 4;
        let mut values = Vec::new();
        for i in 0..(n + 1) * (n + 2) / 2 {
            values.push(g(i as f64 * 0.37));
        }
        let field = RegularTernaryScalarField::new(n, values).unwrap();
        let model = CubicGridField::new(&field, CubicAlphaOptions::default()).unwrap();
        let triangles = model.triangles();
        for left in 0..triangles.len() {
            for right in left + 1..triangles.len() {
                let shared: Vec<_> = triangles[left]
                    .vertices
                    .into_iter()
                    .filter(|id| triangles[right].vertices.contains(id))
                    .collect();
                if shared.len() == 2 {
                    for t in [0.0, 0.2, 0.5, 0.9, 1.0] {
                        let p0 = field.composition(shared[0]).unwrap().as_array();
                        let p1 = field.composition(shared[1]).unwrap().as_array();
                        let point = TernaryPoint::new(
                            p0[0] * (1.0 - t) + p1[0] * t,
                            p0[1] * (1.0 - t) + p1[1] * t,
                            p0[2] * (1.0 - t) + p1[2] * t,
                        );
                        let bl = barycentric_in_triangle(
                            point,
                            model.triangle_vertices(left).unwrap(),
                            1e-9,
                        )
                        .unwrap();
                        let br = barycentric_in_triangle(
                            point,
                            model.triangle_vertices(right).unwrap(),
                            1e-9,
                        )
                        .unwrap();
                        close(
                            model.value_in_triangle(left, bl),
                            model.value_in_triangle(right, br),
                        );
                    }
                }
            }
        }
    }
}
