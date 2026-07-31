//! Backend-independent interpolation primitives for regular simplicial grids.
//!
//! This module deliberately has no Plotters, viewport, or chart dependency.

mod alpha_cubic;
mod edge;

pub use alpha_cubic::{
    BinaryExtrapolation, CubicAlphaTriangle, DirectedAlphaInterval, InterpolationError,
    PairEvaluation, evaluate_pair,
};
pub use edge::AlphaInterval;
