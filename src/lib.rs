//! Backend-independent geometry for ternary compositions.
//!
//! Rendering and viewport support deliberately follow in later milestones.

pub mod coord;

pub use coord::{
    Component, Error, Normalization, TernaryCartesian, TernaryGeometry, TernaryPoint, Tolerance,
    TriangleOrientation, TrianglePointLocation, VertexOrder,
};
