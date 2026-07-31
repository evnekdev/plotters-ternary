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
    Arc<ScalarColorMap>,
);

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
    border: Option<ShapeStyle>,
}

impl<'a> TernaryContourBandSeries<'a> {
    pub fn new<S: Into<ShapeStyle>>(bands: &'a ContourBandSet, style: S) -> Self {
        Self {
            bands,
            styles: ContourBandStylePolicy::Uniform(style.into()),
            border: None,
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
    pub fn border_style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.border = Some(style.into());
        self
    }
    pub(crate) fn into_parts(
        self,
    ) -> (
        &'a ContourBandSet,
        ContourBandStylePolicy,
        Option<ShapeStyle>,
    ) {
        (self.bands, self.styles, self.border)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScalarMapResolution {
    Fixed { subdivisions_per_edge: usize },
    Adaptive { max_depth: u8 },
}
impl Default for ScalarMapResolution {
    fn default() -> Self {
        Self::Fixed {
            subdivisions_per_edge: 6,
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
    color_map: Arc<ScalarColorMap>,
}
impl<'a> TernaryScalarMapSeries<'a> {
    pub fn new(field: &'a RegularTernaryScalarField) -> Self {
        Self {
            field,
            range: None,
            resolution: ScalarMapResolution::default(),
            opacity: 1.0,
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
            self.color_map,
        )
    }
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
        let band = ContourBand {
            lower: Some(1.0),
            upper: Some(2.0),
            regions: Vec::new(),
        };
        let styles = ContourBandStylePolicy::Ordered(vec![BLACK.filled(), RED.filled()]);
        assert_eq!(styles.style_for(0, &band).unwrap(), BLACK.filled());
        assert_eq!(styles.style_for(3, &band).unwrap(), RED.filled());
        assert!(matches!(
            ContourBandStylePolicy::Ordered(Vec::new()).style_for(0, &band),
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
    }
}
