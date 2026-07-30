use super::{
    CartesianSegment, Component, Error, Normalization, TernaryPoint, TernaryViewport, Tolerance,
    clip_segment, clip_segment_with_parameters,
};

/// The height of a unit-side equilateral triangle in logical Cartesian space.
pub const EQUILATERAL_TRIANGLE_HEIGHT: f64 = 0.866_025_403_784_438_6;

/// A point in the logical Cartesian plane used for ternary projection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TernaryCartesian {
    /// Horizontal logical coordinate.
    pub x: f64,
    /// Vertical logical coordinate.
    pub y: f64,
}

impl TernaryCartesian {
    /// Construct a logical Cartesian point.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Whether the apex of the equilateral triangle points up or down.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TriangleOrientation {
    /// Left `(0, 0)`, right `(1, 0)`, apex `(0.5, sqrt(3)/2)`.
    #[default]
    Up,
    /// Left `(0, 0)`, right `(1, 0)`, apex `(0.5, -sqrt(3)/2)`.
    Down,
}

/// The semantic component assigned to each geometric triangle slot.
///
/// `apex` denotes the point opposite the left/right base. It is above the base
/// for [`TriangleOrientation::Up`] and below it for
/// [`TriangleOrientation::Down`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct VertexOrder {
    left: Component,
    right: Component,
    apex: Component,
}

impl VertexOrder {
    /// Construct a one-to-one mapping from components to left, right, and apex.
    ///
    /// ```
    /// use plotters_ternary::{Component, VertexOrder};
    ///
    /// let order = VertexOrder::new(Component::B, Component::C, Component::A)?;
    /// assert_eq!(order.apex(), Component::A);
    /// # Ok::<(), plotters_ternary::Error>(())
    /// ```
    pub fn new(left: Component, right: Component, apex: Component) -> Result<Self, Error> {
        if left == right || left == apex || right == apex {
            return Err(Error::InvalidVertexOrder { left, right, apex });
        }
        Ok(Self { left, right, apex })
    }

    /// Return the component at the left base vertex.
    pub const fn left(self) -> Component {
        self.left
    }

    /// Return the component at the right base vertex.
    pub const fn right(self) -> Component {
        self.right
    }

    /// Return the component at the orientation-dependent apex.
    pub const fn apex(self) -> Component {
        self.apex
    }
}

impl Default for VertexOrder {
    /// Use A at the apex, B at the left base, and C at the right base.
    fn default() -> Self {
        Self {
            left: Component::B,
            right: Component::C,
            apex: Component::A,
        }
    }
}

/// The location of a finite Cartesian point relative to the full triangle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TrianglePointLocation {
    /// Strictly within the triangle by more than the supplied tolerance.
    Inside,
    /// On an edge, a corner, or within tolerance of one.
    Boundary,
    /// Materially beyond at least one triangle edge.
    Outside,
}

/// A directed geometric edge of the canonical triangle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TriangleEdge {
    /// Directed from the left base slot to the right base slot.
    LeftRight,
    /// Directed from the right base slot to the apex slot.
    RightApex,
    /// Directed from the apex slot to the left base slot.
    ApexLeft,
}

impl TriangleEdge {
    /// All edges in directed boundary order.
    pub const ALL: [Self; 3] = [Self::LeftRight, Self::RightApex, Self::ApexLeft];
}

/// A visible edge fragment retaining its identity and source parameter range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VisibleTriangleEdge {
    pub edge: TriangleEdge,
    pub segment: CartesianSegment,
    pub parameter_start: f64,
    pub parameter_end: f64,
}

/// An equilateral ternary triangle and its component-to-vertex assignment.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct TernaryGeometry {
    orientation: TriangleOrientation,
    vertex_order: VertexOrder,
}

impl TernaryGeometry {
    /// Construct geometry with the given orientation and component order.
    pub const fn new(orientation: TriangleOrientation, vertex_order: VertexOrder) -> Self {
        Self {
            orientation,
            vertex_order,
        }
    }

    /// Return the triangle orientation.
    pub const fn orientation(self) -> TriangleOrientation {
        self.orientation
    }

