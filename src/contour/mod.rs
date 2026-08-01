//! Compatibility re-exports for the backend-independent numerical contour core.
//!
//! Grid interpolation, isoline topology, path assembly, regularization, and
//! level projection are owned by [`ternary_contours`]. This crate only projects
//! final semantic coordinates into chart space and clips them for rendering.

pub use ternary_contours::{
    AdaptiveContourOptions, ContourBand, ContourBandOptions, ContourBandSet, ContourError,
    ContourFragment, ContourInterpolation, ContourLevel, ContourOptions, ContourPath,
    ContourRegion, ContourRegularization, ContourSet, CubicAlphaBuildOptions, CubicAlphaMethod,
    CubicAlphaOptions, CubicBoundaryPolicy, CubicBuildDiagnostics, CubicContourDiagnostics,
    CubicGridField, FieldError, FieldEvaluationError, FieldInterpolation, FieldSample,
    GridEvaluationError, GridTriangle, GridVertexId, InterpolatedTernaryField, LatticeCoordinate,
    LocatedTriangle, POINT_LOCATION_TOLERANCE, PointBoundaryLocation, PointLocationError,
    RegularTernaryGrid, RegularTernaryScalarField, TernaryCoordinate,
};

#[cfg(feature = "stable-contours")]
pub use ternary_contours::{
    PreparedStablePhaseEnsemble, StableContourDiagnostics, StableContourError,
    StableContourJunction, StableContourJunctionKind, StableContourLevel, StableContourPath,
    StableContourQuantity, StableContourSet, StableJunctionId, StablePhaseId, StablePhaseSource,
    StableScalarSource, StableSourceEvaluationError, StableUmbrellaOptions,
    StableUmbrellaVerification, StableVerificationPassDiagnostics,
};
