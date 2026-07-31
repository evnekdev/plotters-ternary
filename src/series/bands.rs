use std::sync::Arc;

use plotters::style::{RGBAColor, ShapeStyle};
use ternary_contours::{ContourBand, ContourBandSet, RegularTernaryScalarField};

use super::SeriesError;

type BandStyleCallback = dyn Fn(usize, Option<f64>, Option<f64>) -> ShapeStyle + Send + Sync;
type ScalarColorMap = dyn Fn(f64) -> RGBAColor + Send + Sync;
type ScalarMapParts<'a> = (
    &'a RegularTernaryScalarField,
    Option<(f64, f64)>,
    ScalarMapResolution,
    f64,
    bool,
    Arc<ScalarColorMap>,
);

/// Select which band boundaries are stroked after the non-overlapping fills.
///
/// OuterRegions draws each assembled exterior and hole boundary once.
/// Fragments is primarily diagnostic: it can make internal fragment seams
/// visible and is therefore never the default.
#[derive(Clone, Copy)]
pub enum ContourBandBorderMode {
    /// Do not stroke band boundaries. Use isolines for inter-band boundaries.
    None,
    /// Stroke only assembled region and hole boundaries.
    OuterRegions(ShapeStyle),
    /// Stroke every clipped elementary-triangle fragment.
    Fragments(ShapeStyle),
}

#[derive(Clone)]
pub enum ContourBandStylePolicy {
    Uniform(ShapeStyle),
    Ordered(Vec<ShapeStyle>),
    ByBand(Arc<BandStyleCallback>),
}

impl ContourBandStylePolicy {
    pub(crate) fn style_for(
        &self,
        index: usize,
        band: &ContourBand,
    ) -> Result<ShapeStyle, SeriesError> {
        match self {
            Self::Uniform(style) => Ok(*style),
            Self::Ordered(styles) if styles.is_empty() => {
                Err(SeriesError::EmptyContourBandStyleCollection)
            }
            Self::Ordered(styles) => Ok(styles[index % styles.len()]),
            Self::ByBand(callback) => Ok(callback(index, band.lower, band.upper)),
        }
    }
}

pub struct TernaryContourBandSeries<'a> {
    bands: &'a ContourBandSet,
    styles: ContourBandStylePolicy,
    border: ContourBandBorderMode,
}

