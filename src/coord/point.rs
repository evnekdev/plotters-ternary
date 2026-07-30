use super::validation::validate_components;
use super::{Error, Normalization, Tolerance};

/// One of the three semantic components of a ternary composition.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(usize)]
pub enum Component {
    /// The first semantic component.
    A = 0,
    /// The second semantic component.
    B = 1,
    /// The third semantic component.
    C = 2,
}

impl Component {
    /// Components in stable semantic A/B/C order.
    pub const ALL: [Self; 3] = [Self::A, Self::B, Self::C];

    /// Return this component's stable index in an A/B/C array.
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Return the two components other than this one, in A/B/C order.
    pub const fn others(self) -> [Self; 2] {
        match self {
            Self::A => [Self::B, Self::C],
            Self::B => [Self::A, Self::C],
            Self::C => [Self::A, Self::B],
        }
    }
}

/// A ternary composition in semantic A/B/C order.
///
/// [`TernaryPoint::new`] performs no validation or normalisation. Call
/// [`TernaryPoint::validate`] before projecting a user-provided point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TernaryPoint {
    a: f64,
    b: f64,
    c: f64,
}

impl TernaryPoint {
    /// Construct an unvalidated composition without normalising it.
    ///
    /// ```
    /// use plotters_ternary::{Normalization, TernaryPoint, Tolerance};
    ///
    /// let raw = TernaryPoint::new(0.2, 0.3, 0.5);
    /// let point = raw.validate(Normalization::RequireUnitSum, Tolerance::default())?;
    /// assert_eq!(point.component(plotters_ternary::Component::C), 0.5);
    /// # Ok::<(), plotters_ternary::Error>(())
    /// ```
    pub const fn new(a: f64, b: f64, c: f64) -> Self {
        Self { a, b, c }
    }

    /// Return a component value selected by its semantic name.
    pub const fn component(self, component: Component) -> f64 {
        match component {
            Component::A => self.a,
            Component::B => self.b,
            Component::C => self.c,
        }
    }

    /// Return the unvalidated arithmetic sum of the three components.
    pub const fn sum(self) -> f64 {
        self.a + self.b + self.c
    }

    /// Return the components in stable A/B/C order.
    pub const fn as_array(self) -> [f64; 3] {
        [self.a, self.b, self.c]
    }

    /// Validate this composition under an explicit sum policy.
    ///
    /// Tiny finite negatives in `[-tolerance.absolute, 0.0)` become exactly
    /// zero before the sum is evaluated. Larger negatives, non-finite values,
    /// and near-zero sums fail. `Normalization::Normalize` then scales the
    /// cleaned values to unit sum.
    ///
    /// ```
    /// use plotters_ternary::{Normalization, TernaryPoint, Tolerance};
    ///
    /// let point = TernaryPoint::new(2.0, 3.0, 5.0)
    ///     .validate(Normalization::Normalize, Tolerance::default())?;
    /// assert!((point.sum() - 1.0).abs() < 1.0e-12);
    /// # Ok::<(), plotters_ternary::Error>(())
    /// ```
    pub fn validate(
        self,
        normalization: Normalization,
        tolerance: Tolerance,
    ) -> Result<Self, Error> {
        let [a, b, c] = validate_components(self.as_array(), normalization, tolerance)?;
        Ok(Self::new(a, b, c))
    }
}

impl From<[f64; 3]> for TernaryPoint {
    fn from([a, b, c]: [f64; 3]) -> Self {
        Self::new(a, b, c)
    }
}

impl From<TernaryPoint> for [f64; 3] {
    fn from(point: TernaryPoint) -> Self {
        point.as_array()
    }
}

impl From<(f64, f64, f64)> for TernaryPoint {
    fn from((a, b, c): (f64, f64, f64)) -> Self {
        Self::new(a, b, c)
    }
}

