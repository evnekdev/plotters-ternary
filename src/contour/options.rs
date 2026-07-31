use crate::interpolation::BinaryExtrapolation;

use super::ContourError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CubicAlphaMethod {
    Akima,
    Makima,
    Pchip,
    Steffen,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CubicBoundaryPolicy {
    #[default]
    LinearFallback,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AdaptiveContourOptions {
    pub value_tolerance: f64,
    pub geometry_tolerance: f64,
    pub max_depth: u8,
    pub flatness_tolerance: f64,
}
impl Default for AdaptiveContourOptions {
    fn default() -> Self {
        Self {
            value_tolerance: 1.0e-10,
            geometry_tolerance: 1.0e-7,
            max_depth: 5,
            flatness_tolerance: 1.0e-5,
        }
    }
}
impl AdaptiveContourOptions {
    pub(crate) fn validate(self) -> Result<(), ContourError> {
        if !self.value_tolerance.is_finite()
            || self.value_tolerance <= 0.0
            || !self.geometry_tolerance.is_finite()
            || self.geometry_tolerance <= 0.0
        {
            return Err(ContourError::InvalidTolerance {
                value_tolerance: self.value_tolerance,
                geometry_tolerance: self.geometry_tolerance,
            });
        }
        if self.max_depth == 0
            || self.max_depth > 10
            || !self.flatness_tolerance.is_finite()
            || self.flatness_tolerance <= 0.0
        {
            return Err(ContourError::InvalidAdaptiveOptions {
                max_depth: self.max_depth,
                flatness_tolerance: self.flatness_tolerance,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContourRegularization {
    pub spacing: f64,
    pub redistribution_passes: usize,
    pub projection_tolerance: f64,
    pub max_projection_iterations: usize,
    pub max_normal_step: f64,
}
impl Default for ContourRegularization {
    fn default() -> Self {
        Self {
            spacing: 0.0125,
            redistribution_passes: 2,
            projection_tolerance: 1.0e-9,
            max_projection_iterations: 16,
            max_normal_step: 0.05,
        }
    }
}
impl ContourRegularization {
    pub(crate) fn validate(self) -> Result<(), ContourError> {
        if !self.spacing.is_finite() || self.spacing <= 0.0 {
            return Err(ContourError::InvalidRegularizationSpacing {
                spacing: self.spacing,
            });
        }
        if !self.projection_tolerance.is_finite()
            || self.projection_tolerance <= 0.0
            || self.max_projection_iterations == 0
            || !self.max_normal_step.is_finite()
            || self.max_normal_step <= 0.0
        {
            return Err(ContourError::InvalidProjectionOptions {
                tolerance: self.projection_tolerance,
                iterations: self.max_projection_iterations,
                max_step: self.max_normal_step,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicAlphaOptions {
    pub method: CubicAlphaMethod,
    pub boundary_policy: CubicBoundaryPolicy,
    pub extrapolation: BinaryExtrapolation,
    pub adaptive: AdaptiveContourOptions,
    pub regularization: Option<ContourRegularization>,
}
impl Default for CubicAlphaOptions {
    fn default() -> Self {
        Self {
            method: CubicAlphaMethod::Steffen,
            boundary_policy: CubicBoundaryPolicy::LinearFallback,
            extrapolation: BinaryExtrapolation::Muggianu,
            adaptive: AdaptiveContourOptions::default(),
            regularization: Some(ContourRegularization::default()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ContourInterpolation {
    Linear,
    CubicAlpha(CubicAlphaOptions),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContourOptions {
    pub interpolation: ContourInterpolation,
    pub value_tolerance: f64,
    pub geometry_tolerance: f64,
    pub regularization: Option<ContourRegularization>,
}
impl ContourOptions {
    pub const fn linear() -> Self {
        Self {
            interpolation: ContourInterpolation::Linear,
            value_tolerance: 1.0e-10,
            geometry_tolerance: 1.0e-8,
            regularization: None,
        }
    }
    pub const fn cubic_alpha(options: CubicAlphaOptions) -> Self {
        Self {
            interpolation: ContourInterpolation::CubicAlpha(options),
            value_tolerance: options.adaptive.value_tolerance,
            geometry_tolerance: options.adaptive.geometry_tolerance,
            regularization: options.regularization,
        }
    }
    pub const fn regularization(mut self, options: Option<ContourRegularization>) -> Self {
        self.regularization = options;
        self
    }
    pub(crate) fn validate(self) -> Result<(), ContourError> {
        if !self.value_tolerance.is_finite()
            || self.value_tolerance <= 0.0
            || !self.geometry_tolerance.is_finite()
            || self.geometry_tolerance <= 0.0
        {
            return Err(ContourError::InvalidTolerance {
                value_tolerance: self.value_tolerance,
                geometry_tolerance: self.geometry_tolerance,
            });
        }
        if let ContourInterpolation::CubicAlpha(options) = self.interpolation {
            options.adaptive.validate()?;
        }
        if let Some(regularization) = self.regularization {
            regularization.validate()?;
        }
        Ok(())
    }
}
impl Default for ContourOptions {
    fn default() -> Self {
        Self::linear()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CubicContourDiagnostics {
    pub cubic_edges: usize,
    pub linear_fallback_edges: usize,
    pub refined_triangles: usize,
    pub maximum_depth_hits: usize,
    pub projection_failures: usize,
}
