//! Publication-quality ternary diagrams built on Plotters.
//!
//! The crate combines a backend-independent ternary geometry and clipping kernel
//! with a Cartesian-backed Plotters chart. It supports full and rectangularly
//! cropped ternary views, configurable axes, native Plotters legends, scientific
//! markers, polygons, composition-anchored text, regular-grid line contours,
//! per-level contour styling, native legends, colour bars, and portable labels.
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
//! continuity, cubic-alpha filled contours, irregular triangulation, and N-component grids
//! are not provided in 0.1.0.

pub mod chart;
pub mod contour;
pub mod coord;
pub mod interpolation;
pub mod prelude;
pub mod series;

pub use chart::{
    AxisLabelFormat, AxisNamePosition, AxisTextStyle, CartesianChartContext, CartesianPlottingArea,
    ContourColorBar, ContourColorBarOrientation, ContourColorBarPosition, ContourDisplayError,
    ContourLabelAnchor, ContourLabelConfig, ContourLabelMode, ContourLabelPlacement,
    ContourLabelStyle, CornerLabelVisibility, CroppedAxisPolicy, EndpointLabelPolicy, TernaryAxis,
    TernaryAxisConfig, TernaryChart, TernaryChartBuilder, TernaryChartError, TernaryMesh,
    TernaryMeshConfig, TickDirection, TickRangeMode, TickSpec, TickStyle,
};
pub use coord::{
    CartesianSegment, ClippedSegment, Component, EQUILATERAL_TRIANGLE_HEIGHT, Error, Normalization,
    PixelBounds, PixelPoint, PixelRect, TernaryCartesian, TernaryGeometry, TernaryPoint,
    TernaryViewport, Tolerance, TriangleEdge, TriangleOrientation, TrianglePointLocation,
    VertexOrder, ViewportAlignment, ViewportFit, ViewportPointLocation, ViewportTransform,
    VisibleTriangleEdge, clip_segment, clip_segment_with_parameters,
};
pub use series::{
    AnnotationClipMode, AnnotationError, AnnotationTextStyle, ContourBandBorderMode,
    ContourBandStylePolicy, ContourLegendPolicy, ContourStylePolicy, HorizontalAnchor,
    InvalidPointPolicy, LocalPoint, LocalSegment, MarkerClipMode, MarkerDrawing, MarkerElement,
    MarkerError, MarkerFill, MarkerFillPolygon, MarkerGeometry, MarkerPartition, MarkerShape,
    MarkerSlice, MarkerStyle, PointMarkerStyleProvider, PolygonError, PreparedPolygon,
    ScalarMapResolution, SeriesError, SweepDirection, TernaryContourBandSeries,
    TernaryContourSeries, TernaryInterpolation, TernaryLineSeries, TernaryPointSeries,
    TernaryPolygon, TernaryScalarMapSeries, TernarySeries, TernarySmoothSeries,
    TernaryStableContourSeries, TernaryText, TextAnchor, TextRotation, VerticalAnchor,
    prepare_points, prepare_polygon, prepare_polyline,
};

pub use contour::{
    AdaptiveContourOptions, ContourBand, ContourBandOptions, ContourBandSet, ContourError,
    ContourFragment, ContourInterpolation, ContourLevel, ContourOptions, ContourPath,
    ContourRegion, ContourRegularization, ContourSet, CubicAlphaBuildOptions, CubicAlphaMethod,
    CubicAlphaOptions, CubicBoundaryPolicy, CubicBuildDiagnostics, CubicContourDiagnostics,
    CubicGridField, FieldError, FieldEvaluationError, FieldInterpolation, FieldSample,
    GridEvaluationError, GridTriangle, GridVertexId, InterpolatedTernaryField, LatticeCoordinate,
    LocatedTriangle, POINT_LOCATION_TOLERANCE, PointBoundaryLocation, PointLocationError,
    PreparedStablePhaseEnsemble, RegularTernaryGrid, RegularTernaryScalarField,
    StableContourDiagnostics, StableContourJunction, StableContourJunctionKind, StableContourLevel,
    StableContourPath, StableContourQuantity, StableContourSet, StablePhaseId, StablePhaseSource,
    StableScalarSource, StableUmbrellaOptions, StableUmbrellaVerification, TernaryCoordinate,
};
pub use interpolation::BinaryExtrapolation;
