use plotters::style::{BLACK, Color, RGBAColor, ShapeStyle};

use crate::{contour::ContourSet, series::SeriesError};

/// Maps contour levels to Plotters-native line styles.
pub enum ContourStylePolicy<'a> {
    /// Use one style for every contour level.
    Uniform(ShapeStyle),
    /// Apply styles in level order, cycling when there are more levels than styles.
    Ordered(Vec<ShapeStyle>),
    /// Select a style directly from the scalar level value.
    ByLevel(Box<dyn Fn(f64) -> ShapeStyle + 'a>),
    /// Map the normalized level range to an RGBA colour and fixed stroke width.
    Continuous {
        minimum: f64,
        maximum: f64,
        stroke_width: u32,
        map: Box<dyn Fn(f64) -> RGBAColor + 'a>,
    },
}

impl Default for ContourStylePolicy<'_> {
    fn default() -> Self {
        Self::Uniform(BLACK.stroke_width(2))
    }
}

impl<'a> ContourStylePolicy<'a> {
    /// Construct a uniform style policy.
    pub fn uniform<S: Into<ShapeStyle>>(style: S) -> Self {
        Self::Uniform(style.into())
    }

    /// Construct a cycling ordered palette.
    pub fn ordered<I, S>(styles: I) -> Result<Self, SeriesError>
    where
        I: IntoIterator<Item = S>,
        S: Into<ShapeStyle>,
    {
        let styles = styles.into_iter().map(Into::into).collect::<Vec<_>>();
        if styles.is_empty() {
            return Err(SeriesError::EmptyContourStyleCollection);
        }
        Ok(Self::Ordered(styles))
    }

    /// Construct a callback-driven style policy.
    pub fn by_level<F>(provider: F) -> Self
    where
        F: Fn(f64) -> ShapeStyle + 'a,
    {
        Self::ByLevel(Box::new(provider))
    }

    /// Construct a continuous colour-map policy.
    ///
    /// The callback receives a normalized value in `[0, 1]`.
    pub fn continuous<F>(
        minimum: f64,
        maximum: f64,
        stroke_width: u32,
        map: F,
    ) -> Result<Self, SeriesError>
    where
        F: Fn(f64) -> RGBAColor + 'a,
    {
        if !minimum.is_finite() || !maximum.is_finite() || maximum <= minimum || stroke_width == 0 {
            return Err(SeriesError::InvalidContourColorRange {
                minimum,
                maximum,
                stroke_width,
            });
        }
        Ok(Self::Continuous {
            minimum,
            maximum,
            stroke_width,
            map: Box::new(map),
        })
    }

    /// Resolve the style for one level index and value.
    pub fn style_for(&self, index: usize, level: f64) -> Result<ShapeStyle, SeriesError> {
        let style = match self {
            Self::Uniform(style) => *style,
            Self::Ordered(styles) => {
                if styles.is_empty() {
                    return Err(SeriesError::EmptyContourStyleCollection);
                }
                styles[index % styles.len()]
            }
            Self::ByLevel(provider) => provider(level),
            Self::Continuous {
                minimum,
                maximum,
                stroke_width,
                map,
            } => {
                if !minimum.is_finite()
                    || !maximum.is_finite()
                    || maximum <= minimum
                    || *stroke_width == 0
                {
                    return Err(SeriesError::InvalidContourColorRange {
                        minimum: *minimum,
                        maximum: *maximum,
                        stroke_width: *stroke_width,
                    });
                }
                let normalized = ((level - minimum) / (maximum - minimum)).clamp(0.0, 1.0);
                map(normalized).stroke_width(*stroke_width)
            }
        };
        Ok(style)
    }
}

/// Selects which scalar levels create native Plotters legend entries.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum ContourLegendPolicy {
    /// Register no automatic per-level legend entries.
    #[default]
    None,
    /// Register every level.
    EveryLevel,
    /// Register the listed scalar levels, matched with a small relative tolerance.
    Selected(Vec<f64>),
    /// Register every `n`th level in sorted contour order.
    EveryNth(usize),
}

impl ContourLegendPolicy {
    pub(crate) fn selected(&self, index: usize, level: f64) -> Result<bool, SeriesError> {
        match self {
            Self::None => Ok(false),
            Self::EveryLevel => Ok(true),
            Self::Selected(values) => Ok(values.iter().any(|candidate| {
                let scale = candidate.abs().max(level.abs()).max(1.0);
                (candidate - level).abs() <= 1.0e-12 * scale
            })),
            Self::EveryNth(0) => Err(SeriesError::InvalidContourLegendStride { stride: 0 }),
            Self::EveryNth(stride) => Ok(index.is_multiple_of(*stride)),
        }
    }
}

pub(crate) type LevelFormatter<'a> = Box<dyn Fn(f64) -> String + 'a>;

pub(crate) struct ContourSeriesParts<'a> {
    pub contours: &'a ContourSet,
    pub styles: ContourStylePolicy<'a>,
    pub legend: ContourLegendPolicy,
    pub formatter: LevelFormatter<'a>,
}

/// Plotters adapter for a precomputed [`ContourSet`].
///
/// The input set is immutable final numerical geometry. This adapter only
/// projects paths, clips them for display, selects styles, and registers native
/// Plotters series annotations.
pub struct TernaryContourSeries<'a> {
    contours: &'a ContourSet,
    styles: ContourStylePolicy<'a>,
    legend: ContourLegendPolicy,
    formatter: LevelFormatter<'a>,
}

