//! Ternary composition geometry and Cartesian-backed Plotters charts.

pub mod chart;
pub mod coord;
pub mod series;

pub use chart::{
    CartesianChartContext, CartesianPlottingArea, TernaryChart, TernaryChartBuilder,
    TernaryChartError, TernaryMeshConfig,
};
pub use coord::{
    CartesianSegment, ClippedSegment, Component, EQUILATERAL_TRIANGLE_HEIGHT, Error, Normalization,
    PixelBounds, PixelPoint, PixelRect, TernaryCartesian, TernaryGeometry, TernaryPoint,
    TernaryViewport, Tolerance, TriangleEdge, TriangleOrientation, TrianglePointLocation,
    VertexOrder, ViewportAlignment, ViewportFit, ViewportPointLocation, ViewportTransform,
    VisibleTriangleEdge, clip_segment, clip_segment_with_parameters,
};
pub use series::{
    InvalidPointPolicy, MarkerClipMode, MarkerShape, SeriesError, TernaryInterpolation,
    TernaryLineSeries, TernaryPointSeries, TernarySeries, TernarySmoothSeries, prepare_points,
    prepare_polyline,
};