impl<'a> TernaryContourBandSeries<'a> {
    pub fn new<S: Into<ShapeStyle>>(bands: &'a ContourBandSet, style: S) -> Self {
        Self {
            bands,
            styles: ContourBandStylePolicy::Uniform(style.into()),
            border: ContourBandBorderMode::None,
        }
    }
    pub fn style_by_band<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, Option<f64>, Option<f64>) -> ShapeStyle + Send + Sync + 'static,
    {
        self.styles = ContourBandStylePolicy::ByBand(Arc::new(callback));
        self
    }
    pub fn styles(mut self, styles: Vec<ShapeStyle>) -> Self {
        self.styles = ContourBandStylePolicy::Ordered(styles);
        self
    }
    /// Stroke each assembled region boundary once, including hole boundaries.
    ///
    /// This preserves the legacy method name while avoiding doubled internal
    /// fragment seams.
    pub fn border_style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.border = ContourBandBorderMode::OuterRegions(style.into());
        self
    }

    /// Stroke every elementary-triangle band fragment.
    ///
    /// This is useful for diagnostics only; ordinary figures should use
    /// border_style or isolines instead.
    pub fn fragment_border_style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.border = ContourBandBorderMode::Fragments(style.into());
        self
    }

    /// Disable all band-border strokes.
    pub const fn without_border(mut self) -> Self {
        self.border = ContourBandBorderMode::None;
        self
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        &'a ContourBandSet,
        ContourBandStylePolicy,
        ContourBandBorderMode,
    ) {
        (self.bands, self.styles, self.border)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Resolution of the flat-colour microtriangle approximation.
///
/// Adaptive currently means a deterministic uniform refinement selected by its
/// maximum depth; it does not inspect colours per triangle. This keeps output
/// bounded and backend-independent until a true adaptive policy is introduced.
pub enum ScalarMapResolution {
    Fixed { subdivisions_per_edge: usize },
    Adaptive { max_depth: u8 },
}
impl Default for ScalarMapResolution {
    fn default() -> Self {
        Self::Fixed {
            subdivisions_per_edge: 4,
        }
    }
}
impl ScalarMapResolution {
    pub(crate) fn intervals(self) -> Result<usize, SeriesError> {
        match self {
            Self::Fixed {
                subdivisions_per_edge,
            } if (1..=64).contains(&subdivisions_per_edge) => Ok(subdivisions_per_edge),
            Self::Adaptive { max_depth } if max_depth <= 6 => Ok(1usize << max_depth),
            Self::Fixed {
                subdivisions_per_edge,
            } => Err(SeriesError::InvalidScalarMapResolution {
                value: subdivisions_per_edge,
            }),
            Self::Adaptive { max_depth } => Err(SeriesError::InvalidScalarMapResolution {
                value: usize::from(max_depth),
            }),
        }
    }
}

pub struct TernaryScalarMapSeries<'a> {
    field: &'a RegularTernaryScalarField,
    range: Option<(f64, f64)>,
    resolution: ScalarMapResolution,
    opacity: f64,
    reversed: bool,
    color_map: Arc<ScalarColorMap>,
}
impl<'a> TernaryScalarMapSeries<'a> {
    pub fn new(field: &'a RegularTernaryScalarField) -> Self {
        Self {
            field,
            range: None,
            resolution: ScalarMapResolution::default(),
            opacity: 1.0,
            reversed: false,
            color_map: Arc::new(default_color_map),
        }
    }
    pub fn range(mut self, minimum: f64, maximum: f64) -> Self {
        self.range = Some((minimum, maximum));
        self
    }
    pub fn resolution(mut self, resolution: ScalarMapResolution) -> Self {
        self.resolution = resolution;
        self
    }
    pub fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }
    /// Reverse the normalised colour-map coordinate without changing field data.
    pub const fn reversed(mut self) -> Self {
        self.reversed = !self.reversed;
        self
    }

    pub fn color_map<F>(mut self, color_map: F) -> Self
    where
        F: Fn(f64) -> RGBAColor + Send + Sync + 'static,
    {
        self.color_map = Arc::new(color_map);
        self
    }
    pub(crate) fn into_parts(self) -> ScalarMapParts<'a> {
        (
            self.field,
            self.range,
            self.resolution,
            self.opacity,
            self.reversed,
            self.color_map,
        )
    }
}
/// Number of flat microtriangles emitted for one elementary field triangle.
pub(crate) const fn microtriangle_count(intervals: usize) -> usize {
    intervals * intervals
}

pub(crate) fn default_color_map(value: f64) -> RGBAColor {
    let value = value.clamp(0.0, 1.0);
    RGBAColor(
        (255.0 * value) as u8,
        (80.0 * (1.0 - (2.0 * value - 1.0).abs())) as u8,
        (255.0 * (1.0 - value)) as u8,
        1.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotters::style::{BLACK, Color, RED};

    #[test]
    fn ordered_band_styles_cycle_and_empty_collections_are_rejected() {
        let field = RegularTernaryScalarField::new(1, vec![0.0, 1.0, 2.0]).unwrap();
        let bands = ContourBandSet::compute(
            &field,
            &[1.0],
            ternary_contours::ContourBandOptions::linear(),
        )
        .unwrap();
        let band = &bands.bands[0];
        let styles = ContourBandStylePolicy::Ordered(vec![BLACK.filled(), RED.filled()]);
        assert_eq!(styles.style_for(0, band).unwrap(), BLACK.filled());
        assert_eq!(styles.style_for(3, band).unwrap(), RED.filled());
        assert!(matches!(
            ContourBandStylePolicy::Ordered(Vec::new()).style_for(0, band),
            Err(SeriesError::EmptyContourBandStyleCollection)
        ));
    }

    #[test]
    fn scalar_map_resolution_is_bounded_and_deterministic() {
        assert_eq!(
            ScalarMapResolution::Fixed {
                subdivisions_per_edge: 4
            }
            .intervals()
            .unwrap(),
            4
        );
        assert_eq!(
            ScalarMapResolution::Adaptive { max_depth: 3 }
                .intervals()
                .unwrap(),
            8
        );
        assert!(
            ScalarMapResolution::Fixed {
                subdivisions_per_edge: 0
            }
            .intervals()
            .is_err()
        );
        assert_eq!(microtriangle_count(4), 16);
    }

    #[test]
    fn reverse_map_toggles_without_changing_other_options() {
        let field = RegularTernaryScalarField::new(1, vec![0.0, 1.0, 2.0]).unwrap();
        let (_, _, _, _, reversed, _) = TernaryScalarMapSeries::new(&field).reversed().into_parts();
        assert!(reversed);
    }
}
