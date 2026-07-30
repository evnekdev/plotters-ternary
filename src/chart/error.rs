use std::error::Error as StdError;
use std::fmt;

use plotters::drawing::DrawingAreaErrorKind;

use crate::{coord, series::SeriesError};

/// An error produced while laying out or drawing a ternary chart.
#[derive(Debug)]
#[non_exhaustive]
pub enum TernaryChartError<E: StdError + Send + Sync> {
    /// Backend-independent geometry or viewport preparation failed.
    Geometry(coord::Error),
    /// Backend-independent series preparation failed.
    Series(SeriesError),
    /// Plotters could not complete a drawing operation.
    Drawing(DrawingAreaErrorKind<E>),
    /// The common major-grid step is unusable.
    InvalidMajorStep { value: f64 },
    /// A tick specification could not produce a meaningful sequence.
    InvalidTickCount { count: usize },
    /// A tick step is not finite, positive, or safely bounded.
    InvalidTickStep { value: f64 },
    /// An explicit tick value is not finite or lies outside unit composition space.
    InvalidTickValue { value: f64 },
    /// Caption and margins left no usable plotting rectangle.
    InsufficientDrawingArea { width: u32, height: u32 },
}

impl<E: StdError + Send + Sync> fmt::Display for TernaryChartError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Geometry(error) => write!(formatter, "ternary geometry error: {error}"),
            Self::Series(error) => write!(formatter, "ternary series error: {error}"),
            Self::Drawing(error) => write!(formatter, "Plotters drawing error: {error}"),
            Self::InvalidMajorStep { value } => write!(
                formatter,
                "major grid step must be finite, in (0, 1], and produce at most 10,000 intervals: {value:?}"
            ),
            Self::InvalidTickCount { count } => {
                write!(
                    formatter,
                    "tick count must be at least one interval; received {count}"
                )
            }
            Self::InvalidTickStep { value } => write!(
                formatter,
                "tick step must be finite, in (0, 1], and produce at most 10,000 intervals: {value:?}"
            ),
            Self::InvalidTickValue { value } => write!(
                formatter,
                "tick value must be finite and within the unit composition range: {value:?}"
            ),
            Self::InsufficientDrawingArea { width, height } => write!(
                formatter,
                "caption and margins left an insufficient plotting area: {width}x{height}"
            ),
        }
    }
}

impl<E: StdError + Send + Sync + 'static> StdError for TernaryChartError<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Geometry(error) => Some(error),
            Self::Series(error) => Some(error),
            Self::Drawing(error) => Some(error),
            Self::InvalidMajorStep { .. }
            | Self::InvalidTickCount { .. }
            | Self::InvalidTickStep { .. }
            | Self::InvalidTickValue { .. }
            | Self::InsufficientDrawingArea { .. } => None,
        }
    }
}

impl<E: StdError + Send + Sync> From<coord::Error> for TernaryChartError<E> {
    fn from(error: coord::Error) -> Self {
        Self::Geometry(error)
    }
}

impl<E: StdError + Send + Sync> From<SeriesError> for TernaryChartError<E> {
    fn from(error: SeriesError) -> Self {
        Self::Series(error)
    }
}

impl<E: StdError + Send + Sync> From<DrawingAreaErrorKind<E>> for TernaryChartError<E> {
    fn from(error: DrawingAreaErrorKind<E>) -> Self {
        Self::Drawing(error)
    }
}
