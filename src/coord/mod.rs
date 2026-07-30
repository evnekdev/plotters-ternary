//! Ternary composition validation and geometric projection.

mod clipping;
mod geometry;
mod point;
mod validation;
mod viewport;

pub use clipping::{CartesianSegment, ClippedSegment, clip_segment, clip_segment_with_parameters};
pub use geometry::{
    EQUILATERAL_TRIANGLE_HEIGHT, TernaryCartesian, TernaryGeometry, TriangleEdge,
    TriangleOrientation, TrianglePointLocation, VertexOrder, VisibleTriangleEdge,
};
pub use point::{Component, TernaryPoint};
pub use validation::{Error, Normalization, Tolerance};
pub use viewport::{
    PixelBounds, PixelPoint, PixelRect, TernaryViewport, ViewportAlignment, ViewportFit,
    ViewportPointLocation, ViewportTransform,
};
