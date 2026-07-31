use plotters::style::{BLACK, Color, ShapeStyle};

use crate::contour::ContourSet;

/// Plotters adapter for backend-neutral contour paths.
///
/// Complete contour paths are computed before this series is constructed. The
/// adapter projects and mathematically clips them through the same pipeline as
/// exact ternary line series, while retaining native Plotters annotations.
type LevelStyleProvider<'a> = Box<dyn Fn(f64) -> ShapeStyle + 'a>;
type ContourSeriesParts<'a> = (&'a ContourSet, ShapeStyle, Option<LevelStyleProvider<'a>>);

/// Plotters adapter for a precomputed [`ContourSet`].
///
/// It projects semantic contour paths, applies the chart viewport clipper, and
/// returns native Plotters annotations through [`crate::TernaryChart::draw_series`].
pub struct TernaryContourSeries<'a> {
    contours: &'a ContourSet,
    uniform_style: ShapeStyle,
    style_provider: Option<LevelStyleProvider<'a>>,
}

impl<'a> TernaryContourSeries<'a> {
    /// Create a contour series with a black two-pixel default stroke.
    pub fn new(contours: &'a ContourSet) -> Self {
        Self {
            contours,
            uniform_style: BLACK.stroke_width(2),
            style_provider: None,
        }
    }
    /// Use one Plotters-native stroke style for every level.
    pub fn style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.uniform_style = style.into();
        self
    }
    /// Select an owned style from each scalar level at draw time.
    pub fn style_for_level<F>(mut self, provider: F) -> Self
    where
        F: Fn(f64) -> ShapeStyle + 'a,
    {
        self.style_provider = Some(Box::new(provider));
        self
    }
    pub(crate) fn into_parts(self) -> ContourSeriesParts<'a> {
        (self.contours, self.uniform_style, self.style_provider)
    }
}
