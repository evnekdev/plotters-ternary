//! Publication-quality ternary diagrams built on Plotters.
//!
//! The crate combines a backend-independent ternary geometry and clipping kernel
//! with a Cartesian-backed Plotters chart. It supports full and rectangularly
//! cropped ternary views, configurable axes, native Plotters legends, scientific
//! markers, polygons, composition-anchored text, and regular-grid line contours.
//!
//! Start with [`prelude`] for common stable types, then see the README and the
//! examples for complete PNG/SVG rendering programs. Cubic-alpha contour
//! construction requires the [`cubic-alpha`](https://docs.rs/plotters-ternary/latest/plotters_ternary/#cargo-features) feature.
//!
//! ## Guarantees and limits
//!
//! Ternary coordinates always retain semantic A/B/C order. Viewport clipping is
//! mathematical and the viewport frame is invisible. Cubic contour fields share
//! exact one-dimensional edge intervals and are C0 across regular-grid edges; C1
//! continuity, filled contours, irregular triangulation, and N-component grids
//! are not provided in 0.1.0.

pub mod chart;
pub mod contour;
pub mod coord;
pub mod interpolation;
pub mod prelude;
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
    AnnotationClipMode, AnnotationError, AnnotationTextStyle, HorizontalAnchor, InvalidPointPolicy,
    LocalPoint, LocalSegment, MarkerClipMode, MarkerDrawing, MarkerElement, MarkerError,
    MarkerFill, MarkerFillPolygon, MarkerGeometry, MarkerPartition, MarkerShape, MarkerSlice,
    MarkerStyle, PointMarkerStyleProvider, PolygonError, PreparedPolygon, SeriesError,
    SweepDirection, TernaryContourSeries, TernaryInterpolation, TernaryLineSeries,
    TernaryPointSeries, TernaryPolygon, TernarySeries, TernarySmoothSeries, TernaryText,
    TextAnchor, TextRotation, VerticalAnchor, prepare_points, prepare_polygon, prepare_polyline,
};

pub use contour::{
    AdaptiveContourOptions, ContourError, ContourInterpolation, ContourLevel, ContourOptions,
    ContourPath, ContourRegularization, ContourSet, CubicAlphaMethod, CubicAlphaOptions,
    CubicBoundaryPolicy, CubicContourDiagnostics, FieldError, GridVertexId, LatticeCoordinate,
    RegularTernaryScalarField, TernaryCoordinate,
};
pub use interpolation::BinaryExtrapolation;
