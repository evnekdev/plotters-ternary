use std::fmt;

use plotters::backend::DrawingBackend;
use plotters::element::{Drawable, PointCollection};
use plotters::style::ShapeStyle;
use plotters_backend::{BackendCoord, DrawingErrorKind};

use crate::coord::{
    Normalization, TernaryCartesian, TernaryGeometry, TernaryPoint, TernaryViewport, Tolerance,
};

/// A polygon in semantic ternary-composition coordinates.
///
/// Open and explicitly closed input sequences are both accepted.  Valid simple
/// concave polygons are supported; self-intersecting polygons are rejected so
/// rendering never depends on a backend-specific fill rule.
pub struct TernaryPolygon<I> {
    points: I,
    fill: Option<ShapeStyle>,
    border: Option<ShapeStyle>,
    normalization: Normalization,
    tolerance: Tolerance,
}

impl<I> TernaryPolygon<I> {
    /// Construct a strict unit-sum polygon with no fill or border.
    pub fn new(points: I) -> Self {
        Self {
            points,
            fill: None,
            border: None,
            normalization: Normalization::RequireUnitSum,
            tolerance: Tolerance::default(),
        }
    }

    /// Fill the final clipped polygon with an independent Plotters style.
    pub fn fill_style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.fill = Some(style.into());
        self
    }

    /// Draw one independent border around the final clipped polygon.
    pub fn border_style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.border = Some(style.into());
        self
    }

    /// Select explicit composition validation or normalisation.
    pub const fn normalization(mut self, normalization: Normalization) -> Self {
        self.normalization = normalization;
        self
    }

    /// Select the numerical tolerance used for preparation and clipping.
    pub const fn tolerance(mut self, tolerance: Tolerance) -> Self {
        self.tolerance = tolerance;
        self
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        I,
        Option<ShapeStyle>,
        Option<ShapeStyle>,
        Normalization,
        Tolerance,
    ) {
        (
            self.points,
            self.fill,
            self.border,
            self.normalization,
            self.tolerance,
        )
    }
}

/// A backend-neutral polygon after validation, projection, and viewport clipping.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedPolygon {
    vertices: Vec<TernaryCartesian>,
}

impl PreparedPolygon {
    /// Return the final open vertex loop. The closing edge is implicit.
    pub fn vertices(&self) -> &[TernaryCartesian] {
        &self.vertices
    }

    /// Return whether clipping removed all positive-area geometry.
    pub const fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

/// Failures specific to polygon validation and preparation.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum PolygonError {
    /// Fewer than three distinct vertices remain after tolerance cleanup.
    TooFewVertices { distinct: usize },
    /// A source composition could not be validated or projected.
    InvalidPoint {
        index: usize,
        source: crate::coord::Error,
    },
    /// The source loop has effectively zero signed area.
    Degenerate,
    /// Two non-adjacent source edges intersect or touch.
    SelfIntersection {
        first_edge: usize,
        second_edge: usize,
    },
}

impl fmt::Display for PolygonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewVertices { distinct } => write!(
                f,
                "polygon needs at least three distinct vertices; found {distinct}"
            ),
            Self::InvalidPoint { index, source } => write!(
                f,
                "invalid ternary polygon point at index {index}: {source}"
            ),
            Self::Degenerate => write!(f, "polygon has effectively zero area"),
            Self::SelfIntersection {
                first_edge,
                second_edge,
            } => write!(f, "polygon edges {first_edge} and {second_edge} intersect"),
        }
    }
}

impl std::error::Error for PolygonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPoint { source, .. } => Some(source),
            Self::TooFewVertices { .. } | Self::Degenerate | Self::SelfIntersection { .. } => None,
        }
    }
}

/// Validate, project, and mathematically clip a simple ternary polygon.
pub fn prepare_polygon<I, P>(
    points: I,
    geometry: TernaryGeometry,
    viewport: TernaryViewport,
    normalization: Normalization,
    tolerance: Tolerance,
) -> Result<PreparedPolygon, PolygonError>
where
    I: IntoIterator<Item = P>,
    P: Into<TernaryPoint>,
{
    let mut projected = Vec::new();
    for (index, point) in points.into_iter().enumerate() {
        let point = geometry
            .project(point.into(), normalization, tolerance)
            .map_err(|source| PolygonError::InvalidPoint { index, source })?;
        push_distinct(&mut projected, point, tolerance);
    }
    if projected.len() > 1 && same_point(projected[0], *projected.last().unwrap(), tolerance) {
        projected.pop();
    }
    validate_source_polygon(&projected, tolerance)?;

    let clipped = clip_polygon_to_viewport(projected, viewport, tolerance);
    if clipped.len() < 3 || signed_area(&clipped).abs() <= tolerance.absolute {
        return Ok(PreparedPolygon {
            vertices: Vec::new(),
        });
    }
    Ok(PreparedPolygon { vertices: clipped })
}