impl From<TernaryPoint> for (f64, f64, f64) {
    fn from(point: TernaryPoint) -> Self {
        (point.a, point.b, point.c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOLERANCE: Tolerance = Tolerance {
        absolute: 1.0e-9,
        relative: 1.0e-9,
    };

    #[test]
    fn construction_component_access_and_conversions_preserve_abc_order() {
        let point = TernaryPoint::new(0.2, 0.3, 0.5);
        assert_eq!(point.component(Component::A), 0.2);
        assert_eq!(point.component(Component::B), 0.3);
        assert_eq!(point.component(Component::C), 0.5);
        assert_eq!(point.sum(), 1.0);
        assert_eq!(point.as_array(), [0.2, 0.3, 0.5]);
        assert_eq!(TernaryPoint::from([0.2, 0.3, 0.5]), point);
        assert_eq!(<[f64; 3]>::from(point), [0.2, 0.3, 0.5]);
        assert_eq!(TernaryPoint::from((0.2, 0.3, 0.5)), point);
        assert_eq!(<(f64, f64, f64)>::from(point), (0.2, 0.3, 0.5));
        assert_eq!(Component::B.index(), 1);
        assert_eq!(Component::C.others(), [Component::A, Component::B]);
    }

    #[test]
    fn validation_supports_all_sum_policies() {
        let unit = TernaryPoint::new(0.2, 0.3, 0.5)
            .validate(Normalization::RequireUnitSum, TOLERANCE)
            .unwrap();
        assert_eq!(unit, TernaryPoint::new(0.2, 0.3, 0.5));

        assert!(matches!(
            TernaryPoint::new(2.0, 3.0, 5.0).validate(Normalization::RequireUnitSum, TOLERANCE),
            Err(Error::RequiredSumMismatch { .. })
        ));

        let normalised = TernaryPoint::new(2.0, 3.0, 5.0)
            .validate(Normalization::Normalize, TOLERANCE)
            .unwrap();
        assert!((normalised.sum() - 1.0).abs() < 1.0e-12);
        assert_eq!(normalised, TernaryPoint::new(0.2, 0.3, 0.5));

        let custom = TernaryPoint::new(2.0, 3.0, 5.0)
            .validate(Normalization::RequireSum(10.0), TOLERANCE)
            .unwrap();
        assert_eq!(custom, TernaryPoint::new(2.0, 3.0, 5.0));
        assert!(matches!(
            TernaryPoint::new(2.0, 3.0, 5.0).validate(Normalization::RequireSum(9.0), TOLERANCE),
            Err(Error::RequiredSumMismatch { .. })
        ));
        assert!(matches!(
            TernaryPoint::new(0.0, 0.0, 0.0).validate(Normalization::RequireSum(0.0), TOLERANCE),
            Err(Error::InvalidRequiredSum { .. })
        ));
    }

    #[test]
    fn invalid_sums_and_negative_values_are_rejected_or_cleaned_deterministically() {
        for point in [
            TernaryPoint::new(0.0, 0.0, 0.0),
            TernaryPoint::new(1.0e-12, 0.0, 0.0),
        ] {
            assert!(matches!(
                point.validate(Normalization::Normalize, TOLERANCE),
                Err(Error::InvalidSum { .. })
            ));
        }

        assert!(matches!(
            TernaryPoint::new(-2.0e-9, 0.5, 0.5).validate(Normalization::RequireUnitSum, TOLERANCE),
            Err(Error::NegativeComponent {
                component: Component::A,
                ..
            })
        ));

        let cleaned = TernaryPoint::new(-5.0e-10, 0.5, 0.5)
            .validate(Normalization::RequireUnitSum, TOLERANCE)
            .unwrap();
        assert_eq!(cleaned, TernaryPoint::new(0.0, 0.5, 0.5));
    }

    #[test]
    fn non_finite_components_and_invalid_tolerance_are_rejected() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                TernaryPoint::new(value, 0.5, 0.5)
                    .validate(Normalization::RequireUnitSum, TOLERANCE),
                Err(Error::NonFiniteComponent {
                    component: Component::A,
                    ..
                })
            ));
        }

        assert!(matches!(
            TernaryPoint::new(0.2, 0.3, 0.5).validate(
                Normalization::RequireUnitSum,
                Tolerance {
                    absolute: 0.0,
                    relative: 1.0e-9,
                },
            ),
            Err(Error::InvalidTolerance { .. })
        ));
        assert!(matches!(
            Tolerance::new(1.0e-9, f64::NAN),
            Err(Error::InvalidTolerance { .. })
        ));
    }
}
