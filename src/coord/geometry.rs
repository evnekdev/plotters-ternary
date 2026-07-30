use super::{Component, Error, Normalization, TernaryPoint, Tolerance};

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
    fn default() -> Self {
        Self {
            left: Component::A,
            right: Component::B,
            apex: Component::C,
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
    /// assert_eq!(point.as_array(), [0.5, 0.5, 0.0]);
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
        assert_cartesian_close(
            geometry
                .project(
                    unit_point(1.0, 0.0, 0.0),
                    Normalization::RequireUnitSum,
                    TOLERANCE,
                )
                .unwrap(),
            TernaryCartesian::new(0.0, 0.0),
        );
        assert_cartesian_close(
            geometry
                .project(
                    unit_point(0.0, 1.0, 0.0),
                    Normalization::RequireUnitSum,
                    TOLERANCE,
                )
                .unwrap(),
            TernaryCartesian::new(1.0, 0.0),
        );
        assert_cartesian_close(
            geometry
                .project(
                    unit_point(0.0, 0.0, 1.0),
                    Normalization::RequireUnitSum,
                    TOLERANCE,
                )
                .unwrap(),
            TernaryCartesian::new(0.5, height),
        );
        assert_cartesian_close(
            geometry
                .project(
                    unit_point(0.5, 0.5, 0.0),
                    Normalization::RequireUnitSum,
                    TOLERANCE,
                )
                .unwrap(),
            TernaryCartesian::new(0.5, 0.0),
        );
        assert_cartesian_close(
            geometry
                .project(
                    unit_point(0.5, 0.0, 0.5),
                    Normalization::RequireUnitSum,
                    TOLERANCE,
                )
                .unwrap(),
            TernaryCartesian::new(0.25, height / 2.0),
        );
        assert_cartesian_close(
            geometry
                .project(
                    unit_point(0.0, 0.5, 0.5),
                    Normalization::RequireUnitSum,
                    TOLERANCE,
                )
                .unwrap(),
            TernaryCartesian::new(0.75, height / 2.0),
        );
        assert_cartesian_close(
            geometry
                .project(
                    unit_point(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0),
                    Normalization::RequireUnitSum,
                    TOLERANCE,
                )
                .unwrap(),
            TernaryCartesian::new(0.5, height / 3.0),
        );
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
                geometry.vertex(Component::C),
                TernaryCartesian::new(0.5, apex_y),
            );
            let projected = geometry
                .project(point, Normalization::RequireUnitSum, TOLERANCE)
                .unwrap();
            assert_point_close(geometry.unproject(projected, TOLERANCE).unwrap(), point);
        }
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