fn validate_source_polygon(
    vertices: &[TernaryCartesian],
    tolerance: Tolerance,
) -> Result<(), PolygonError> {
    if vertices.len() < 3 {
        return Err(PolygonError::TooFewVertices {
            distinct: vertices.len(),
        });
    }
    for first in 0..vertices.len() {
        let first_end = (first + 1) % vertices.len();
        for second in (first + 1)..vertices.len() {
            let second_end = (second + 1) % vertices.len();
            if first == second || first_end == second || second_end == first {
                continue;
            }
            if segments_intersect(
                vertices[first],
                vertices[first_end],
                vertices[second],
                vertices[second_end],
                tolerance,
            ) {
                return Err(PolygonError::SelfIntersection {
                    first_edge: first,
                    second_edge: second,
                });
            }
        }
    }
    if signed_area(vertices).abs() <= tolerance.absolute {
        return Err(PolygonError::Degenerate);
    }
    Ok(())
}

fn clip_polygon_to_viewport(
    mut vertices: Vec<TernaryCartesian>,
    viewport: TernaryViewport,
    tolerance: Tolerance,
) -> Vec<TernaryCartesian> {
    for boundary in [
        Boundary::Left(viewport.x_min()),
        Boundary::Right(viewport.x_max()),
        Boundary::Bottom(viewport.y_min()),
        Boundary::Top(viewport.y_max()),
    ] {
        vertices = clip_against_boundary(&vertices, boundary, tolerance);
        if vertices.is_empty() {
            break;
        }
    }
    vertices
}

#[derive(Clone, Copy)]
enum Boundary {
    Left(f64),
    Right(f64),
    Bottom(f64),
    Top(f64),
}

impl Boundary {
    fn is_inside(self, point: TernaryCartesian, tolerance: Tolerance) -> bool {
        match self {
            Self::Left(value) => point.x >= value - tolerance.absolute,
            Self::Right(value) => point.x <= value + tolerance.absolute,
            Self::Bottom(value) => point.y >= value - tolerance.absolute,
            Self::Top(value) => point.y <= value + tolerance.absolute,
        }
    }
    fn intersection(self, from: TernaryCartesian, to: TernaryCartesian) -> TernaryCartesian {
        match self {
            Self::Left(x) | Self::Right(x) => {
                let t = (x - from.x) / (to.x - from.x);
                TernaryCartesian::new(x, from.y + t * (to.y - from.y))
            }
            Self::Bottom(y) | Self::Top(y) => {
                let t = (y - from.y) / (to.y - from.y);
                TernaryCartesian::new(from.x + t * (to.x - from.x), y)
            }
        }
    }
}

fn clip_against_boundary(
    vertices: &[TernaryCartesian],
    boundary: Boundary,
    tolerance: Tolerance,
) -> Vec<TernaryCartesian> {
    let mut output = Vec::new();
    let Some(mut previous) = vertices.last().copied() else {
        return output;
    };
    let mut previous_inside = boundary.is_inside(previous, tolerance);
    for current in vertices.iter().copied() {
        let current_inside = boundary.is_inside(current, tolerance);
        if current_inside != previous_inside {
            push_distinct(
                &mut output,
                boundary.intersection(previous, current),
                tolerance,
            );
        }
        if current_inside {
            push_distinct(&mut output, current, tolerance);
        }
        previous = current;
        previous_inside = current_inside;
    }
    if output.len() > 1 && same_point(output[0], *output.last().unwrap(), tolerance) {
        output.pop();
    }
    output
}

fn signed_area(vertices: &[TernaryCartesian]) -> f64 {
    vertices
        .iter()
        .zip(vertices.iter().cycle().skip(1))
        .take(vertices.len())
        .map(|(a, b)| a.x * b.y - a.y * b.x)
        .sum::<f64>()
        * 0.5
}

fn push_distinct(
    points: &mut Vec<TernaryCartesian>,
    point: TernaryCartesian,
    tolerance: Tolerance,
) {
    if points
        .last()
        .is_none_or(|previous| !same_point(*previous, point, tolerance))
    {
        points.push(point);
    }
}

fn same_point(left: TernaryCartesian, right: TernaryCartesian, tolerance: Tolerance) -> bool {
    tolerance.is_close(left.x, right.x) && tolerance.is_close(left.y, right.y)
}

