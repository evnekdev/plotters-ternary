//! Ternary composition geometry and Cartesian-backed Plotters charts.

pub mod chart;
pub mod coord;
pub mod series;

pub use chart::{
    AxisLabelFormat, AxisNamePosition, AxisTextStyle, CartesianChartContext, CartesianPlottingArea,
    CornerLabelVisibility, CroppedAxisPolicy, EndpointLabelPolicy, PreparedRotatedText,
    TernaryAxis, TernaryAxisConfig, TernaryChart, TernaryChartBuilder, TernaryChartError,
    TernaryMeshConfig, TickDirection, TickRangeMode, TickSpec, TickStyle, capture_rotated_text,
    capture_svg_rotated_text, draw_prepared_rotated_text, svg_rotated_text_elements,
};
pub use coord::{
    CartesianSegment, ClippedSegment, Component, EQUILATERAL_TRIANGLE_HEIGHT, Error, Normalization,
    PixelBounds, PixelPoint, PixelRect, TernaryCartesian, TernaryGeometry, TernaryPoint,
    TernaryViewport, Tolerance, TriangleEdge, TriangleOrientation, TrianglePointLocation,
    VertexOrder, ViewportAlignment, ViewportFit, ViewportPointLocation, ViewportTransform,
    VisibleTriangleEdge, clip_segment, clip_segment_with_parameters,
};
pub use series::{
    InvalidPointPolicy, LocalPoint, LocalSegment, MarkerClipMode, MarkerDrawing, MarkerElement,
    MarkerError, MarkerFill, MarkerFillPolygon, MarkerGeometry, MarkerPartition, MarkerShape,
    MarkerSlice, MarkerStyle, PointMarkerStyleProvider, SeriesError, SweepDirection,
    TernaryInterpolation, TernaryLineSeries, TernaryPointSeries, TernarySeries,
    TernarySmoothSeries, prepare_points, prepare_polyline,
};
