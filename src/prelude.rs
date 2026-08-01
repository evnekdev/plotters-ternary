//! Common stable imports for ordinary ternary chart and contour workflows.
//!
//! Import with `use plotters_ternary::prelude::*;`. Plotters drawing styles,
//! colours, backends, and elements remain intentionally imported from Plotters.

pub use crate::{
    AxisLabelFormat, BinaryExtrapolation, Component, ContourBandOptions, ContourBandSet,
    ContourBandStylePolicy, ContourColorBar, ContourInterpolation, ContourLabelConfig,
    ContourLegendPolicy, ContourOptions, ContourSet, ContourStylePolicy, CubicAlphaBuildOptions,
    CubicAlphaMethod, CubicAlphaOptions, FieldInterpolation, FieldSample, InterpolatedTernaryField,
    MarkerClipMode, MarkerShape, MarkerStyle, Normalization, PreparedStablePhaseEnsemble,
    RegularTernaryGrid, RegularTernaryScalarField, ScalarMapResolution, StableContourQuantity,
    StablePhaseId, StablePhaseSource, StableScalarSource, StableUmbrellaOptions,
    TernaryChartBuilder, TernaryContourBandSeries, TernaryContourSeries, TernaryGeometry,
    TernaryLineSeries, TernaryPoint, TernaryPointSeries, TernaryPolygon, TernaryScalarMapSeries,
    TernaryStableContourSeries, TernaryText, TernaryViewport, Tolerance,
};
