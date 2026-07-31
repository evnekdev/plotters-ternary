//! Compatibility re-exports for the extracted numerical interpolation core.
//!
//! Low-level field and interpolation algorithms live in [`ternary_contours`].
//! `plotters-ternary` retains contour topology, clipping, and Plotters rendering.

pub use ternary_contours::{
    AlphaInterval, BinaryExtrapolation, CubicAlphaTriangle, DirectedAlphaInterval,
    InterpolationError, PairEvaluation, evaluate_pair,
};
