use std::error::Error as StdError;
use std::fmt;

use plotters::drawing::DrawingAreaErrorKind;

use crate::coord;

/// An error produced while laying out or drawing a ternary chart.
#[derive(Debug)]
#[non_exhaustive]
pub enum TernaryChartError<E: StdError + Send + Sync> {
    /// Backend-independent geometry or viewport preparation failed.
    Geometry(coord::Error),
    /// Plotters could not complete a drawing operation.
    Drawing(DrawingAreaErrorKind<E>),
    /// The common major-grid step is unusable.
    InvalidMajorStep { value: f64 },
    /// Caption and margins left no usable plotting rectangle.
    InsufficientDrawingArea { width: u32, height: u32 },
}

impl<E: StdError + Send + Sync> fmt::Display for TernaryChartError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Geometry(error) => write!(formatter, "ternary geometry error: {error}"),
            Self::Drawing(error) => write!(formatter, "Plotters drawing error: {error}"),
            Self::InvalidMajorStep { value } => write!(
                formatter,
                "major grid step must be finite, in (0, 1], and produce at most 10,000 intervals: {value:?}"
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
            Self::Drawing(error) => Some(error),
            Self::InvalidMajorStep { .. } | Self::InsufficientDrawingArea { .. } => None,
        }
    }
}

impl<E: StdError + Send + Sync> From<coord::Error> for TernaryChartError<E> {
    fn from(error: coord::Error) -> Self {
        Self::Geometry(error)
    }
}

impl<E: StdError + Send + Sync> From<DrawingAreaErrorKind<E>> for TernaryChartError<E> {
    fn from(error: DrawingAreaErrorKind<E>) -> Self {
        Self::Drawing(error)
    }
}