impl<'a> TernaryContourSeries<'a> {
    /// Create a contour series with a black two-pixel default stroke.
    pub fn new(contours: &'a ContourSet) -> Self {
        Self {
            contours,
            styles: ContourStylePolicy::default(),
            legend: ContourLegendPolicy::None,
            formatter: Box::new(|level| format!("{level}")),
        }
    }

    /// Use one Plotters-native stroke style for every level.
    pub fn style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.styles = ContourStylePolicy::uniform(style);
        self
    }

    /// Select an owned style from each scalar level at draw time.
    pub fn style_for_level<F>(mut self, provider: F) -> Self
    where
        F: Fn(f64) -> ShapeStyle + 'a,
    {
        self.styles = ContourStylePolicy::by_level(provider);
        self
    }

    /// Alias for [`Self::style_for_level`] with level-oriented naming.
    pub fn style_by_level<F>(self, provider: F) -> Self
    where
        F: Fn(f64) -> ShapeStyle + 'a,
    {
        self.style_for_level(provider)
    }

    /// Use an explicit style policy.
    pub fn style_policy(mut self, policy: ContourStylePolicy<'a>) -> Self {
        self.styles = policy;
        self
    }

    /// Apply a cycling ordered style collection in contour-level order.
    pub fn ordered_styles<I, S>(mut self, styles: I) -> Result<Self, SeriesError>
    where
        I: IntoIterator<Item = S>,
        S: Into<ShapeStyle>,
    {
        self.styles = ContourStylePolicy::ordered(styles)?;
        Ok(self)
    }

    /// Apply a continuous normalized colour map over a scalar range.
    pub fn color_map<F>(
        mut self,
        minimum: f64,
        maximum: f64,
        stroke_width: u32,
        map: F,
    ) -> Result<Self, SeriesError>
    where
        F: Fn(f64) -> RGBAColor + 'a,
    {
        self.styles = ContourStylePolicy::continuous(minimum, maximum, stroke_width, map)?;
        Ok(self)
    }

    /// Select automatic native Plotters legend entries.
    pub fn legend_policy(mut self, policy: ContourLegendPolicy) -> Self {
        self.legend = policy;
        self
    }

    /// Format automatic legend labels from scalar values.
    pub fn level_formatter<F, S>(mut self, formatter: F) -> Self
    where
        F: Fn(f64) -> S + 'a,
        S: Into<String>,
    {
        self.formatter = Box::new(move |level| formatter(level).into());
        self
    }

    pub(crate) fn into_parts(self) -> ContourSeriesParts<'a> {
        ContourSeriesParts {
            contours: self.contours,
            styles: self.styles,
            legend: self.legend,
            formatter: self.formatter,
        }
    }
}

#[cfg(test)]
mod tests {
    use plotters::style::{BLUE, Color, RED, RGBColor};

    use super::*;

    #[test]
    fn style_selection_uses_exact_levels_ordered_palettes_and_continuous_maps() {
        let callback = ContourStylePolicy::by_level(|level| {
            if level == 2.5 {
                RED.stroke_width(4)
            } else {
                BLUE.stroke_width(1)
            }
        });
        assert_eq!(callback.style_for(9, 2.5).unwrap(), RED.stroke_width(4));

        let ordered =
            ContourStylePolicy::ordered([RED.stroke_width(1), BLUE.stroke_width(2)]).unwrap();
        assert_eq!(ordered.style_for(0, 10.0).unwrap(), RED.stroke_width(1));
        assert_eq!(ordered.style_for(3, 40.0).unwrap(), BLUE.stroke_width(2));

        let continuous = ContourStylePolicy::continuous(0.0, 100.0, 3, |t| {
            RGBColor((255.0 * t).round() as u8, 0, 0).to_rgba()
        })
        .unwrap();
        assert_eq!(
            continuous.style_for(0, 50.0).unwrap(),
            RGBColor(128, 0, 0).stroke_width(3)
        );

        assert!(
            ContourStylePolicy::Ordered(Vec::new())
                .style_for(0, 0.0)
                .is_err()
        );
        let invalid_continuous = ContourStylePolicy::Continuous {
            minimum: 1.0,
            maximum: 1.0,
            stroke_width: 2,
            map: Box::new(|_| RED.to_rgba()),
        };
        assert!(invalid_continuous.style_for(0, 1.0).is_err());
    }

    #[test]
    fn legend_policies_select_deterministic_level_rows() {
        assert!(!ContourLegendPolicy::None.selected(0, 1.0).unwrap());
        assert!(ContourLegendPolicy::EveryLevel.selected(4, 1.0).unwrap());
        assert!(
            ContourLegendPolicy::Selected(vec![2.0])
                .selected(9, 2.0 + 1.0e-13)
                .unwrap()
        );
        assert!(ContourLegendPolicy::EveryNth(3).selected(6, 9.0).unwrap());
        assert!(!ContourLegendPolicy::EveryNth(3).selected(5, 9.0).unwrap());
        assert!(ContourLegendPolicy::EveryNth(0).selected(0, 0.0).is_err());
    }
}
