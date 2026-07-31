//! Common stable imports for ordinary ternary chart and contour workflows.
//!
//! Import with `use plotters_ternary::prelude::*;`. Plotters drawing styles,
//! colours, backends, and elements remain intentionally imported from Plotters.

pub use crate::{
    AxisLabelFormat, BinaryExtrapolation, Component, ContourColorBar, ContourInterpolation,
    ContourLabelConfig, ContourLegendPolicy, ContourOptions, ContourSet, ContourStylePolicy,
    CubicAlphaMethod, CubicAlphaOptions, MarkerClipMode, MarkerShape, MarkerStyle, Normalization,
    RegularTernaryGrid, RegularTernaryScalarField, TernaryChartBuilder, TernaryContourSeries,
    TernaryGeometry, TernaryLineSeries, TernaryPoint, TernaryPointSeries, TernaryPolygon,
    TernaryText, TernaryViewport, Tolerance,
};