fn cross(a: TernaryCartesian, b: TernaryCartesian, c: TernaryCartesian) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn on_segment(
    a: TernaryCartesian,
    b: TernaryCartesian,
    point: TernaryCartesian,
    tolerance: Tolerance,
) -> bool {
    cross(a, b, point).abs() <= tolerance.absolute
        && point.x >= a.x.min(b.x) - tolerance.absolute
        && point.x <= a.x.max(b.x) + tolerance.absolute
        && point.y >= a.y.min(b.y) - tolerance.absolute
        && point.y <= a.y.max(b.y) + tolerance.absolute
}

fn segments_intersect(
    a: TernaryCartesian,
    b: TernaryCartesian,
    c: TernaryCartesian,
    d: TernaryCartesian,
    tolerance: Tolerance,
) -> bool {
    let ab_c = cross(a, b, c);
    let ab_d = cross(a, b, d);
    let cd_a = cross(c, d, a);
    let cd_b = cross(c, d, b);
    if (ab_c > tolerance.absolute && ab_d < -tolerance.absolute
        || ab_c < -tolerance.absolute && ab_d > tolerance.absolute)
        && (cd_a > tolerance.absolute && cd_b < -tolerance.absolute
            || cd_a < -tolerance.absolute && cd_b > tolerance.absolute)
    {
        return true;
    }
    on_segment(a, b, c, tolerance)
        || on_segment(a, b, d, tolerance)
        || on_segment(c, d, a, tolerance)
        || on_segment(c, d, b, tolerance)
}

/// An owned Plotters element that fills first and draws one final polygon border.
pub(crate) struct PolygonElement<Coord> {
    vertices: Vec<Coord>,
    fill: Option<ShapeStyle>,
    border: Option<ShapeStyle>,
}

impl<Coord> PolygonElement<Coord> {
    pub(crate) fn new(
        vertices: Vec<Coord>,
        fill: Option<ShapeStyle>,
        border: Option<ShapeStyle>,
    ) -> Self {
        Self {
            vertices,
            fill,
            border,
        }
    }
}

impl<'a, Coord> PointCollection<'a, Coord> for &'a PolygonElement<Coord> {
    type Point = &'a Coord;
    type IntoIter = std::slice::Iter<'a, Coord>;
    fn point_iter(self) -> Self::IntoIter {
        self.vertices.iter()
    }
}