    /// Return the component-to-vertex assignment.
    pub const fn vertex_order(self) -> VertexOrder {
        self.vertex_order
    }

    /// Return the logical Cartesian vertex for a semantic component.
    pub fn vertex(self, component: Component) -> TernaryCartesian {
        let [left, right, apex] = self.slot_vertices();
        if component == self.vertex_order.left {
            left
        } else if component == self.vertex_order.right {
            right
        } else {
            apex
        }
    }

    /// Return the vertices in stable semantic A/B/C order.
    pub fn vertices(self) -> [TernaryCartesian; 3] {
        Component::ALL.map(|component| self.vertex(component))
    }

    /// Return a complete directed geometric triangle edge.
    pub fn triangle_edge(self, edge: TriangleEdge) -> CartesianSegment {
        let [left, right, apex] = self.slot_vertices();
        match edge {
            TriangleEdge::LeftRight => CartesianSegment::new(left, right),
            TriangleEdge::RightApex => CartesianSegment::new(right, apex),
            TriangleEdge::ApexLeft => CartesianSegment::new(apex, left),
        }
    }

    /// Return all complete edges in directed boundary order.
    pub fn triangle_edges(self) -> [(TriangleEdge, CartesianSegment); 3] {
        TriangleEdge::ALL.map(|edge| (edge, self.triangle_edge(edge)))
    }

    /// Clip every original triangle edge and retain its identity and parameters.
    pub fn visible_edges(
        self,
        viewport: TernaryViewport,
        tolerance: Tolerance,
    ) -> Result<Vec<VisibleTriangleEdge>, Error> {
        let mut visible = Vec::with_capacity(3);
        for (edge, source) in self.triangle_edges() {
            if let Some(clipped) = clip_segment_with_parameters(source, viewport, tolerance)? {
                visible.push(VisibleTriangleEdge {
                    edge,
                    segment: clipped.segment,
                    parameter_start: clipped.parameter_start,
                    parameter_end: clipped.parameter_end,
                });
            }
        }
        Ok(visible)
    }

    /// Construct the full-triangle segment where a semantic component equals `value`.
    ///
    /// Values within tolerance of zero or one are snapped to that boundary.
    /// `value = 0` returns the opposite edge and `value = 1` returns a
    /// zero-length segment at the component vertex.
    pub fn component_isoline(
        self,
        component: Component,
        value: f64,
        tolerance: Tolerance,
    ) -> Result<CartesianSegment, Error> {
        tolerance.validate()?;
        if !value.is_finite()
            || (value < 0.0 && !tolerance.is_close(value, 0.0))
            || (value > 1.0 && !tolerance.is_close(value, 1.0))
        {
            return Err(Error::InvalidIsolineValue { value, tolerance });
        }

        let value = if tolerance.is_close(value, 0.0) {
            0.0
        } else if tolerance.is_close(value, 1.0) {
            1.0
        } else {
            value
        };
        let vertex = self.vertex(component);
        let [first_other, second_other] = component.others();
        Ok(CartesianSegment::new(
            interpolate(self.vertex(first_other), vertex, value),
            interpolate(self.vertex(second_other), vertex, value),
        ))
    }

    /// Construct and clip one semantic component isoline to a viewport.
    pub fn visible_component_isoline(
        self,
        component: Component,
        value: f64,
        viewport: TernaryViewport,
        tolerance: Tolerance,
    ) -> Result<Option<CartesianSegment>, Error> {
        clip_segment(
            self.component_isoline(component, value, tolerance)?,
            viewport,
            tolerance,
        )
    }

