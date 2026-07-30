use plotters::style::{BLACK, Color, ShapeStyle};

use crate::coord::{Normalization, Tolerance};

use super::{InvalidPointPolicy, MarkerClipMode, MarkerShape};

/// A ternary point collection with marker and validation configuration.
pub struct TernaryPointSeries<I> {
    points: I,
    size: u32,
    style: ShapeStyle,
    marker: MarkerShape,
    clip_mode: MarkerClipMode,
    normalization: Normalization,
    tolerance: Tolerance,
    invalid_point_policy: InvalidPointPolicy,
}

impl<I> TernaryPointSeries<I> {
    /// Construct a strict unit-sum circular point series.
    pub fn new(points: I) -> Self {
        Self {
            points,
            size: 5,
            style: BLACK.filled(),
            marker: MarkerShape::default(),
            clip_mode: MarkerClipMode::default(),
            normalization: Normalization::RequireUnitSum,
            tolerance: Tolerance::default(),
            invalid_point_policy: InvalidPointPolicy::Error,
        }
    }

    /// Set the marker radius/half-size in backend pixels.
    pub const fn size(mut self, size: u32) -> Self {
        self.size = size;
        self
    }

    /// Set a Plotters-native marker style.
    pub fn style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Select a built-in marker shape.
    pub const fn marker(mut self, marker: MarkerShape) -> Self {
        self.marker = marker;
        self
    }

    /// Select centre clipping or the explicit unrestricted escape hatch.
    pub const fn clip_mode(mut self, clip_mode: MarkerClipMode) -> Self {
        self.clip_mode = clip_mode;
        self
    }

    /// Select explicit validation or normalization for source compositions.
    pub const fn normalization(mut self, normalization: Normalization) -> Self {
        self.normalization = normalization;
        self
    }

    /// Select the numerical tolerance used for validation and centre clipping.
    pub const fn tolerance(mut self, tolerance: Tolerance) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Select strict errors or omission for invalid marker points.
    pub const fn invalid_point_policy(mut self, policy: InvalidPointPolicy) -> Self {
        self.invalid_point_policy = policy;
        self
    }

    pub const fn marker_size(&self) -> u32 {
        self.size
    }

    pub const fn marker_style(&self) -> ShapeStyle {
        self.style
    }

    pub const fn marker_shape(&self) -> MarkerShape {
        self.marker
    }

    pub const fn marker_clip_mode(&self) -> MarkerClipMode {
        self.clip_mode
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn into_parts(
        self,
    ) -> (
        I,
        u32,
        ShapeStyle,
        MarkerShape,
        MarkerClipMode,
        Normalization,
        Tolerance,
        InvalidPointPolicy,
    ) {
        (
            self.points,
            self.size,
            self.style,
            self.marker,
            self.clip_mode,
            self.normalization,
            self.tolerance,
            self.invalid_point_policy,
        )
    }
}

#[cfg(test)]
mod tests {
    use plotters::prelude::*;

    use super::*;

    #[test]
    fn point_configuration_retains_plotters_style_and_marker_policy() {
        let series = TernaryPointSeries::new(Vec::<crate::TernaryPoint>::new())
            .size(9)
            .style(RED.filled())
            .marker(MarkerShape::Triangle)
            .clip_mode(MarkerClipMode::None)
            .normalization(Normalization::Normalize)
            .invalid_point_policy(InvalidPointPolicy::Break);
        assert_eq!(series.marker_size(), 9);
        assert_eq!(series.marker_style().color, RED.to_rgba());
        assert_eq!(series.marker_shape(), MarkerShape::Triangle);
        assert_eq!(series.marker_clip_mode(), MarkerClipMode::None);
    }
}
