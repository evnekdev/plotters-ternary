use plotters::style::ShapeStyle;

use crate::coord::{Normalization, Tolerance};

use super::InvalidPointPolicy;

/// A ternary polyline plus its explicit validation and Plotters style policy.
pub struct TernaryLineSeries<I> {
    points: I,
    style: ShapeStyle,
    normalization: Normalization,
    tolerance: Tolerance,
    invalid_point_policy: InvalidPointPolicy,
}

impl<I> TernaryLineSeries<I> {
    /// Construct a strict unit-sum line series.
    pub fn new<S: Into<ShapeStyle>>(points: I, style: S) -> Self {
        Self {
            points,
            style: style.into(),
            normalization: Normalization::RequireUnitSum,
            tolerance: Tolerance::default(),
            invalid_point_policy: InvalidPointPolicy::Error,
        }
    }

    /// Select explicit validation or normalization for source compositions.
    pub const fn normalization(mut self, normalization: Normalization) -> Self {
        self.normalization = normalization;
        self
    }

    /// Select the numerical tolerance used for validation and clipping.
    pub const fn tolerance(mut self, tolerance: Tolerance) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Select strict errors or run breaks for invalid source points.
    pub const fn invalid_point_policy(mut self, policy: InvalidPointPolicy) -> Self {
        self.invalid_point_policy = policy;
        self
    }

    /// Return the Plotters-native line style.
    pub const fn style(&self) -> ShapeStyle {
        self.style
    }

    pub(crate) fn into_parts(
        self,
    ) -> (I, ShapeStyle, Normalization, Tolerance, InvalidPointPolicy) {
        (
            self.points,
            self.style,
            self.normalization,
            self.tolerance,
            self.invalid_point_policy,
        )
    }
}

#[cfg(test)]
mod tests {
    use plotters::prelude::*;

    use super::*;

    #[test]
    fn constructor_and_configuration_preserve_style_and_policies() {
        let series =
            TernaryLineSeries::new(Vec::<crate::TernaryPoint>::new(), BLUE.stroke_width(4))
                .normalization(Normalization::Normalize)
                .tolerance(Tolerance::new(1.0e-8, 1.0e-8).unwrap())
                .invalid_point_policy(InvalidPointPolicy::Break);
        assert_eq!(series.style().color, BLUE.to_rgba());
        assert_eq!(series.style().stroke_width, 4);
        let (_, _, normalization, tolerance, invalid_policy) = series.into_parts();
        assert_eq!(normalization, Normalization::Normalize);
        assert_eq!(tolerance.absolute, 1.0e-8);
        assert_eq!(invalid_policy, InvalidPointPolicy::Break);
    }
}
