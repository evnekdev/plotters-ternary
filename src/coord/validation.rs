use std::fmt;

use super::Component;

/// The explicit sum policy applied before a composition is used geometrically.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Normalization {
    /// Accept only compositions whose components sum to one within tolerance.
    RequireUnitSum,
    /// Scale a finite, non-negative composition to unit sum.
    Normalize,
    /// Accept only compositions whose components sum to the specified positive value.
    RequireSum(f64),
}

/// Absolute and relative tolerances used by validation and triangle boundaries.
///
/// Both values must be finite and strictly positive. Values within the absolute
/// tolerance of zero are treated as floating-point boundary noise where the
/// documented operation permits boundary clean-up.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tolerance {
    /// Absolute tolerance, used near zero.
    pub absolute: f64,
    /// Relative tolerance, scaled by the compared magnitudes.
    pub relative: f64,
}

impl Tolerance {
    /// Construct a tolerance after checking that both terms are usable.
    pub fn new(absolute: f64, relative: f64) -> Result<Self, Error> {
        let tolerance = Self { absolute, relative };
        tolerance.validate()?;
        Ok(tolerance)
    }

    /// Validate a tolerance value, including one built with public fields.
    pub fn validate(self) -> Result<(), Error> {
        if !self.absolute.is_finite()
            || !self.relative.is_finite()
            || self.absolute <= 0.0
            || self.relative <= 0.0
        {
            return Err(Error::InvalidTolerance {
                absolute: self.absolute,
                relative: self.relative,
            });
        }
        Ok(())
    }

    /// Return whether two finite values are equal within this tolerance.
    pub fn is_close(self, left: f64, right: f64) -> bool {
        (left - right).abs() <= self.absolute + self.relative * left.abs().max(right.abs())
    }

    pub(crate) fn is_near_zero(self, value: f64) -> bool {
        value.abs() <= self.absolute
    }
}

impl Default for Tolerance {
    fn default() -> Self {
        Self {
            absolute: 1.0e-12,
            relative: 1.0e-12,
        }
    }
}

/// Errors reported by composition validation and geometric projection.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
    /// A tolerance has a non-finite or non-positive term.
    InvalidTolerance { absolute: f64, relative: f64 },
    /// A composition component is NaN or infinite.
    NonFiniteComponent { component: Component, value: f64 },
    /// A composition component is negative by more than the absolute tolerance.
    NegativeComponent {
        component: Component,
        value: f64,
        tolerance: f64,
    },
    /// A composition sum is non-finite, zero, or too close to zero.
    InvalidSum { sum: f64, minimum: f64 },
    /// A custom required sum is non-finite or too close to zero.
    InvalidRequiredSum { required_sum: f64, minimum: f64 },
    /// A composition does not have the sum required by its policy.
    RequiredSumMismatch {
        expected: f64,
        actual: f64,
        tolerance: Tolerance,
    },
    /// A vertex order repeats one or more components.
    InvalidVertexOrder {
        left: Component,
        right: Component,
        apex: Component,
    },
    /// A Cartesian coordinate is NaN or infinite.
    NonFiniteCartesian { x: f64, y: f64 },
    /// A Cartesian point is materially outside the complete ternary triangle.
    CartesianOutsideTriangle {
        x: f64,
        y: f64,
        tolerance: Tolerance,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTolerance { absolute, relative } => write!(
                formatter,
                "invalid tolerance: absolute={absolute:?}, relative={relative:?}; both must be finite and positive"
            ),
            Self::NonFiniteComponent { component, value } => {
                write!(
                    formatter,
                    "component {component:?} is not finite: {value:?}"
                )
            }
            Self::NegativeComponent {
                component,
                value,
                tolerance,
            } => write!(
                formatter,
                "component {component:?} is negative beyond tolerance {tolerance:?}: {value:?}"
            ),
            Self::InvalidSum { sum, minimum } => write!(
                formatter,
                "composition sum must be finite and greater than {minimum:?}: {sum:?}"
            ),
            Self::InvalidRequiredSum {
                required_sum,
                minimum,
            } => write!(
                formatter,
                "required sum must be finite and greater than {minimum:?}: {required_sum:?}"
            ),
            Self::RequiredSumMismatch {
                expected,
                actual,
                tolerance,
            } => write!(
                formatter,
                "composition sum {actual:?} does not match required sum {expected:?} within {tolerance:?}"
            ),
            Self::InvalidVertexOrder { left, right, apex } => write!(
                formatter,
                "vertex order must contain A, B, and C exactly once; received {left:?}, {right:?}, {apex:?}"
            ),
            Self::NonFiniteCartesian { x, y } => {
                write!(
                    formatter,
                    "Cartesian coordinate is not finite: ({x:?}, {y:?})"
                )
            }
            Self::CartesianOutsideTriangle { x, y, tolerance } => write!(
                formatter,
                "Cartesian point ({x:?}, {y:?}) is outside the ternary triangle within {tolerance:?}"
            ),
        }
    }
}

impl std::error::Error for Error {}

pub(crate) fn validate_components(
    mut components: [f64; 3],
    normalization: Normalization,
    tolerance: Tolerance,
) -> Result<[f64; 3], Error> {
    tolerance.validate()?;

    if let Normalization::RequireSum(required_sum) = normalization
        && (!required_sum.is_finite() || required_sum <= tolerance.absolute)
    {
        return Err(Error::InvalidRequiredSum {
            required_sum,
            minimum: tolerance.absolute,
        });
    }

    for (component, value) in Component::ALL.into_iter().zip(&mut components) {
        if !value.is_finite() {
            return Err(Error::NonFiniteComponent {
                component,
                value: *value,
            });
        }
        if *value < -tolerance.absolute {
            return Err(Error::NegativeComponent {
                component,
                value: *value,
                tolerance: tolerance.absolute,
            });
        }
        if *value < 0.0 {
            *value = 0.0;
        }
    }

    let sum: f64 = components.into_iter().sum();
    if !sum.is_finite() || sum <= tolerance.absolute {
        return Err(Error::InvalidSum {
            sum,
            minimum: tolerance.absolute,
        });
    }

    match normalization {
        Normalization::Normalize => Ok(components.map(|component| component / sum)),
        Normalization::RequireUnitSum => {
            require_sum(sum, 1.0, tolerance)?;
            Ok(components)
        }
        Normalization::RequireSum(required_sum) => {
            require_sum(sum, required_sum, tolerance)?;
            Ok(components)
        }
    }
}

fn require_sum(actual: f64, expected: f64, tolerance: Tolerance) -> Result<(), Error> {
    if tolerance.is_close(actual, expected) {
        Ok(())
    } else {
        Err(Error::RequiredSumMismatch {
            expected,
            actual,
            tolerance,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_values_use_absolute_and_relative_terms() {
        let tolerance = Tolerance::new(1.0e-6, 1.0e-6).unwrap();
        assert!(tolerance.is_close(1.0, 1.0 + 1.5e-6));
        assert!(!tolerance.is_close(1.0, 1.0 + 3.0e-6));
    }
}
