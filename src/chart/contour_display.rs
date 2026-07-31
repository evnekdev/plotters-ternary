//! Chart-space contour colour bars and portable labels.
//!
//! These APIs consume final numerical contour coordinates and never mutate or
//! reconstruct them.

use std::f64::consts::{FRAC_PI_2, PI};
use std::fmt;

use plotters::backend::DrawingBackend;
use plotters::element::{EmptyElement, PathElement, Rectangle, Text};
use plotters::style::{BLACK, Color, FontStyle, RGBAColor, ShapeStyle, WHITE};
use ternary_contours::{ContourPath, ContourSet};

use crate::coord::{Normalization, TernaryCartesian, TernaryPoint};
use crate::series::{InvalidPointPolicy, SeriesError, prepare_polyline};

use super::{AxisTextStyle, RotatedText, TernaryChart, TernaryChartError};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[non_exhaustive]
pub enum ContourLabelMode {
    #[default]
    Tangent,
    Curved,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ContourLabelAnchor {
    pub level: f64,
    pub path_index: usize,
    pub arc_fraction: f64,
}
impl ContourLabelAnchor {
    pub const fn new(level: f64, path_index: usize, arc_fraction: f64) -> Self {
        Self {
            level,
            path_index,
            arc_fraction,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub enum ContourLabelPlacement {
    #[default]
    Automatic,
    Repeated {
        spacing: f64,
    },
    Manual(Vec<ContourLabelAnchor>),
}

#[derive(Clone)]
pub struct ContourLabelStyle {
    text: AxisTextStyle,
    halo_color: Option<RGBAColor>,
    halo_width: u32,
    normal_offset: f64,
}
impl Default for ContourLabelStyle {
    fn default() -> Self {
        Self {
            text: AxisTextStyle::sans_serif(19, FontStyle::Bold, BLACK.to_rgba()),
            halo_color: Some(WHITE.to_rgba()),
            halo_width: 2,
            normal_offset: 0.0,
        }
    }
}
impl ContourLabelStyle {
    pub fn new(text: AxisTextStyle) -> Self {
        Self {
            text,
            ..Self::default()
        }
    }
    pub fn halo<C: Color>(mut self, color: C, width: u32) -> Self {
        self.halo_color = Some(color.to_rgba());
        self.halo_width = width;
        self
    }
    pub fn without_halo(mut self) -> Self {
        self.halo_color = None;
        self.halo_width = 0;
        self
    }
    pub fn normal_offset(mut self, pixels: f64) -> Self {
        self.normal_offset = pixels;
        self
    }
    pub fn text_style(&self) -> &AxisTextStyle {
        &self.text
    }
    pub const fn halo_width(&self) -> u32 {
        self.halo_width
    }
    pub const fn normal_offset_pixels(&self) -> f64 {
        self.normal_offset
    }
}

type LabelFormatter<'a> = Box<dyn Fn(f64) -> String + 'a>;
/// Formatting, placement, clearance and appearance policy for contour labels.
pub struct ContourLabelConfig<'a> {
    mode: ContourLabelMode,
    placement: ContourLabelPlacement,
    style: ContourLabelStyle,
    formatter: LabelFormatter<'a>,
    minimum_visible_length: f64,
    endpoint_clearance: f64,
    viewport_clearance: f64,
    maximum_curvature_degrees: f64,
    collision_padding: f64,
}
impl Default for ContourLabelConfig<'_> {
    fn default() -> Self {
        Self {
            mode: ContourLabelMode::Tangent,
            placement: ContourLabelPlacement::Automatic,
            style: ContourLabelStyle::default(),
            formatter: Box::new(|level| format!("{level:.3}")),
            minimum_visible_length: 90.0,
            endpoint_clearance: 20.0,
            viewport_clearance: 8.0,
            maximum_curvature_degrees: 32.0,
            collision_padding: 5.0,
        }
    }
}
impl<'a> ContourLabelConfig<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn mode(mut self, value: ContourLabelMode) -> Self {
        self.mode = value;
        self
    }
    pub fn placement(mut self, value: ContourLabelPlacement) -> Self {
        self.placement = value;
        self
    }
    pub fn style(mut self, value: ContourLabelStyle) -> Self {
        self.style = value;
        self
    }
    pub fn formatter<F, S>(mut self, value: F) -> Self
    where
        F: Fn(f64) -> S + 'a,
        S: Into<String>,
    {
        self.formatter = Box::new(move |level| value(level).into());
        self
    }
    pub fn minimum_visible_length(mut self, value: f64) -> Self {
        self.minimum_visible_length = value;
        self
    }
    pub fn endpoint_clearance(mut self, value: f64) -> Self {
        self.endpoint_clearance = value;
        self
    }
    pub fn viewport_clearance(mut self, value: f64) -> Self {
        self.viewport_clearance = value;
        self
    }
    pub fn maximum_curvature_degrees(mut self, value: f64) -> Self {
        self.maximum_curvature_degrees = value;
        self
    }
    pub fn collision_padding(mut self, value: f64) -> Self {
        self.collision_padding = value;
        self
    }
    fn validate(&self) -> Result<(), ContourDisplayError> {
        for (name, value, positive) in [
            ("minimum_visible_length", self.minimum_visible_length, false),
            ("endpoint_clearance", self.endpoint_clearance, false),
            ("viewport_clearance", self.viewport_clearance, false),
            (
                "maximum_curvature_degrees",
                self.maximum_curvature_degrees,
                true,
            ),
            ("collision_padding", self.collision_padding, false),
        ] {
            if !value.is_finite() || value < 0.0 || (positive && value == 0.0) {
                return Err(ContourDisplayError::InvalidOption { name, value });
            }
        }
        if !self.style.normal_offset.is_finite() {
            return Err(ContourDisplayError::InvalidOption {
                name: "normal_offset",
                value: self.style.normal_offset,
            });
        }
        if let ContourLabelPlacement::Repeated { spacing } = self.placement
            && (!spacing.is_finite() || spacing <= 0.0)
        {
            return Err(ContourDisplayError::InvalidOption {
                name: "repeated_label_spacing",
                value: spacing,
            });
        }
        if let ContourLabelPlacement::Manual(anchors) = &self.placement {
            for anchor in anchors {
                if !anchor.level.is_finite()
                    || !anchor.arc_fraction.is_finite()
                    || !(0.0..=1.0).contains(&anchor.arc_fraction)
                {
                    return Err(ContourDisplayError::InvalidManualAnchor {
                        level: anchor.level,
                        path_index: anchor.path_index,
                        arc_fraction: anchor.arc_fraction,
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ContourColorBarOrientation {
    #[default]
    Vertical,
    Horizontal,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ContourColorBarPosition {
    UpperLeft,
    #[default]
    UpperRight,
    LowerLeft,
    LowerRight,
}
type ColorMap<'a> = Box<dyn Fn(f64) -> RGBAColor + 'a>;
/// Continuous scalar-level key drawn in chart layout space.
pub struct ContourColorBar<'a> {
    minimum: f64,
    maximum: f64,
    map: ColorMap<'a>,
    orientation: ContourColorBarOrientation,
    position: ContourColorBarPosition,
    length: u32,
    thickness: u32,
    margin: i32,
    ticks: Vec<f64>,
    formatter: LabelFormatter<'a>,
    title: Option<String>,
    text_style: AxisTextStyle,
    border_style: ShapeStyle,
}
impl<'a> ContourColorBar<'a> {
    pub fn new<F>(minimum: f64, maximum: f64, map: F) -> Result<Self, ContourDisplayError>
    where
        F: Fn(f64) -> RGBAColor + 'a,
    {
        if !minimum.is_finite() || !maximum.is_finite() || maximum <= minimum {
            return Err(ContourDisplayError::InvalidColorBarRange { minimum, maximum });
        }
        Ok(Self {
            minimum,
            maximum,
            map: Box::new(map),
            orientation: ContourColorBarOrientation::Vertical,
            position: ContourColorBarPosition::UpperRight,
            length: 220,
            thickness: 18,
            margin: 18,
            ticks: (0..=4)
                .map(|i| minimum + (maximum - minimum) * f64::from(i) / 4.0)
                .collect(),
            formatter: Box::new(|value| format!("{value:.2}")),
            title: None,
            text_style: AxisTextStyle::sans_serif(16, FontStyle::Normal, BLACK.to_rgba()),
            border_style: BLACK.stroke_width(1),
        })
    }
    pub fn orientation(mut self, value: ContourColorBarOrientation) -> Self {
        self.orientation = value;
        self
    }
    pub fn position(mut self, value: ContourColorBarPosition) -> Self {
        self.position = value;
        self
    }
    pub fn size(mut self, length: u32, thickness: u32) -> Self {
        self.length = length;
        self.thickness = thickness;
        self
    }
    pub fn margin(mut self, value: i32) -> Self {
        self.margin = value;
        self
    }
    pub fn tick_values(mut self, value: Vec<f64>) -> Self {
        self.ticks = value;
        self
    }
    pub fn formatter<F, S>(mut self, value: F) -> Self
    where
        F: Fn(f64) -> S + 'a,
        S: Into<String>,
    {
        self.formatter = Box::new(move |v| value(v).into());
        self
    }
    pub fn title<S: Into<String>>(mut self, value: S) -> Self {
        self.title = Some(value.into());
        self
    }
    pub fn text_style(mut self, value: AxisTextStyle) -> Self {
        self.text_style = value;
        self
    }
    pub fn border_style<S: Into<ShapeStyle>>(mut self, value: S) -> Self {
        self.border_style = value.into();
        self
    }
    fn validate(&self) -> Result<(), ContourDisplayError> {
        if self.length == 0 || self.thickness == 0 {
            return Err(ContourDisplayError::InvalidColorBarSize {
                length: self.length,
                thickness: self.thickness,
            });
        }
        if self
            .ticks
            .iter()
            .any(|v| !v.is_finite() || *v < self.minimum || *v > self.maximum)
        {
            return Err(ContourDisplayError::InvalidColorBarTicks);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ContourDisplayError {
    InvalidOption {
        name: &'static str,
        value: f64,
    },
    InvalidManualAnchor {
        level: f64,
        path_index: usize,
        arc_fraction: f64,
    },
    ManualLevelNotFound {
        level: f64,
    },
    ManualPathNotFound {
        level: f64,
        path_index: usize,
    },
    InvalidColorBarRange {
        minimum: f64,
        maximum: f64,
    },
    InvalidColorBarSize {
        length: u32,
        thickness: u32,
    },
    InvalidColorBarTicks,
}
impl fmt::Display for ContourDisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOption { name, value } => {
                write!(f, "invalid contour display option {name}: {value:?}")
            }
            Self::InvalidManualAnchor {
                level,
                path_index,
                arc_fraction,
            } => write!(
                f,
                "invalid manual contour anchor at level {level:?}, path {path_index}, fraction {arc_fraction:?}"
            ),
            Self::ManualLevelNotFound { level } => {
                write!(f, "manual contour-label level was not found: {level:?}")
            }
            Self::ManualPathNotFound { level, path_index } => write!(
                f,
                "manual contour-label path {path_index} was not found at level {level:?}"
            ),
            Self::InvalidColorBarRange { minimum, maximum } => write!(
                f,
                "colour-bar range must be finite and increasing: {minimum:?}..{maximum:?}"
            ),
            Self::InvalidColorBarSize { length, thickness } => write!(
                f,
                "colour-bar dimensions must be nonzero: {length}x{thickness}"
            ),
            Self::InvalidColorBarTicks => write!(
                f,
                "colour-bar ticks must be finite and within the configured range"
            ),
        }
    }
}
impl std::error::Error for ContourDisplayError {}
#[derive(Clone, Copy, Debug, PartialEq)]
struct PixelPoint {
    x: f64,
    y: f64,
}
impl PixelPoint {
    fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
    fn interpolate(self, other: Self, t: f64) -> Self {
        Self {
            x: self.x + t * (other.x - self.x),
            y: self.y + t * (other.y - self.y),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
struct Bounds {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}
impl Bounds {
    fn padded(self, p: f64) -> Self {
        Self {
            x_min: self.x_min - p,
            x_max: self.x_max + p,
            y_min: self.y_min - p,
            y_max: self.y_max + p,
        }
    }
    fn overlaps(self, o: Self) -> bool {
        self.x_min < o.x_max && self.x_max > o.x_min && self.y_min < o.y_max && self.y_max > o.y_min
    }
    fn inside(self, o: Self, c: f64) -> bool {
        self.x_min >= o.x_min + c
            && self.x_max <= o.x_max - c
            && self.y_min >= o.y_min + c
            && self.y_max <= o.y_max - c
    }
    fn union(self, o: Self) -> Self {
        Self {
            x_min: self.x_min.min(o.x_min),
            x_max: self.x_max.max(o.x_max),
            y_min: self.y_min.min(o.y_min),
            y_max: self.y_max.max(o.y_max),
        }
    }
}
#[derive(Clone)]
struct ProjectedPath {
    logical: Vec<TernaryCartesian>,
    pixels: Vec<PixelPoint>,
    cumulative: Vec<f64>,
}
impl ProjectedPath {
    fn new<DB: DrawingBackend>(
        chart: &TernaryChart<'_, DB>,
        logical: Vec<TernaryCartesian>,
    ) -> Self {
        let pixels = logical
            .iter()
            .map(|p| {
                let (x, y) = chart.plotting_area().map_coordinate(&(p.x, p.y));
                PixelPoint {
                    x: f64::from(x),
                    y: f64::from(y),
                }
            })
            .collect::<Vec<_>>();
        let mut cumulative = Vec::with_capacity(pixels.len());
        cumulative.push(0.0);
        for pair in pixels.windows(2) {
            cumulative.push(cumulative.last().copied().unwrap_or(0.0) + pair[0].distance(pair[1]));
        }
        Self {
            logical,
            pixels,
            cumulative,
        }
    }
    fn length(&self) -> f64 {
        self.cumulative.last().copied().unwrap_or(0.0)
    }
    fn sample(&self, distance: f64) -> Option<(TernaryCartesian, PixelPoint)> {
        if self.logical.len() < 2 {
            return None;
        }
        let distance = distance.clamp(0.0, self.length());
        let index = self
            .cumulative
            .partition_point(|v| *v < distance)
            .min(self.cumulative.len() - 1);
        let segment = index.saturating_sub(1).min(self.logical.len() - 2);
        let (start, end) = (self.cumulative[segment], self.cumulative[segment + 1]);
        let t = if end > start {
            (distance - start) / (end - start)
        } else {
            0.0
        };
        let (a, b) = (self.logical[segment], self.logical[segment + 1]);
        Some((
            TernaryCartesian {
                x: a.x + t * (b.x - a.x),
                y: a.y + t * (b.y - a.y),
            },
            self.pixels[segment].interpolate(self.pixels[segment + 1], t),
        ))
    }
    fn tangent(&self, distance: f64, window: f64) -> Option<(f64, bool)> {
        let left = self.sample((distance - window).max(0.0))?.1;
        let right = self.sample((distance + window).min(self.length()))?.1;
        readable_angle((right.y - left.y).atan2(right.x - left.x))
    }
    fn curvature_degrees(&self, distance: f64, span: f64) -> Option<f64> {
        let left = self.tangent((distance - span).max(0.0), 4.0)?.0;
        let right = self.tangent((distance + span).min(self.length()), 4.0)?.0;
        Some(angle_difference(left, right).to_degrees().abs())
    }
}
struct VisibleFragment {
    level: f64,
    path_index: usize,
    path: ProjectedPath,
}
#[derive(Clone)]
struct LabelGlyph {
    logical: TernaryCartesian,
    text: String,
    angle: f64,
    offset: (i32, i32),
}
#[derive(Clone)]
struct PreparedLabel {
    glyphs: Vec<LabelGlyph>,
    bounds: Bounds,
    mask_interval: (f64, f64),
}

fn visible_fragments<DB: DrawingBackend>(
    chart: &TernaryChart<'_, DB>,
    contours: &ContourSet,
) -> Result<Vec<VisibleFragment>, TernaryChartError<DB::ErrorType>> {
    let mut fragments = Vec::new();
    for level in &contours.levels {
        for (path_index, path) in level.paths.iter().enumerate() {
            let mut source = path
                .points
                .iter()
                .copied()
                .map(|p| TernaryPoint::from(p.as_array()))
                .collect::<Vec<_>>();
            if path.closed && !source.is_empty() {
                source.push(source[0]);
            }
            for logical in prepare_polyline(
                chart.geometry,
                chart.viewport,
                source,
                Normalization::RequireUnitSum,
                chart.tolerance,
                InvalidPointPolicy::Error,
            )? {
                if logical.len() >= 2 {
                    fragments.push(VisibleFragment {
                        level: level.value,
                        path_index,
                        path: ProjectedPath::new(chart, logical),
                    });
                }
            }
        }
    }
    Ok(fragments)
}
fn project_complete_path<DB: DrawingBackend>(
    chart: &TernaryChart<'_, DB>,
    path: &ContourPath,
) -> Result<ProjectedPath, TernaryChartError<DB::ErrorType>> {
    let mut logical = path
        .points
        .iter()
        .copied()
        .map(|p| {
            chart.geometry.project(
                TernaryPoint::from(p.as_array()),
                Normalization::RequireUnitSum,
                chart.tolerance,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    if path.closed && !logical.is_empty() {
        logical.push(logical[0]);
    }
    Ok(ProjectedPath::new(chart, logical))
}

fn prepare_labels<DB: DrawingBackend>(
    chart: &TernaryChart<'_, DB>,
    contours: &ContourSet,
    config: &ContourLabelConfig<'_>,
) -> Result<Vec<PreparedLabel>, TernaryChartError<DB::ErrorType>> {
    let viewport = viewport_pixel_bounds(chart);
    let mut reserved: Vec<Bounds> = Vec::new();
    let mut result = Vec::new();
    if let ContourLabelPlacement::Manual(anchors) = &config.placement {
        for anchor in anchors {
            let level = contours
                .levels
                .iter()
                .find(|v| level_matches(v.value, anchor.level))
                .ok_or(SeriesError::ContourDisplay(
                    ContourDisplayError::ManualLevelNotFound {
                        level: anchor.level,
                    },
                ))?;
            let source = level
                .paths
                .get(anchor.path_index)
                .ok_or(SeriesError::ContourDisplay(
                    ContourDisplayError::ManualPathNotFound {
                        level: anchor.level,
                        path_index: anchor.path_index,
                    },
                ))?;
            let path = project_complete_path(chart, source)?;
            let text = (config.formatter)(level.value);
            let dimensions = chart
                .plotting_area()
                .estimate_text_size(&text, &config.style.text.plotters_style())?;
            if let Some(candidate) = make_label(
                &path,
                anchor.arc_fraction * path.length(),
                &text,
                dimensions,
                config,
                viewport,
                false,
                chart,
            )? && !reserved.iter().any(|b| candidate.bounds.overlaps(*b))
            {
                reserved.push(candidate.bounds);
                result.push(candidate);
            }
        }
        return Ok(result);
    }
    let fragments = visible_fragments(chart, contours)?;
    for level in &contours.levels {
        let text = (config.formatter)(level.value);
        let dimensions = chart
            .plotting_area()
            .estimate_text_size(&text, &config.style.text.plotters_style())?;
        for path_index in 0..level.paths.len() {
            let eligible = fragments
                .iter()
                .filter(|f| level_matches(f.level, level.value) && f.path_index == path_index)
                .collect::<Vec<_>>();
            match config.placement {
                ContourLabelPlacement::Automatic => {
                    let mut candidates = Vec::new();
                    for fragment in eligible {
                        for distance in automatic_distances(
                            fragment.path.length(),
                            f64::from(dimensions.0),
                            config.endpoint_clearance,
                        ) {
                            if let Some(candidate) = make_label(
                                &fragment.path,
                                distance,
                                &text,
                                dimensions,
                                config,
                                viewport,
                                true,
                                chart,
                            )? {
                                let curvature = fragment
                                    .path
                                    .curvature_degrees(distance, f64::from(dimensions.0) * 0.3)
                                    .unwrap_or(180.0);
                                let score = -(distance - fragment.path.length() * 0.5).abs()
                                    - 2.0 * curvature;
                                candidates.push((score, candidate));
                            }
                        }
                    }
                    candidates.sort_by(|a, b| b.0.total_cmp(&a.0));
                    if let Some((_, candidate)) = candidates
                        .into_iter()
                        .find(|(_, c)| !reserved.iter().any(|b| c.bounds.overlaps(*b)))
                    {
                        reserved.push(candidate.bounds);
                        result.push(candidate);
                    }
                }
                ContourLabelPlacement::Repeated { spacing } => {
                    for fragment in eligible {
                        for distance in repeated_distances(
                            fragment.path.length(),
                            f64::from(dimensions.0),
                            config.endpoint_clearance,
                            spacing,
                        ) {
                            if let Some(candidate) = make_label(
                                &fragment.path,
                                distance,
                                &text,
                                dimensions,
                                config,
                                viewport,
                                true,
                                chart,
                            )? && !reserved.iter().any(|b| candidate.bounds.overlaps(*b))
                            {
                                reserved.push(candidate.bounds);
                                result.push(candidate);
                            }
                        }
                    }
                }
                ContourLabelPlacement::Manual(_) => unreachable!(),
            }
        }
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn make_label<DB: DrawingBackend>(
    path: &ProjectedPath,
    distance: f64,
    text: &str,
    dimensions: (u32, u32),
    config: &ContourLabelConfig<'_>,
    viewport: Bounds,
    enforce_clearance: bool,
    chart: &TernaryChart<'_, DB>,
) -> Result<Option<PreparedLabel>, TernaryChartError<DB::ErrorType>> {
    let width = f64::from(dimensions.0.max(1));
    let height = f64::from(dimensions.1.max(config.style.text.size()));
    let half = width * 0.5 + 5.0;
    if path.length()
        < config
            .minimum_visible_length
            .max(width + 2.0 * config.endpoint_clearance)
        || (enforce_clearance
            && (distance < config.endpoint_clearance + half
                || path.length() - distance < config.endpoint_clearance + half))
    {
        return Ok(None);
    }
    if path
        .curvature_degrees(distance, half * 0.75)
        .unwrap_or(f64::INFINITY)
        > config.maximum_curvature_degrees
    {
        return Ok(None);
    }
    let Some((angle, reversed)) = path.tangent(distance, (width * 0.15).max(6.0)) else {
        return Ok(None);
    };
    let label = match config.mode {
        ContourLabelMode::Tangent => {
            let Some((logical, pixel)) = path.sample(distance) else {
                return Ok(None);
            };
            let offset = normal_offset(angle, config.style.normal_offset);
            PreparedLabel {
                glyphs: vec![LabelGlyph {
                    logical,
                    text: text.to_owned(),
                    angle,
                    offset,
                }],
                bounds: rotated_bounds(pixel, width, height, angle, offset)
                    .padded(config.collision_padding),
                mask_interval: mask_interval(distance, width, 6.0, path.length()),
            }
        }
        ContourLabelMode::Curved => {
            let style = config.style.text.plotters_style();
            let mut advances = Vec::new();
            for ch in text.chars() {
                let glyph = ch.to_string();
                let (w, h) = chart.plotting_area().estimate_text_size(&glyph, &style)?;
                advances.push((glyph, f64::from(w.max(1)), f64::from(h.max(1))));
            }
            let total = advances.iter().map(|v| v.1).sum::<f64>();
            if total > path.length() - 2.0 * config.endpoint_clearance {
                return Ok(None);
            }
            let mut cursor = -total * 0.5;
            let mut glyphs = Vec::new();
            let mut bounds: Option<Bounds> = None;
            for (glyph, advance, glyph_height) in advances {
                let local = cursor + advance * 0.5;
                cursor += advance;
                let d = if reversed {
                    distance - local
                } else {
                    distance + local
                };
                let Some((logical, pixel)) = path.sample(d) else {
                    return Ok(None);
                };
                let Some((glyph_angle, _)) = path.tangent(d, (advance * 0.5).max(3.0)) else {
                    return Ok(None);
                };
                let offset = normal_offset(glyph_angle, config.style.normal_offset);
                let b = rotated_bounds(
                    pixel,
                    advance,
                    glyph_height.max(height),
                    glyph_angle,
                    offset,
                );
                bounds = Some(bounds.map_or(b, |old| old.union(b)));
                if !glyph.chars().all(char::is_whitespace) {
                    glyphs.push(LabelGlyph {
                        logical,
                        text: glyph,
                        angle: glyph_angle,
                        offset,
                    });
                }
            }
            PreparedLabel {
                glyphs,
                bounds: bounds
                    .unwrap_or(Bounds {
                        x_min: 0.0,
                        x_max: 0.0,
                        y_min: 0.0,
                        y_max: 0.0,
                    })
                    .padded(config.collision_padding),
                mask_interval: mask_interval(distance, total, 6.0, path.length()),
            }
        }
    };
    if !label.bounds.inside(viewport, config.viewport_clearance) {
        return Ok(None);
    }
    Ok(Some(label))
}
impl<'a, DB: DrawingBackend + 'a> TernaryChart<'a, DB> {
    pub fn draw_contour_labels(
        &self,
        contours: &ContourSet,
        config: &ContourLabelConfig<'_>,
    ) -> Result<(), TernaryChartError<DB::ErrorType>> {
        config.validate().map_err(SeriesError::ContourDisplay)?;
        for label in prepare_labels(self, contours, config)? {
            draw_label(self, &label, &config.style)?;
        }
        Ok(())
    }
    pub fn draw_contour_color_bar(
        &self,
        bar: &ContourColorBar<'_>,
    ) -> Result<(), TernaryChartError<DB::ErrorType>> {
        self.draw_contour_color_bar_geometry(bar, 1)?;
        self.draw_contour_color_bar_text(bar)
    }
    pub fn draw_contour_color_bar_geometry(
        &self,
        bar: &ContourColorBar<'_>,
        scale: u32,
    ) -> Result<(), TernaryChartError<DB::ErrorType>> {
        bar.validate().map_err(SeriesError::ContourDisplay)?;
        let layout = color_bar_layout(self, bar, scale);
        let cells = 64_i32;
        for index in 0..cells {
            let start = f64::from(index) / f64::from(cells);
            let end = f64::from(index + 1) / f64::from(cells);
            let style = (bar.map)((start + end) * 0.5).filled();
            let (first, second) = match bar.orientation {
                ContourColorBarOrientation::Vertical => {
                    let y0 = layout.origin.1 + layout.length
                        - (end * f64::from(layout.length)).round() as i32;
                    let y1 = layout.origin.1 + layout.length
                        - (start * f64::from(layout.length)).round() as i32;
                    (
                        (layout.origin.0, y0),
                        (layout.origin.0 + layout.thickness, y1),
                    )
                }
                ContourColorBarOrientation::Horizontal => {
                    let x0 = layout.origin.0 + (start * f64::from(layout.length)).round() as i32;
                    let x1 = layout.origin.0 + (end * f64::from(layout.length)).round() as i32;
                    (
                        (x0, layout.origin.1),
                        (x1, layout.origin.1 + layout.thickness),
                    )
                }
            };
            self.plotting_area().draw(
                &(EmptyElement::<_, DB>::at((layout.anchor.x, layout.anchor.y))
                    + Rectangle::new([first, second], style)),
            )?;
        }
        let outline = match bar.orientation {
            ContourColorBarOrientation::Vertical => [
                layout.origin,
                (
                    layout.origin.0 + layout.thickness,
                    layout.origin.1 + layout.length,
                ),
            ],
            ContourColorBarOrientation::Horizontal => [
                layout.origin,
                (
                    layout.origin.0 + layout.length,
                    layout.origin.1 + layout.thickness,
                ),
            ],
        };
        self.plotting_area().draw(
            &(EmptyElement::<_, DB>::at((layout.anchor.x, layout.anchor.y))
                + Rectangle::new(outline, scaled_shape(bar.border_style, scale))),
        )?;
        for value in &bar.ticks {
            let t = ((*value - bar.minimum) / (bar.maximum - bar.minimum)).clamp(0.0, 1.0);
            let points = match bar.orientation {
                ContourColorBarOrientation::Vertical => {
                    let y = layout.origin.1 + layout.length
                        - (t * f64::from(layout.length)).round() as i32;
                    [
                        (layout.origin.0 + layout.thickness, y),
                        (layout.origin.0 + layout.thickness + 6 * scale as i32, y),
                    ]
                }
                ContourColorBarOrientation::Horizontal => {
                    let x = layout.origin.0 + (t * f64::from(layout.length)).round() as i32;
                    [
                        (x, layout.origin.1 + layout.thickness),
                        (x, layout.origin.1 + layout.thickness + 6 * scale as i32),
                    ]
                }
            };
            self.plotting_area().draw(
                &(EmptyElement::<_, DB>::at((layout.anchor.x, layout.anchor.y))
                    + PathElement::new(points, BLACK.stroke_width(scale))),
            )?;
        }
        Ok(())
    }
    pub fn draw_contour_color_bar_text(
        &self,
        bar: &ContourColorBar<'_>,
    ) -> Result<(), TernaryChartError<DB::ErrorType>> {
        bar.validate().map_err(SeriesError::ContourDisplay)?;
        let layout = color_bar_layout(self, bar, 1);
        let style = bar.text_style.plotters_style();
        for value in &bar.ticks {
            let t = ((*value - bar.minimum) / (bar.maximum - bar.minimum)).clamp(0.0, 1.0);
            let offset = match bar.orientation {
                ContourColorBarOrientation::Vertical => {
                    let y = layout.origin.1 + layout.length
                        - (t * f64::from(layout.length)).round() as i32;
                    (layout.origin.0 + layout.thickness + 10, y - 8)
                }
                ContourColorBarOrientation::Horizontal => {
                    let x = layout.origin.0 + (t * f64::from(layout.length)).round() as i32;
                    (x - 14, layout.origin.1 + layout.thickness + 10)
                }
            };
            self.plotting_area().draw(
                &(EmptyElement::<_, DB>::at((layout.anchor.x, layout.anchor.y))
                    + Text::new((bar.formatter)(*value), offset, style.clone())),
            )?;
        }
        if let Some(title) = &bar.title {
            let offset = (
                layout.origin.0,
                layout.origin.1 - bar.text_style.size() as i32 - 4,
            );
            self.plotting_area().draw(
                &(EmptyElement::<_, DB>::at((layout.anchor.x, layout.anchor.y))
                    + Text::new(title.clone(), offset, style)),
            )?;
        }
        Ok(())
    }
}

fn draw_label<DB: DrawingBackend>(
    chart: &TernaryChart<'_, DB>,
    label: &PreparedLabel,
    style: &ContourLabelStyle,
) -> Result<(), TernaryChartError<DB::ErrorType>> {
    let _ = label.mask_interval;
    if let Some(color) = style.halo_color {
        let halo = AxisTextStyle::new(
            style.text.family(),
            style.text.size(),
            style.text.font_style(),
            color,
        );
        let radius = style.halo_width as i32;
        for glyph in &label.glyphs {
            for y in -radius..=radius {
                for x in -radius..=radius {
                    if (x == 0 && y == 0) || x * x + y * y > radius * radius {
                        continue;
                    }
                    chart.plotting_area().draw(&RotatedText::new(
                        (glyph.logical.x, glyph.logical.y),
                        glyph.text.clone(),
                        halo.clone(),
                        glyph.angle,
                        (glyph.offset.0 + x, glyph.offset.1 + y),
                    ))?;
                }
            }
        }
    }
    for glyph in &label.glyphs {
        chart.plotting_area().draw(&RotatedText::new(
            (glyph.logical.x, glyph.logical.y),
            glyph.text.clone(),
            style.text.clone(),
            glyph.angle,
            glyph.offset,
        ))?;
    }
    Ok(())
}
fn automatic_distances(length: f64, width: f64, endpoint: f64) -> Vec<f64> {
    let start = endpoint + width * 0.5 + 5.0;
    let end = length - start;
    if end < start {
        return Vec::new();
    }
    (0..=16)
        .map(|i| start + (end - start) * f64::from(i) / 16.0)
        .collect()
}
fn repeated_distances(length: f64, width: f64, endpoint: f64, spacing: f64) -> Vec<f64> {
    let clearance = endpoint + width * 0.5 + 5.0;
    let usable = length - 2.0 * clearance;
    if usable < 0.0 {
        return Vec::new();
    }
    let count = (usable / spacing).floor() as usize + 1;
    if count == 1 {
        return vec![length * 0.5];
    }
    let actual = usable / (count - 1) as f64;
    (0..count).map(|i| clearance + i as f64 * actual).collect()
}
fn readable_angle(mut angle: f64) -> Option<(f64, bool)> {
    if !angle.is_finite() {
        return None;
    }
    while angle > PI {
        angle -= 2.0 * PI;
    }
    while angle <= -PI {
        angle += 2.0 * PI;
    }
    let mut reversed = false;
    if angle > FRAC_PI_2 {
        angle -= PI;
        reversed = true;
    } else if angle < -FRAC_PI_2 {
        angle += PI;
        reversed = true;
    }
    Some((angle, reversed))
}
fn angle_difference(left: f64, right: f64) -> f64 {
    let mut d = right - left;
    while d > PI {
        d -= 2.0 * PI;
    }
    while d < -PI {
        d += 2.0 * PI;
    }
    d
}
fn normal_offset(angle: f64, distance: f64) -> (i32, i32) {
    (
        (-angle.sin() * distance).round() as i32,
        (angle.cos() * distance).round() as i32,
    )
}
fn rotated_bounds(
    c: PixelPoint,
    width: f64,
    height: f64,
    angle: f64,
    offset: (i32, i32),
) -> Bounds {
    let hx = 0.5 * (width * angle.cos().abs() + height * angle.sin().abs());
    let hy = 0.5 * (width * angle.sin().abs() + height * angle.cos().abs());
    let x = c.x + f64::from(offset.0);
    let y = c.y + f64::from(offset.1);
    Bounds {
        x_min: x - hx,
        x_max: x + hx,
        y_min: y - hy,
        y_max: y + hy,
    }
}
fn mask_interval(distance: f64, width: f64, gap: f64, length: f64) -> (f64, f64) {
    let half = width * 0.5 + gap;
    ((distance - half).max(0.0), (distance + half).min(length))
}
fn viewport_pixel_bounds<DB: DrawingBackend>(chart: &TernaryChart<'_, DB>) -> Bounds {
    let v = chart.viewport();
    let corners = [
        (v.x_min(), v.y_min()),
        (v.x_min(), v.y_max()),
        (v.x_max(), v.y_min()),
        (v.x_max(), v.y_max()),
    ];
    let pixels = corners.map(|p| chart.plotting_area().map_coordinate(&p));
    Bounds {
        x_min: f64::from(pixels.iter().map(|p| p.0).min().unwrap_or(0)),
        x_max: f64::from(pixels.iter().map(|p| p.0).max().unwrap_or(0)),
        y_min: f64::from(pixels.iter().map(|p| p.1).min().unwrap_or(0)),
        y_max: f64::from(pixels.iter().map(|p| p.1).max().unwrap_or(0)),
    }
}
fn level_matches(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1.0e-12 * a.abs().max(b.abs()).max(1.0)
}
#[derive(Clone, Copy)]
struct ColorBarLayout {
    anchor: TernaryCartesian,
    origin: (i32, i32),
    length: i32,
    thickness: i32,
}
fn color_bar_layout<DB: DrawingBackend>(
    chart: &TernaryChart<'_, DB>,
    bar: &ContourColorBar<'_>,
    scale: u32,
) -> ColorBarLayout {
    let v = chart.viewport();
    let (anchor, right, lower) = match bar.position {
        ContourColorBarPosition::UpperLeft => (
            TernaryCartesian {
                x: v.x_min(),
                y: v.y_max(),
            },
            false,
            false,
        ),
        ContourColorBarPosition::UpperRight => (
            TernaryCartesian {
                x: v.x_max(),
                y: v.y_max(),
            },
            true,
            false,
        ),
        ContourColorBarPosition::LowerLeft => (
            TernaryCartesian {
                x: v.x_min(),
                y: v.y_min(),
            },
            false,
            true,
        ),
        ContourColorBarPosition::LowerRight => (
            TernaryCartesian {
                x: v.x_max(),
                y: v.y_min(),
            },
            true,
            true,
        ),
    };
    let length = bar.length.saturating_mul(scale) as i32;
    let thickness = bar.thickness.saturating_mul(scale) as i32;
    let margin = bar.margin.saturating_mul(scale as i32);
    let (w, h) = match bar.orientation {
        ContourColorBarOrientation::Vertical => (thickness, length),
        ContourColorBarOrientation::Horizontal => (length, thickness),
    };
    ColorBarLayout {
        anchor,
        origin: (
            if right { -margin - w } else { margin },
            if lower { -margin - h } else { margin },
        ),
        length,
        thickness,
    }
}
fn scaled_shape(style: ShapeStyle, scale: u32) -> ShapeStyle {
    style.stroke_width(style.stroke_width.saturating_mul(scale))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn straight_path(reverse: bool) -> ProjectedPath {
        let mut logical = vec![
            TernaryCartesian { x: 0.0, y: 0.0 },
            TernaryCartesian { x: 0.5, y: 0.0 },
            TernaryCartesian { x: 1.0, y: 0.0 },
        ];
        let mut pixels = vec![
            PixelPoint { x: 0.0, y: 50.0 },
            PixelPoint { x: 100.0, y: 50.0 },
            PixelPoint { x: 200.0, y: 50.0 },
        ];
        if reverse {
            logical.reverse();
            pixels.reverse();
        }
        ProjectedPath {
            logical,
            pixels,
            cumulative: vec![0.0, 100.0, 200.0],
        }
    }
    #[test]
    fn projected_arc_length_sampling_and_tangent_are_stable() {
        let path = straight_path(false);
        assert_eq!(path.length(), 200.0);
        let (logical, pixel) = path.sample(75.0).unwrap();
        assert!((logical.x - 0.375).abs() < 1.0e-12);
        assert_eq!(pixel, PixelPoint { x: 75.0, y: 50.0 });
        assert_eq!(path.tangent(75.0, 5.0), Some((0.0, false)));
    }
    #[test]
    fn upside_down_tangents_are_corrected() {
        let (angle, reversed) = readable_angle(PI).unwrap();
        assert!(angle.abs() < 1.0e-12);
        assert!(reversed);
        assert!(readable_angle(FRAC_PI_2 * 0.9).unwrap().0.abs() <= FRAC_PI_2);
    }
    #[test]
    fn curvature_endpoint_and_viewport_helpers_reject_bad_candidates() {
        let path = ProjectedPath {
            logical: vec![
                TernaryCartesian { x: 0.0, y: 0.0 },
                TernaryCartesian { x: 0.5, y: 0.5 },
                TernaryCartesian { x: 1.0, y: 0.0 },
            ],
            pixels: vec![
                PixelPoint { x: 0.0, y: 100.0 },
                PixelPoint { x: 100.0, y: 0.0 },
                PixelPoint { x: 200.0, y: 100.0 },
            ],
            cumulative: vec![0.0, 141.421356, 282.842712],
        };
        assert!(path.curvature_degrees(path.length() * 0.5, 50.0).unwrap() > 60.0);
        assert!(automatic_distances(60.0, 50.0, 20.0).is_empty());
        let inner = Bounds {
            x_min: 1.0,
            x_max: 9.0,
            y_min: 1.0,
            y_max: 9.0,
        };
        let outer = Bounds {
            x_min: 0.0,
            x_max: 10.0,
            y_min: 0.0,
            y_max: 10.0,
        };
        assert!(!inner.inside(outer, 2.0));
    }
    #[test]
    fn repeated_spacing_collision_and_masks_are_deterministic() {
        assert_eq!(
            repeated_distances(500.0, 80.0, 20.0, 150.0),
            vec![65.0, 250.0, 435.0]
        );
        assert_eq!(mask_interval(100.0, 60.0, 5.0, 200.0), (65.0, 135.0));
        let a = Bounds {
            x_min: 0.0,
            x_max: 10.0,
            y_min: 0.0,
            y_max: 10.0,
        };
        let b = Bounds {
            x_min: 9.0,
            x_max: 20.0,
            y_min: 0.0,
            y_max: 10.0,
        };
        assert!(a.overlaps(b));
    }
    #[test]
    fn manual_anchors_and_color_bars_validate() {
        let invalid = ContourLabelConfig::new().placement(ContourLabelPlacement::Manual(vec![
            ContourLabelAnchor::new(1.0, 0, 1.5),
        ]));
        assert!(invalid.validate().is_err());
        assert!(ContourColorBar::new(1.0, 1.0, |_| BLACK.to_rgba()).is_err());
        assert!(
            ContourColorBar::new(0.0, 1.0, |_| BLACK.to_rgba())
                .unwrap()
                .validate()
                .is_ok()
        );
    }
}