    /// Validate and project a composition into the normalised ternary plane.
    ///
    /// The explicit policy permits source data using a non-unit required sum.
    /// After validation, the three weights are divided by their validated sum
    /// before the barycentric weighted sum is calculated.
    ///
    /// ```
    /// use plotters_ternary::{Normalization, TernaryGeometry, TernaryPoint, Tolerance};
    ///
    /// let point = TernaryPoint::new(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0);
    /// let projected = TernaryGeometry::default().project(
    ///     point,
    ///     Normalization::RequireUnitSum,
    ///     Tolerance::default(),
    /// )?;
    /// assert!((projected.x - 0.5).abs() < 1.0e-12);
    /// # Ok::<(), plotters_ternary::Error>(())
    /// ```
    pub fn project(
        self,
        point: TernaryPoint,
        normalization: Normalization,
        tolerance: Tolerance,
    ) -> Result<TernaryCartesian, Error> {
        let point = point.validate(normalization, tolerance)?;
        let sum = point.sum();
        let unit_components = point.as_array().map(|component| component / sum);
        Ok(self.project_unit_components(unit_components))
    }

    /// Recover unit-sum A/B/C component weights from a Cartesian point.
    ///
    /// Points materially outside the full triangle return
    /// [`Error::CartesianOutsideTriangle`]. Weights whose absolute value is at
    /// most the absolute tolerance are set to zero, then the remaining weights
    /// are rescaled to sum to one. This admits small boundary round-off without
    /// silently clamping external points.
    ///
    /// ```
    /// use plotters_ternary::{TernaryCartesian, TernaryGeometry, Tolerance};
    ///
    /// let point = TernaryGeometry::default()
    ///     .unproject(TernaryCartesian::new(0.5, 0.0), Tolerance::default())?;
    /// assert_eq!(point.as_array(), [0.0, 0.5, 0.5]);
    /// # Ok::<(), plotters_ternary::Error>(())
    /// ```
    pub fn unproject(
        self,
        cartesian: TernaryCartesian,
        tolerance: Tolerance,
    ) -> Result<TernaryPoint, Error> {
        tolerance.validate()?;
        let weights = self.slot_weights(cartesian)?;
        if self.classify_weights(weights, tolerance) == TrianglePointLocation::Outside {
            return Err(Error::CartesianOutsideTriangle {
                x: cartesian.x,
                y: cartesian.y,
                tolerance,
            });
        }

        let mut weights = weights.map(|weight| {
            if tolerance.is_near_zero(weight) {
                0.0
            } else {
                weight
            }
        });
        let sum: f64 = weights.into_iter().sum();
        weights = weights.map(|weight| weight / sum);

        let mut components = [0.0; 3];
        components[self.vertex_order.left.index()] = weights[0];
        components[self.vertex_order.right.index()] = weights[1];
        components[self.vertex_order.apex.index()] = weights[2];
        TernaryPoint::from(components).validate(Normalization::RequireUnitSum, tolerance)
    }

    /// Classify a finite Cartesian point against the complete ternary triangle.
    ///
    /// Non-finite coordinates are rejected because they have no geometric
    /// location. This operation does not consider future rectangular viewports.
    pub fn classify(
        self,
        cartesian: TernaryCartesian,
        tolerance: Tolerance,
    ) -> Result<TrianglePointLocation, Error> {
        tolerance.validate()?;
        let weights = self.slot_weights(cartesian)?;
        Ok(self.classify_weights(weights, tolerance))
    }

    fn project_unit_components(self, components: [f64; 3]) -> TernaryCartesian {
        let [left, right, apex] = self.slot_vertices();
        let left_weight = components[self.vertex_order.left.index()];
        let right_weight = components[self.vertex_order.right.index()];
        let apex_weight = components[self.vertex_order.apex.index()];
        TernaryCartesian {
            x: left_weight * left.x + right_weight * right.x + apex_weight * apex.x,
            y: left_weight * left.y + right_weight * right.y + apex_weight * apex.y,
        }
    }

    fn slot_vertices(self) -> [TernaryCartesian; 3] {
        let apex_y = match self.orientation {
            TriangleOrientation::Up => EQUILATERAL_TRIANGLE_HEIGHT,
            TriangleOrientation::Down => -EQUILATERAL_TRIANGLE_HEIGHT,
        };
        [
            TernaryCartesian::new(0.0, 0.0),
            TernaryCartesian::new(1.0, 0.0),
            TernaryCartesian::new(0.5, apex_y),
        ]
    }

