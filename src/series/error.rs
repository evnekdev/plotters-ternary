use std::fmt;

use crate::{ContourDisplayError, coord};

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
    /// An ordered contour style collection contained no styles.
    EmptyContourStyleCollection,
    /// An ordered filled-contour style collection contained no styles.
    EmptyContourBandStyleCollection,
    /// Scalar-map micro-triangulation was zero or exceeded the guardrail.
    InvalidScalarMapResolution { value: usize },
    /// Scalar-map opacity was not a finite value in the inclusive zero-to-one range.
    InvalidScalarMapOpacity { opacity: f64 },
    /// A continuous contour colour range or stroke width was invalid.
    InvalidContourColorRange {
        minimum: f64,
        maximum: f64,
        stroke_width: u32,
    },
    /// An automatic contour legend stride was zero.
    InvalidContourLegendStride { stride: usize },
    /// Contour label or colour-bar rendering configuration was invalid.
    ContourDisplay(ContourDisplayError),
    /// Scalar-map field access failed despite an already validated field.
    ScalarMapField(ternary_contours::FieldError),
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
            Self::InvalidScalarMapOpacity { opacity } => {
                write!(
                    formatter,
                    "scalar-map opacity must be finite and in 0..=1: {opacity:?}"
                )
            }
            Self::EmptyContourBandStyleCollection => {
                write!(
                    formatter,
                    "filled contour style collection must not be empty"
                )
            }
            Self::InvalidScalarMapResolution { value } => write!(
                formatter,
                "scalar-map subdivisions per edge must be in 1..=64, or adaptive depth in 0..=6; received {value}"
            ),
            Self::EmptyContourStyleCollection => {
                write!(formatter, "contour style collection must not be empty")
            }
            Self::InvalidContourColorRange {
                minimum,
                maximum,
                stroke_width,
            } => write!(
                formatter,
                "contour colour range must be finite and increasing with nonzero stroke width: {minimum:?}..{maximum:?}, width {stroke_width}"
            ),
            Self::InvalidContourLegendStride { stride } => write!(
                formatter,
                "contour legend stride must be greater than zero: {stride}"
            ),
            Self::ContourDisplay(source) => write!(formatter, "contour display error: {source}"),
            Self::ScalarMapField(source) => write!(formatter, "scalar-map field error: {source}"),
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
            | Self::InvalidMarkerSize { .. }
            | Self::EmptyContourStyleCollection
            | Self::EmptyContourBandStyleCollection
            | Self::InvalidScalarMapResolution { .. }
            | Self::InvalidScalarMapOpacity { .. }
            | Self::InvalidContourColorRange { .. }
            | Self::InvalidContourLegendStride { .. } => None,
            Self::Marker { source, .. } => Some(source),
            Self::ContourDisplay(source) => Some(source),
            Self::ScalarMapField(source) => Some(source),
            Self::Polygon(source) => Some(source),
            Self::Annotation(source) => Some(source),
        }
    }
}
