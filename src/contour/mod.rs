//! Compatibility re-exports for the backend-independent numerical contour core.
//!
//! Grid interpolation, isoline topology, path assembly, regularization, and
//! level projection are owned by [`ternary_contours`]. This crate only projects
//! final semantic coordinates into chart space and clips them for rendering.

pub use ternary_contours::{
    AdaptiveContourOptions, ContourError, ContourInterpolation, ContourLevel, ContourOptions,
    ContourPath, ContourRegularization, ContourSet, CubicAlphaMethod, CubicAlphaOptions,
    CubicBoundaryPolicy, CubicContourDiagnostics, FieldError, GridEvaluationError, GridVertexId,
    LatticeCoordinate, RegularTernaryGrid, RegularTernaryScalarField, TernaryCoordinate,
};