    fn slot_weights(self, cartesian: TernaryCartesian) -> Result<[f64; 3], Error> {
        if !cartesian.x.is_finite() || !cartesian.y.is_finite() {
            return Err(Error::NonFiniteCartesian {
                x: cartesian.x,
                y: cartesian.y,
            });
        }

        let [left, right, apex] = self.slot_vertices();
        let right_from_left = subtract(right, left);
        let apex_from_left = subtract(apex, left);
        let point_from_left = subtract(cartesian, left);
        let determinant = cross(right_from_left, apex_from_left);
        let right_weight = cross(point_from_left, apex_from_left) / determinant;
        let apex_weight = cross(right_from_left, point_from_left) / determinant;
        let left_weight = 1.0 - right_weight - apex_weight;
        Ok([left_weight, right_weight, apex_weight])
    }

    fn classify_weights(self, weights: [f64; 3], tolerance: Tolerance) -> TrianglePointLocation {
        let _ = self;
        if weights
            .into_iter()
            .any(|weight| weight < -tolerance.absolute)
        {
            TrianglePointLocation::Outside
        } else if weights
            .into_iter()
            .any(|weight| tolerance.is_near_zero(weight))
        {
            TrianglePointLocation::Boundary
        } else {
            TrianglePointLocation::Inside
        }
    }
}

fn interpolate(start: TernaryCartesian, end: TernaryCartesian, parameter: f64) -> TernaryCartesian {
    TernaryCartesian::new(
        start.x + (end.x - start.x) * parameter,
        start.y + (end.y - start.y) * parameter,
    )
}

fn subtract(left: TernaryCartesian, right: TernaryCartesian) -> TernaryCartesian {
    TernaryCartesian::new(left.x - right.x, left.y - right.y)
}

