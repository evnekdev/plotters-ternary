use plotters::style::{BLACK, Color, ShapeStyle};

use crate::coord::{Normalization, TernaryPoint, Tolerance};

use super::{InvalidPointPolicy, MarkerClipMode, MarkerShape, MarkerStyle};

/// A callback that chooses one complete marker style for a validated source
/// composition. The callback is evaluated only while the series is drawn.
pub trait PointMarkerStyleProvider {
    /// Return the style for `index` and its normalized A/B/C composition.
    fn marker_style(
        &self,
        index: usize,
        composition: TernaryPoint,
        fallback: &MarkerStyle,
    ) -> MarkerStyle;
}

impl PointMarkerStyleProvider for () {
    fn marker_style(
        &self,
        _index: usize,
        _composition: TernaryPoint,
        fallback: &MarkerStyle,
    ) -> MarkerStyle {
        fallback.clone()
    }
}

impl<F> PointMarkerStyleProvider for F
where
    F: Fn(usize, TernaryPoint) -> MarkerStyle,
{
    fn marker_style(
        &self,
        index: usize,
        composition: TernaryPoint,
        _fallback: &MarkerStyle,
    ) -> MarkerStyle {
        self(index, composition)
    }
}

/// A ternary point collection with marker and validation configuration.
///
/// `Provider` is normally `()`. Calling [`Self::point_style_provider`] stores
/// a callback without a `'static` bound and selects a complete marker style per
/// original source index.
pub struct TernaryPointSeries<I, Provider = ()> {
    points: I,
    size: u32,
    style: ShapeStyle,
    marker: MarkerShape,
    custom_style: Option<MarkerStyle>,
    style_provider: Provider,
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
            custom_style: None,
            style_provider: (),
            clip_mode: MarkerClipMode::default(),
            normalization: Normalization::RequireUnitSum,
            tolerance: Tolerance::default(),
            invalid_point_policy: InvalidPointPolicy::Error,
        }
    }
}

impl<I, Provider> TernaryPointSeries<I, Provider> {
    /// Set the marker radius/half-size in backend pixels.
    pub const fn size(mut self, size: u32) -> Self {
        self.size = size;
        self
    }

    /// Set the legacy Plotters-native marker style.
    ///
    /// This remains the compatibility path for `.marker(...).style(...)`.
    /// An explicit [`Self::marker_style`] takes precedence when set.
    pub fn style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.style = style.into();
        self
    }

    /// Select a built-in marker shape for the compatibility style path.
    pub const fn marker(mut self, marker: MarkerShape) -> Self {
        self.marker = marker;
        self
    }

    /// Set a complete scientific marker style with independent geometry, fill,
    /// partitions, divider, and outer edge.
    pub fn marker_style(mut self, style: MarkerStyle) -> Self {
        self.custom_style = Some(style);
        self
    }

    /// Replace the uniform style with a callback evaluated for each original
    /// source composition. Source indexes are never renumbered after clipping.
    pub fn point_style_provider<NewProvider>(
        self,
        provider: NewProvider,
    ) -> TernaryPointSeries<I, NewProvider> {
        TernaryPointSeries {
            points: self.points,
            size: self.size,
            style: self.style,
            marker: self.marker,
            custom_style: self.custom_style,
            style_provider: provider,
            clip_mode: self.clip_mode,
            normalization: self.normalization,
            tolerance: self.tolerance,
            invalid_point_policy: self.invalid_point_policy,
        }
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

    /// Return the configured backend-pixel half-size.
    pub const fn marker_size(&self) -> u32 {
        self.size
    }

    /// Return the compatibility Plotters style.
    pub const fn legacy_marker_style(&self) -> ShapeStyle {
        self.style
    }

    /// Return the compatibility marker shape.
    pub const fn marker_shape(&self) -> MarkerShape {
        self.marker
    }

    /// Return any explicit uniform scientific style.
    pub fn configured_marker_style(&self) -> Option<&MarkerStyle> {
        self.custom_style.as_ref()
    }

    /// Return the selected marker clipping policy.
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
        Option<MarkerStyle>,
        Provider,
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
            self.custom_style,
            self.style_provider,
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
    fn point_configuration_retains_legacy_style_and_marker_policy() {
        let series = TernaryPointSeries::new(Vec::<crate::TernaryPoint>::new())
            .size(9)
            .style(RED.filled())
            .marker(MarkerShape::Triangle)
            .clip_mode(MarkerClipMode::None)
            .normalization(Normalization::Normalize)
            .invalid_point_policy(InvalidPointPolicy::Break);
        assert_eq!(series.marker_size(), 9);
        assert_eq!(series.legacy_marker_style().color, RED.to_rgba());
        assert_eq!(series.marker_shape(), MarkerShape::Triangle);
        assert_eq!(series.marker_clip_mode(), MarkerClipMode::None);
    }

    #[test]
    fn a_per_point_provider_keeps_original_index() {
        let fallback = MarkerStyle::solid(MarkerShape::Circle, RED, BLACK).unwrap();
        let provider = |index, _| {
            if index == 3 {
                MarkerStyle::solid(MarkerShape::Diamond, BLUE, BLACK).unwrap()
            } else {
                fallback.clone()
            }
        };
        let series = TernaryPointSeries::new(Vec::<crate::TernaryPoint>::new())
            .marker_style(fallback.clone())
            .point_style_provider(provider);
        let (_, _, _, _, explicit, provider, _, _, _, _) = series.into_parts();
        assert_eq!(explicit, Some(fallback.clone()));
        assert_eq!(
            provider
                .marker_style(3, TernaryPoint::new(0.2, 0.3, 0.5), &fallback)
                .geometry
                .shape(),
            MarkerShape::Diamond
        );
    }
}
