use std::fmt;

use crate::coord;

use super::{AnnotationError, MarkerError, PolygonError};

/// How invalid compositions affect a ternary series.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum InvalidPointPolicy {
    /// Stop preparation and report the source point index.
    #[default]
    Error,
    /// End the current line run, then continue with later valid points.
    Break,
}

/// Backend-independent failures while preparing ternary series geometry.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum SeriesError {
    /// A source composition could not be validated or projected.
    InvalidPoint { index: usize, source: coord::Error },
    /// A generated interpolation sample is materially invalid.
    InvalidInterpolatedPoint { sample: usize, source: coord::Error },
    /// `spline1d` could not evaluate a parameter inside the source domain.
    SmoothInterpolationFailed { sample: usize, parameter: f64 },
    /// The fixed fallback sampling density is zero or exceeds its guardrail.
    InvalidSmoothSampling {
        samples_per_interval: u32,
        maximum: u32,
    },
    /// The requested curve would exceed the bounded fallback sample count.
    TooManySmoothSamples { requested: usize, maximum: usize },
    /// Marker size zero has no useful rendering semantics.
    InvalidMarkerSize { size: u32 },
    /// Polygon validation, projection, or clipping preparation failed.
    Polygon(PolygonError),
    /// A ternary text annotation could not be prepared.
    Annotation(AnnotationError),
    /// A scientific marker configuration was invalid for an optional source
    /// index. `None` denotes a legacy uniform marker before point preparation.
    Marker {
        index: Option<usize>,
        source: MarkerError,
    },
}

impl fmt::Display for SeriesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPoint { index, source } => write!(
                formatter,
                "invalid ternary series point at index {index}: {source}"
            ),
            Self::InvalidInterpolatedPoint { sample, source } => write!(
                formatter,
                "invalid generated ternary interpolation sample {sample}: {source}"
            ),
            Self::SmoothInterpolationFailed { sample, parameter } => write!(
                formatter,
                "spline1d could not evaluate sample {sample} at parameter {parameter:?}"
            ),
            Self::InvalidSmoothSampling {
                samples_per_interval,
                maximum,
            } => write!(
                formatter,
                "smooth samples per interval must be in 1..={maximum}; received {samples_per_interval}"
            ),
            Self::TooManySmoothSamples { requested, maximum } => write!(
                formatter,
                "smooth curve requires {requested} samples; configured maximum is {maximum}"
            ),
            Self::InvalidMarkerSize { size } => {
                write!(formatter, "marker size must be greater than zero: {size}")
            }
            Self::Polygon(source) => write!(formatter, "ternary polygon error: {source}"),
            Self::Annotation(source) => write!(formatter, "ternary annotation error: {source}"),
            Self::Marker {
                index: Some(index),
                source,
            } => {
                write!(
                    formatter,
                    "invalid marker for ternary series point at index {index}: {source}"
                )
            }
            Self::Marker {
                index: None,
                source,
            } => write!(formatter, "invalid marker style: {source}"),
        }
    }
}

impl std::error::Error for SeriesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPoint { source, .. } | Self::InvalidInterpolatedPoint { source, .. } => {
                Some(source)
            }
            Self::SmoothInterpolationFailed { .. }
            | Self::InvalidSmoothSampling { .. }
            | Self::TooManySmoothSamples { .. }
            | Self::InvalidMarkerSize { .. } => None,
            Self::Marker { source, .. } => Some(source),
            Self::Polygon(source) => Some(source),
            Self::Annotation(source) => Some(source),
        }
    }
}
