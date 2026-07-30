//! Ternary composition validation and geometric projection.

mod geometry;
mod point;
mod validation;

pub use geometry::{
    TernaryCartesian, TernaryGeometry, TriangleOrientation, TrianglePointLocation, VertexOrder,
};
pub use point::{Component, TernaryPoint};
pub use validation::{Error, Normalization, Tolerance};
