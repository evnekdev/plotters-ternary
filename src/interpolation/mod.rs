//! Compatibility re-exports for the extracted numerical interpolation core.
//!
//! Low-level field and interpolation algorithms live in [`ternary_contours`].
//! `plotters-ternary` retains only chart projection, visual clipping, styling,
//! legends, and backend rendering.

pub use ternary_contours::BinaryExtrapolation;
pub use ternary_contours::interpolation::{
    AlphaInterval, CubicAlphaTriangle, DirectedAlphaInterval, InterpolationError, PairEvaluation,
    evaluate_pair,
};