fn cross(left: TernaryCartesian, right: TernaryCartesian) -> f64 {
    left.x * right.y - left.y * right.x
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const TOLERANCE: Tolerance = Tolerance {
        absolute: 1.0e-9,
        relative: 1.0e-9,
    };
    const ASSERTION_EPSILON: f64 = 1.0e-10;

    fn unit_point(a: f64, b: f64, c: f64) -> TernaryPoint {
        TernaryPoint::new(a, b, c)
            .validate(Normalization::RequireUnitSum, TOLERANCE)
            .unwrap()
    }

    fn assert_cartesian_close(actual: TernaryCartesian, expected: TernaryCartesian) {
        assert!(
            (actual.x - expected.x).abs() < ASSERTION_EPSILON,
            "{actual:?}"
        );
        assert!(
            (actual.y - expected.y).abs() < ASSERTION_EPSILON,
            "{actual:?}"
        );
    }

    fn assert_point_close(actual: TernaryPoint, expected: TernaryPoint) {
        for component in Component::ALL {
            assert!(
                (actual.component(component) - expected.component(component)).abs()
                    < ASSERTION_EPSILON,
                "{component:?}: {actual:?} != {expected:?}"
            );
        }
    }

    fn all_orders() -> [VertexOrder; 6] {
        [
            VertexOrder::new(Component::A, Component::B, Component::C).unwrap(),
            VertexOrder::new(Component::A, Component::C, Component::B).unwrap(),
            VertexOrder::new(Component::B, Component::A, Component::C).unwrap(),
            VertexOrder::new(Component::B, Component::C, Component::A).unwrap(),
            VertexOrder::new(Component::C, Component::A, Component::B).unwrap(),
            VertexOrder::new(Component::C, Component::B, Component::A).unwrap(),
        ]
    }

    #[test]
    fn conventional_pure_corners_edges_and_centroid_project_correctly() {
        let geometry = TernaryGeometry::default();
        let height = EQUILATERAL_TRIANGLE_HEIGHT;
        for (point, expected) in [
            (
                unit_point(1.0, 0.0, 0.0),
                TernaryCartesian::new(0.5, height),
            ),
            (unit_point(0.0, 1.0, 0.0), TernaryCartesian::new(0.0, 0.0)),
            (unit_point(0.0, 0.0, 1.0), TernaryCartesian::new(1.0, 0.0)),
            (
                unit_point(0.5, 0.5, 0.0),
                TernaryCartesian::new(0.25, height / 2.0),
            ),
            (
                unit_point(0.5, 0.0, 0.5),
                TernaryCartesian::new(0.75, height / 2.0),
            ),
            (unit_point(0.0, 0.5, 0.5), TernaryCartesian::new(0.5, 0.0)),
            (
                unit_point(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0),
                TernaryCartesian::new(0.5, height / 3.0),
            ),
        ] {
            assert_cartesian_close(
                geometry
                    .project(point, Normalization::RequireUnitSum, TOLERANCE)
                    .unwrap(),
                expected,
            );
        }
    }

    #[test]
    fn vertex_orders_map_all_pure_components_and_preserve_abc_on_round_trip() {
        let interior = unit_point(0.2, 0.3, 0.5);
        for order in all_orders() {
            let geometry = TernaryGeometry::new(TriangleOrientation::Up, order);
            for component in Component::ALL {
                let mut components = [0.0; 3];
                components[component.index()] = 1.0;
                let projected = geometry
                    .project(
                        TernaryPoint::from(components),
                        Normalization::RequireUnitSum,
                        TOLERANCE,
                    )
                    .unwrap();
                assert_cartesian_close(projected, geometry.vertex(component));
            }
            let round_trip = geometry
                .unproject(
                    geometry
                        .project(interior, Normalization::RequireUnitSum, TOLERANCE)
                        .unwrap(),
                    TOLERANCE,
                )
                .unwrap();
            assert_point_close(round_trip, interior);
        }
    }

    #[test]
    fn both_orientations_have_expected_apexes_and_round_trip() {
        let point = unit_point(0.2, 0.3, 0.5);
        for (orientation, apex_y) in [
            (TriangleOrientation::Up, EQUILATERAL_TRIANGLE_HEIGHT),
            (TriangleOrientation::Down, -EQUILATERAL_TRIANGLE_HEIGHT),
        ] {
            let geometry = TernaryGeometry::new(orientation, VertexOrder::default());
            assert_cartesian_close(
                geometry.vertex(Component::A),
                TernaryCartesian::new(0.5, apex_y),
            );
            let projected = geometry
                .project(point, Normalization::RequireUnitSum, TOLERANCE)
                .unwrap();
            assert_point_close(geometry.unproject(projected, TOLERANCE).unwrap(), point);
        }
    }

    #[test]
    fn canonical_default_order_and_reverse_corners_are_semantic_abc() {
        let order = VertexOrder::default();
        assert_eq!(order.apex(), Component::A);
        assert_eq!(order.left(), Component::B);
        assert_eq!(order.right(), Component::C);

        let geometry = TernaryGeometry::default();
        for (component, vertex) in [
            (
                Component::A,
                TernaryCartesian::new(0.5, EQUILATERAL_TRIANGLE_HEIGHT),
            ),
            (Component::B, TernaryCartesian::new(0.0, 0.0)),
            (Component::C, TernaryCartesian::new(1.0, 0.0)),
        ] {
            assert_cartesian_close(geometry.vertex(component), vertex);
            let recovered = geometry.unproject(vertex, TOLERANCE).unwrap();
            for candidate in Component::ALL {
                let expected = if candidate == component { 1.0 } else { 0.0 };
                assert!((recovered.component(candidate) - expected).abs() < ASSERTION_EPSILON);
            }
        }
    }

    #[test]
    fn default_component_isolines_are_parallel_to_semantic_opposite_edges() {
        let geometry = TernaryGeometry::default();
        for component in Component::ALL {
            let isoline = geometry
                .component_isoline(component, 0.4, TOLERANCE)
                .unwrap();
            let [first, second] = component.others();
            let edge = CartesianSegment::new(geometry.vertex(first), geometry.vertex(second));
            assert!(
                cross(
                    subtract(isoline.end, isoline.start),
                    subtract(edge.end, edge.start),
                )
                .abs()
                    < ASSERTION_EPSILON
            );
        }
    }

    #[test]
    fn downward_default_keeps_a_apex_b_left_c_right() {
        let geometry = TernaryGeometry::new(TriangleOrientation::Down, VertexOrder::default());
        assert_cartesian_close(
            geometry.vertex(Component::A),
            TernaryCartesian::new(0.5, -EQUILATERAL_TRIANGLE_HEIGHT),
        );
        assert_cartesian_close(
            geometry.vertex(Component::B),
            TernaryCartesian::new(0.0, 0.0),
        );
        assert_cartesian_close(
            geometry.vertex(Component::C),
            TernaryCartesian::new(1.0, 0.0),
        );
    }

    #[test]
    fn vertex_order_rejects_duplicates() {
        assert!(matches!(
            VertexOrder::new(Component::A, Component::A, Component::C),
            Err(Error::InvalidVertexOrder { .. })
        ));
    }

    #[test]
    fn classify_covers_interior_edges_corners_and_tolerance_boundary() {
        let geometry = TernaryGeometry::default();
        let height = EQUILATERAL_TRIANGLE_HEIGHT;
        assert_eq!(
            geometry
                .classify(TernaryCartesian::new(0.5, 0.2), TOLERANCE)
                .unwrap(),
            TrianglePointLocation::Inside
        );
        for edge_or_corner in [
            TernaryCartesian::new(0.5, 0.0),
            TernaryCartesian::new(0.25, height / 2.0),
            TernaryCartesian::new(0.75, height / 2.0),
            TernaryCartesian::new(0.0, 0.0),
            TernaryCartesian::new(1.0, 0.0),
            TernaryCartesian::new(0.5, height),
        ] {
            assert_eq!(
                geometry.classify(edge_or_corner, TOLERANCE).unwrap(),
                TrianglePointLocation::Boundary
            );
        }
        assert_eq!(
            geometry
                .classify(TernaryCartesian::new(0.5, 2.0e-9), TOLERANCE)
                .unwrap(),
            TrianglePointLocation::Inside
        );
        assert_eq!(
            geometry
                .classify(TernaryCartesian::new(0.5, -5.0e-10), TOLERANCE)
                .unwrap(),
            TrianglePointLocation::Boundary
        );
        assert_eq!(
            geometry
                .classify(TernaryCartesian::new(0.5, -1.0e-5), TOLERANCE)
                .unwrap(),
            TrianglePointLocation::Outside
        );
        for cartesian in [
            TernaryCartesian::new(f64::NAN, 0.0),
            TernaryCartesian::new(f64::INFINITY, 0.0),
            TernaryCartesian::new(0.0, f64::NEG_INFINITY),
        ] {
            assert!(matches!(
                geometry.classify(cartesian, TOLERANCE),
                Err(Error::NonFiniteCartesian { .. })
            ));
        }
        assert!(matches!(
            geometry.unproject(TernaryCartesian::new(0.5, -1.0e-5), TOLERANCE),
            Err(Error::CartesianOutsideTriangle { .. })
        ));
    }

    fn oriented_viewport(
        orientation: TriangleOrientation,
        x_min: f64,
        x_max: f64,
        upward_y_min: f64,
        upward_y_max: f64,
    ) -> TernaryViewport {
        match orientation {
            TriangleOrientation::Up => {
                TernaryViewport::new(x_min, x_max, upward_y_min, upward_y_max).unwrap()
            }
            TriangleOrientation::Down => {
                TernaryViewport::new(x_min, x_max, -upward_y_max, -upward_y_min).unwrap()
            }
        }
    }

    #[test]
    fn visible_edges_cover_full_cropped_interior_and_external_views() {
        let height = EQUILATERAL_TRIANGLE_HEIGHT;
        for orientation in [TriangleOrientation::Up, TriangleOrientation::Down] {
            let geometry = TernaryGeometry::new(orientation, VertexOrder::default());
            let full = geometry
                .visible_edges(TernaryViewport::full(geometry), TOLERANCE)
                .unwrap();
            assert_eq!(full.len(), 3);
            for visible in &full {
                assert_eq!(visible.segment, geometry.triangle_edge(visible.edge));
                assert_eq!(visible.parameter_start, 0.0);
                assert_eq!(visible.parameter_end, 1.0);
            }

            let cases = [
                (
                    oriented_viewport(orientation, -0.05, 0.12, -0.05, 0.12),
                    2,
                    "left corner",
                ),
                (
                    oriented_viewport(orientation, 0.88, 1.05, -0.05, 0.12),
                    2,
                    "right corner",
                ),
                (
                    oriented_viewport(orientation, 0.45, 0.55, height - 0.12, height + 0.05),
                    2,
                    "apex corner",
                ),
                (
                    oriented_viewport(orientation, 0.65, 1.05, -0.05, height + 0.05),
                    2,
                    "right crop",
                ),
                (
                    oriented_viewport(orientation, -0.05, 0.35, -0.05, height + 0.05),
                    2,
                    "left crop",
                ),
                (
                    oriented_viewport(orientation, 0.0, 1.0, 0.5, height + 0.05),
                    2,
                    "top crop",
                ),
                (
                    oriented_viewport(orientation, 0.17, 0.25, 0.3, 0.4),
                    1,
                    "edge only",
                ),
                (
                    oriented_viewport(orientation, 0.45, 0.55, 0.1, 0.2),
                    0,
                    "interior",
                ),
                (
                    oriented_viewport(orientation, 2.0, 3.0, 2.0, 3.0),
                    0,
                    "external",
                ),
            ];

            for (viewport, expected_count, description) in cases {
                let visible = geometry.visible_edges(viewport, TOLERANCE).unwrap();
                assert_eq!(
                    visible.len(),
                    expected_count,
                    "{orientation:?}: {description}"
                );
                for fragment in visible {
                    let source = geometry.triangle_edge(fragment.edge);
                    assert_cartesian_close(
                        fragment.segment.start,
                        source.point_at(fragment.parameter_start),
                    );
                    assert_cartesian_close(
                        fragment.segment.end,
                        source.point_at(fragment.parameter_end),
                    );
                    assert!(fragment.parameter_start >= 0.0);
                    assert!(fragment.parameter_end <= 1.0);
                    assert!(fragment.parameter_start <= fragment.parameter_end);
                }
            }
        }
    }

    #[test]
    fn component_isolines_preserve_semantics_for_every_order_and_orientation() {
        for orientation in [TriangleOrientation::Up, TriangleOrientation::Down] {
            for order in all_orders() {
                let geometry = TernaryGeometry::new(orientation, order);
                for component in Component::ALL {
                    for value in [0.0, 0.25, 0.5, 1.0] {
                        let isoline = geometry
                            .component_isoline(component, value, TOLERANCE)
                            .unwrap();
                        for endpoint in [isoline.start, isoline.end] {
                            let composition = geometry.unproject(endpoint, TOLERANCE).unwrap();
                            assert!(
                                (composition.component(component) - value).abs()
                                    < ASSERTION_EPSILON
                            );
                        }
                        if value == 0.0 {
                            let [first_other, second_other] = component.others();
                            assert_cartesian_close(isoline.start, geometry.vertex(first_other));
                            assert_cartesian_close(isoline.end, geometry.vertex(second_other));
                        }
                        if value == 1.0 {
                            assert_cartesian_close(isoline.start, geometry.vertex(component));
                            assert_eq!(isoline.start, isoline.end);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn isoline_tolerance_and_invalid_values_are_explicit() {
        let geometry = TernaryGeometry::default();
        for (value, expected) in [(-5.0e-10, 0.0), (1.0 + 5.0e-10, 1.0)] {
            let isoline = geometry
                .component_isoline(Component::A, value, TOLERANCE)
                .unwrap();
            for endpoint in [isoline.start, isoline.end] {
                let point = geometry.unproject(endpoint, TOLERANCE).unwrap();
                assert!((point.component(Component::A) - expected).abs() < ASSERTION_EPSILON);
            }
        }

        for value in [-1.0e-5, 1.0 + 1.0e-5, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                geometry.component_isoline(Component::A, value, TOLERANCE),
                Err(Error::InvalidIsolineValue { .. })
            ));
        }
    }

    #[test]
    fn viewport_and_triangle_classifications_remain_separate() {
        let geometry = TernaryGeometry::default();
        let interior = TernaryViewport::new(0.45, 0.55, 0.1, 0.2).unwrap();
        assert!(
            geometry
                .visible_edges(interior, TOLERANCE)
                .unwrap()
                .is_empty()
        );

        let outside_triangle = TernaryCartesian::new(0.1, 0.8);
        let local_viewport = TernaryViewport::new(0.05, 0.15, 0.75, 0.85).unwrap();
        assert!(
            local_viewport
                .contains(outside_triangle, TOLERANCE)
                .unwrap()
        );
        assert_eq!(
            geometry.classify(outside_triangle, TOLERANCE).unwrap(),
            TrianglePointLocation::Outside
        );

        let inside_triangle = TernaryCartesian::new(0.5, 0.2);
        let remote_viewport = TernaryViewport::new(0.0, 0.2, 0.7, 0.8).unwrap();
        assert_eq!(
            geometry.classify(inside_triangle, TOLERANCE).unwrap(),
            TrianglePointLocation::Inside
        );
        assert!(
            !remote_viewport
                .contains(inside_triangle, TOLERANCE)
                .unwrap()
        );
    }

    #[test]
    fn cropped_edges_retain_expected_geometric_identities() {
        let geometry = TernaryGeometry::default();
        let height = EQUILATERAL_TRIANGLE_HEIGHT;
        let cases = [
            (
                TernaryViewport::new(0.65, 1.05, -0.05, height + 0.05).unwrap(),
                vec![TriangleEdge::LeftRight, TriangleEdge::RightApex],
            ),
            (
                TernaryViewport::new(-0.05, 0.35, -0.05, height + 0.05).unwrap(),
                vec![TriangleEdge::LeftRight, TriangleEdge::ApexLeft],
            ),
            (
                TernaryViewport::new(0.0, 1.0, 0.5, height + 0.05).unwrap(),
                vec![TriangleEdge::RightApex, TriangleEdge::ApexLeft],
            ),
            (
                TernaryViewport::new(0.17, 0.25, 0.3, 0.4).unwrap(),
                vec![TriangleEdge::ApexLeft],
            ),
        ];

        for (viewport, expected) in cases {
            let actual: Vec<_> = geometry
                .visible_edges(viewport, TOLERANCE)
                .unwrap()
                .into_iter()
                .map(|visible| visible.edge)
                .collect();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn visible_component_isoline_uses_the_generic_clipper() {
        let geometry = TernaryGeometry::default();
        let viewport = TernaryViewport::new(0.4, 0.6, 0.4, 0.5).unwrap();
        let visible = geometry
            .visible_component_isoline(Component::A, 0.5, viewport, TOLERANCE)
            .unwrap()
            .unwrap();
        assert!(viewport.contains(visible.start, TOLERANCE).unwrap());
        assert!(viewport.contains(visible.end, TOLERANCE).unwrap());
        for endpoint in [visible.start, visible.end] {
            let composition = geometry.unproject(endpoint, TOLERANCE).unwrap();
            assert!((composition.component(Component::A) - 0.5).abs() < ASSERTION_EPSILON);
        }
    }

    proptest! {
        #[test]
        fn projection_round_trip_holds_for_bounded_non_negative_triples(
            a in 0.0_f64..1.0,
            b in 0.0_f64..1.0,
            c in 0.0_f64..1.0,
        ) {
            prop_assume!(a + b + c > 1.0e-6);
            let point = TernaryPoint::new(a, b, c)
                .validate(Normalization::Normalize, TOLERANCE)
                .unwrap();
            for orientation in [TriangleOrientation::Up, TriangleOrientation::Down] {
                for order in all_orders() {
                    let geometry = TernaryGeometry::new(orientation, order);
                    let projected = geometry
                        .project(point, Normalization::RequireUnitSum, TOLERANCE)
                        .unwrap();
                    let recovered = geometry.unproject(projected, TOLERANCE).unwrap();
                    for component in Component::ALL {
                        prop_assert!(
                            (recovered.component(component) - point.component(component)).abs()
                                < ASSERTION_EPSILON
                        );
                    }
                }
            }
        }
    }
}