impl<Coord, DB: DrawingBackend> Drawable<DB> for PolygonElement<Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        points: I,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        let coordinates: Vec<_> = points.collect();
        if coordinates.len() < 3 {
            return Ok(());
        }
        if let Some(style) = self.fill {
            backend.fill_polygon(coordinates.iter().copied(), &style)?;
        }
        if let Some(style) = self.border {
            let mut path = coordinates;
            path.push(path[0]);
            backend.draw_path(path, &style)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::{Component, TriangleOrientation, VertexOrder};

    fn point(a: f64, b: f64, c: f64) -> TernaryPoint {
        TernaryPoint::new(a, b, c)
    }
    fn prepared(
        points: impl IntoIterator<Item = TernaryPoint>,
        viewport: TernaryViewport,
    ) -> Result<PreparedPolygon, PolygonError> {
        prepare_polygon(
            points,
            TernaryGeometry::default(),
            viewport,
            Normalization::RequireUnitSum,
            Tolerance::default(),
        )
    }

    #[test]
    fn accepts_open_closed_and_simple_concave_source_loops() {
        let viewport = TernaryViewport::full(TernaryGeometry::default());
        let open = prepared(
            [
                point(0.2, 0.7, 0.1),
                point(0.2, 0.1, 0.7),
                point(0.7, 0.1, 0.2),
            ],
            viewport,
        )
        .unwrap();
        let closed = prepared(
            [
                point(0.2, 0.7, 0.1),
                point(0.2, 0.1, 0.7),
                point(0.7, 0.1, 0.2),
                point(0.2, 0.7, 0.1),
            ],
            viewport,
        )
        .unwrap();
        assert_eq!(open, closed);
        let concave = prepared(
            [
                point(0.1, 0.8, 0.1),
                point(0.1, 0.1, 0.8),
                point(0.35, 0.325, 0.325),
                point(0.6, 0.1, 0.3),
                point(0.6, 0.3, 0.1),
            ],
            viewport,
        )
        .unwrap();
        assert_eq!(concave.vertices().len(), 5);
    }

    #[test]
    fn rejects_degenerate_and_self_intersecting_loops() {
        let viewport = TernaryViewport::full(TernaryGeometry::default());
        assert!(matches!(
            prepared(
                [
                    point(0.5, 0.5, 0.0),
                    point(0.4, 0.6, 0.0),
                    point(0.3, 0.7, 0.0)
                ],
                viewport
            ),
            Err(PolygonError::Degenerate)
        ));
        assert!(matches!(
            prepared(
                [
                    point(0.2, 0.7, 0.1),
                    point(0.6, 0.1, 0.3),
                    point(0.2, 0.1, 0.7),
                    point(0.6, 0.3, 0.1),
                ],
                viewport
            ),
            Err(PolygonError::SelfIntersection { .. })
        ));
    }

    #[test]
    fn clips_enclosing_and_external_polygons_without_backend_clamping() {
        let viewport = TernaryViewport::new(0.35, 0.65, 0.15, 0.45).unwrap();
        let enclosed = prepared(
            [
                point(1.0, 0.0, 0.0),
                point(0.0, 1.0, 0.0),
                point(0.0, 0.0, 1.0),
            ],
            viewport,
        )
        .unwrap();
        assert_eq!(enclosed.vertices().len(), 4);
        assert!(
            enclosed
                .vertices()
                .iter()
                .all(|point| viewport.contains(*point, Tolerance::default()).unwrap())
        );
        let outside = prepared(
            [
                point(0.0, 1.0, 0.0),
                point(0.0, 0.9, 0.1),
                point(0.1, 0.9, 0.0),
            ],
            viewport,
        )
        .unwrap();
        assert!(outside.is_empty());
    }

    #[test]
    fn clipping_handles_each_side_adjacent_opposite_and_boundary_cases() {
        let viewport = TernaryViewport::new(0.0, 1.0, 0.0, 1.0).unwrap();
        let tolerance = Tolerance::default();
        let cases = [
            vec![
                cart(-0.2, 0.2),
                cart(0.4, 0.2),
                cart(0.4, 0.6),
                cart(-0.2, 0.6),
            ],
            vec![
                cart(0.6, 0.2),
                cart(1.2, 0.2),
                cart(1.2, 0.6),
                cart(0.6, 0.6),
            ],
            vec![
                cart(0.2, -0.2),
                cart(0.8, -0.2),
                cart(0.8, 0.3),
                cart(0.2, 0.3),
            ],
            vec![
                cart(0.2, 0.7),
                cart(0.8, 0.7),
                cart(0.8, 1.2),
                cart(0.2, 1.2),
            ],
            vec![
                cart(-0.2, 0.7),
                cart(0.4, 0.7),
                cart(0.4, 1.2),
                cart(-0.2, 1.2),
            ],
            vec![
                cart(-0.2, 0.35),
                cart(1.2, 0.35),
                cart(1.2, 0.55),
                cart(-0.2, 0.55),
            ],
            vec![
                cart(0.0, 0.2),
                cart(0.5, 0.2),
                cart(0.5, 0.7),
                cart(0.0, 0.7),
            ],
        ];
        for subject in cases {
            let clipped = clip_polygon_to_viewport(subject, viewport, tolerance);
            assert!(clipped.len() >= 3);
            assert!(
                clipped
                    .iter()
                    .all(|point| viewport.contains(*point, tolerance).unwrap())
            );
            assert!(
                clipped
                    .windows(2)
                    .all(|pair| !same_point(pair[0], pair[1], tolerance))
            );
        }

        let corner_tangent = clip_polygon_to_viewport(
            vec![cart(1.0, 1.0), cart(1.3, 1.1), cart(1.1, 1.3)],
            viewport,
            tolerance,
        );
        assert!(
            corner_tangent.len() < 3 || signed_area(&corner_tangent).abs() <= tolerance.absolute
        );
    }

    #[test]
    fn semantic_input_remains_valid_under_custom_order_and_downward_orientation() {
        let points = [
            point(0.65, 0.20, 0.15),
            point(0.30, 0.50, 0.20),
            point(0.20, 0.20, 0.60),
        ];
        let order = VertexOrder::new(Component::C, Component::A, Component::B).unwrap();
        for orientation in [TriangleOrientation::Up, TriangleOrientation::Down] {
            let geometry = TernaryGeometry::new(orientation, order);
            let prepared = prepare_polygon(
                points,
                geometry,
                TernaryViewport::full(geometry),
                Normalization::RequireUnitSum,
                Tolerance::default(),
            )
            .unwrap();
            assert_eq!(prepared.vertices().len(), 3);
            for vertex in prepared.vertices() {
                let recovered = geometry.unproject(*vertex, Tolerance::default()).unwrap();
                assert!(points.iter().any(|source| {
                    source
                        .as_array()
                        .into_iter()
                        .zip(recovered.as_array())
                        .all(|(expected, actual)| Tolerance::default().is_close(expected, actual))
                }));
            }
        }
    }

    const fn cart(x: f64, y: f64) -> TernaryCartesian {
        TernaryCartesian::new(x, y)
    }
}
