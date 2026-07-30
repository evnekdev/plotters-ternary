use std::sync::Arc;

use plotters::backend::DrawingBackend;
use plotters::element::{EmptyElement, PathElement, Text};
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters::style::{
    BLACK, Color, FontStyle, IntoFont, IntoTextStyle, RGBAColor, RGBColor, ShapeStyle, TextStyle,
};

use super::rotated_text::RotatedText;
use super::{TernaryChart, TernaryChartError};
use crate::Tolerance;
use crate::coord::{
    CartesianSegment, Component, TernaryCartesian, TernaryGeometry, clip_segment_with_parameters,
};

const DEFAULT_MAJOR_STEP: f64 = 0.1;
const MAX_TICK_INTERVALS: usize = 10_000;

/// Semantic component axis. A/B/C remain semantic when vertices are reordered.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TernaryAxis {
    A,
    B,
    C,
}
impl TernaryAxis {
    pub const ALL: [Self; 3] = [Self::A, Self::B, Self::C];
    pub const fn component(self) -> Component {
        match self {
            Self::A => Component::A,
            Self::B => Component::B,
            Self::C => Component::C,
        }
    }
    const fn index(self) -> usize {
        self.component().index()
    }
    fn from_component(component: Component) -> Self {
        match component {
            Component::A => Self::A,
            Component::B => Self::B,
            Component::C => Self::C,
        }
    }
}

/// Deterministic positions for an axis. `Count(n)` means `n` intervals, so it
/// has `n + 1` candidates before endpoint-label policy is applied.
#[derive(Clone, Debug, PartialEq)]
pub enum TickSpec {
    Count(usize),
    Step(f64),
    Values(Vec<f64>),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TickRangeMode {
    #[default]
    FullCompositionRange,
    VisibleRange,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CroppedAxisPolicy {
    #[default]
    TriangleEdgesOnly,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TickDirection {
    Inward,
    #[default]
    Outward,
    Both,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum EndpointLabelPolicy {
    Both,
    MinimumOnly,
    MaximumOnly,
    InteriorOnly,
    None,
    #[default]
    AutoAvoidDuplicates,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CornerLabelVisibility {
    #[default]
    Auto,
    Always,
    VisibleCornersOnly,
    Hidden,
}

/// Generated numeric-label formatting. Custom formatters may return Unicode.
#[derive(Clone)]
pub enum AxisLabelFormat {
    Decimal { precision: usize },
    Percentage { precision: usize },
    Custom(Arc<dyn Fn(f64) -> String + Send + Sync + 'static>),
}
impl Default for AxisLabelFormat {
    fn default() -> Self {
        Self::Decimal { precision: 1 }
    }
}
impl AxisLabelFormat {
    pub fn format(&self, value: f64) -> String {
        match self {
            Self::Decimal { precision } => format!("{value:.precision$}"),
            Self::Percentage { precision } => format!("{:.precision$}%", value * 100.0),
            Self::Custom(formatter) => formatter(value),
        }
    }
}

/// Owned Plotters-compatible text attributes for independently configured axes.
#[derive(Clone)]
pub struct AxisTextStyle {
    family: String,
    size: u32,
    font_style: FontStyle,
    color: RGBAColor,
}
impl AxisTextStyle {
    pub fn new<S: Into<String>>(
        family: S,
        size: u32,
        font_style: FontStyle,
        color: RGBAColor,
    ) -> Self {
        Self {
            family: family.into(),
            size,
            font_style,
            color,
        }
    }
    pub fn sans_serif(size: u32, font_style: FontStyle, color: RGBAColor) -> Self {
        Self::new("sans-serif", size, font_style, color)
    }
    pub const fn size(&self) -> u32 {
        self.size
    }
    pub fn family(&self) -> &str {
        &self.family
    }
    pub const fn font_style(&self) -> FontStyle {
        self.font_style
    }
    pub const fn color(&self) -> RGBAColor {
        self.color
    }
    fn from_plotters(style: TextStyle<'_>) -> Self {
        let color = style.color;
        Self::new(
            style.font.get_name().to_owned(),
            style.font.get_size().round().max(0.0) as u32,
            style.font.get_style(),
            RGBAColor(color.rgb.0, color.rgb.1, color.rgb.2, color.alpha),
        )
    }
    pub(crate) fn plotters_style(&self) -> TextStyle<'_> {
        (self.family.as_str(), self.size, self.font_style)
            .into_font()
            .color(&self.color)
    }
}

/// Final-output pixel dimensions and style for one tick class.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TickStyle {
    style: ShapeStyle,
    length: u32,
    direction: TickDirection,
}
impl TickStyle {
    pub const fn new(style: ShapeStyle, length: u32, direction: TickDirection) -> Self {
        Self {
            style,
            length,
            direction,
        }
    }
    pub const fn style(self) -> ShapeStyle {
        self.style
    }
    pub const fn length(self) -> u32 {
        self.length
    }
    pub const fn direction(self) -> TickDirection {
        self.direction
    }
}

/// Axis-name placement override for cropped charts.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AxisNamePosition {
    #[default]
    Auto,
    Logical(TernaryCartesian),
    Hidden,
}

/// Configuration for one semantic axis. It is supplied through
/// `TernaryMeshConfig::axis`, `axis_a`, `axis_b`, or `axis_c`.
#[derive(Clone)]
pub struct TernaryAxisConfig {
    visible: bool,
    major_ticks: TickSpec,
    minor_ticks: Option<TickSpec>,
    major_grid: bool,
    minor_grid: bool,
    ticks: bool,
    tick_labels: bool,
    range_mode: TickRangeMode,
    cropped_policy: CroppedAxisPolicy,
    endpoint_labels: EndpointLabelPolicy,
    major_tick: TickStyle,
    minor_tick: TickStyle,
    major_grid_style: ShapeStyle,
    minor_grid_style: ShapeStyle,
    tick_label_style: AxisTextStyle,
    tick_label_offset: u32,
    label_format: AxisLabelFormat,
    name: Option<String>,
    name_style: AxisTextStyle,
    name_offset: u32,
    name_position: AxisNamePosition,
}
impl Default for TernaryAxisConfig {
    fn default() -> Self {
        Self {
            visible: true,
            major_ticks: TickSpec::Step(DEFAULT_MAJOR_STEP),
            minor_ticks: None,
            major_grid: true,
            minor_grid: false,
            // Existing mesh calls retain their pre-M5 no-tick/no-number look.
            ticks: false,
            tick_labels: false,
            range_mode: TickRangeMode::FullCompositionRange,
            cropped_policy: CroppedAxisPolicy::TriangleEdgesOnly,
            endpoint_labels: EndpointLabelPolicy::AutoAvoidDuplicates,
            major_tick: TickStyle::new(BLACK.stroke_width(1), 8, TickDirection::Outward),
            minor_tick: TickStyle::new(BLACK.stroke_width(1), 4, TickDirection::Outward),
            major_grid_style: RGBColor(185, 190, 198).stroke_width(1),
            minor_grid_style: RGBColor(225, 229, 235).stroke_width(1),
            tick_label_style: AxisTextStyle::sans_serif(18, FontStyle::Normal, BLACK.to_rgba()),
            tick_label_offset: 8,
            label_format: AxisLabelFormat::default(),
            name: None,
            name_style: AxisTextStyle::sans_serif(26, FontStyle::Bold, BLACK.to_rgba()),
            name_offset: 24,
            name_position: AxisNamePosition::Auto,
        }
    }
}
impl TernaryAxisConfig {
    pub fn visible(&mut self, value: bool) -> &mut Self {
        self.visible = value;
        self
    }
    pub fn major_ticks(&mut self, value: TickSpec) -> &mut Self {
        self.major_ticks = value;
        self
    }
    pub fn minor_ticks(&mut self, value: TickSpec) -> &mut Self {
        self.minor_ticks = Some(value);
        self
    }
    pub fn hide_minor_ticks(&mut self) -> &mut Self {
        self.minor_ticks = None;
        self
    }
    pub fn draw_major_grid(&mut self, value: bool) -> &mut Self {
        self.major_grid = value;
        self
    }
    pub fn draw_minor_grid(&mut self, value: bool) -> &mut Self {
        self.minor_grid = value;
        self
    }
    pub fn draw_ticks(&mut self, value: bool) -> &mut Self {
        self.ticks = value;
        self
    }
    pub fn draw_tick_labels(&mut self, value: bool) -> &mut Self {
        self.tick_labels = value;
        self
    }
    pub fn tick_range_mode(&mut self, value: TickRangeMode) -> &mut Self {
        self.range_mode = value;
        self
    }
    pub fn cropped_axis_policy(&mut self, value: CroppedAxisPolicy) -> &mut Self {
        self.cropped_policy = value;
        self
    }
    pub fn endpoint_label_policy(&mut self, value: EndpointLabelPolicy) -> &mut Self {
        self.endpoint_labels = value;
        self
    }
    pub fn major_tick_style<S: Into<ShapeStyle>>(&mut self, value: S) -> &mut Self {
        self.major_tick.style = value.into();
        self
    }
    pub fn minor_tick_style<S: Into<ShapeStyle>>(&mut self, value: S) -> &mut Self {
        self.minor_tick.style = value.into();
        self
    }
    pub fn major_tick_length(&mut self, value: u32) -> &mut Self {
        self.major_tick.length = value;
        self
    }
    pub fn minor_tick_length(&mut self, value: u32) -> &mut Self {
        self.minor_tick.length = value;
        self
    }
    pub fn major_tick_direction(&mut self, value: TickDirection) -> &mut Self {
        self.major_tick.direction = value;
        self
    }
    pub fn minor_tick_direction(&mut self, value: TickDirection) -> &mut Self {
        self.minor_tick.direction = value;
        self
    }
    pub fn major_grid_style<S: Into<ShapeStyle>>(&mut self, value: S) -> &mut Self {
        self.major_grid_style = value.into();
        self
    }
    pub fn minor_grid_style<S: Into<ShapeStyle>>(&mut self, value: S) -> &mut Self {
        self.minor_grid_style = value.into();
        self
    }
    pub fn tick_label_style(&mut self, value: AxisTextStyle) -> &mut Self {
        self.tick_label_style = value;
        self
    }
    pub fn tick_label_offset(&mut self, value: u32) -> &mut Self {
        self.tick_label_offset = value;
        self
    }
    pub fn label_format(&mut self, value: AxisLabelFormat) -> &mut Self {
        self.label_format = value;
        self
    }
    pub fn label_formatter<F>(&mut self, value: F) -> &mut Self
    where
        F: Fn(f64) -> String + Send + Sync + 'static,
    {
        self.label_format = AxisLabelFormat::Custom(Arc::new(value));
        self
    }
    pub fn axis_name<S: Into<String>>(&mut self, value: S) -> &mut Self {
        self.name = Some(value.into());
        self
    }
    pub fn hide_axis_name(&mut self) -> &mut Self {
        self.name = None;
        self
    }
    pub fn axis_name_style(&mut self, value: AxisTextStyle) -> &mut Self {
        self.name_style = value;
        self
    }
    pub fn axis_name_offset(&mut self, value: u32) -> &mut Self {
        self.name_offset = value;
        self
    }
    pub fn axis_name_position(&mut self, value: AxisNamePosition) -> &mut Self {
        self.name_position = value;
        self
    }
}

/// Plotters-like configuration for ternary mesh, ticks, and labels.
pub struct TernaryMeshConfig<'chart, 'series, DB: DrawingBackend> {
    chart: &'chart mut TernaryChart<'series, DB>,
    axes: [TernaryAxisConfig; 3],
    boundary_style: ShapeStyle,
    corner_names: [Option<String>; 3],
    corner_label_style: AxisTextStyle,
    corner_label_offset: u32,
    corner_visibility: CornerLabelVisibility,
    draw_corner_names: bool,
    draw_grid: bool,
    draw_boundary: bool,
}
impl<'chart, 'series, DB: DrawingBackend> TernaryMeshConfig<'chart, 'series, DB> {
    pub(crate) fn new(chart: &'chart mut TernaryChart<'series, DB>) -> Self {
        Self {
            chart,
            axes: std::array::from_fn(|_| TernaryAxisConfig::default()),
            boundary_style: BLACK.stroke_width(2),
            corner_names: [None, None, None],
            corner_label_style: AxisTextStyle::sans_serif(28, FontStyle::Bold, BLACK.to_rgba()),
            corner_label_offset: 16,
            corner_visibility: CornerLabelVisibility::Auto,
            draw_corner_names: true,
            draw_grid: true,
            draw_boundary: true,
        }
    }
    /// Configure an independent semantic axis without exposing geometric edge ids.
    pub fn axis<F>(mut self, axis: TernaryAxis, configure: F) -> Self
    where
        F: FnOnce(&mut TernaryAxisConfig),
    {
        configure(&mut self.axes[axis.index()]);
        self
    }
    pub fn axis_a<F>(self, configure: F) -> Self
    where
        F: FnOnce(&mut TernaryAxisConfig),
    {
        self.axis(TernaryAxis::A, configure)
    }
    pub fn axis_b<F>(self, configure: F) -> Self
    where
        F: FnOnce(&mut TernaryAxisConfig),
    {
        self.axis(TernaryAxis::B, configure)
    }
    pub fn axis_c<F>(self, configure: F) -> Self
    where
        F: FnOnce(&mut TernaryAxisConfig),
    {
        self.axis(TernaryAxis::C, configure)
    }

    /// Compatibility shortcut: use one major-grid step for all three axes.
    pub fn major_step(mut self, step: f64) -> Self {
        for axis in &mut self.axes {
            axis.major_ticks = TickSpec::Step(step);
        }
        self
    }
    pub fn boundary_style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.boundary_style = style.into();
        self
    }
    pub fn major_grid_style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        let style = style.into();
        for axis in &mut self.axes {
            axis.major_grid_style = style;
        }
        self
    }
    pub fn minor_grid_style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        let style = style.into();
        for axis in &mut self.axes {
            axis.minor_grid_style = style;
        }
        self
    }
    /// Backwards-compatible common text style; explicit per-axis setters take precedence when called later.
    pub fn text_style<'style, S>(mut self, style: S) -> Self
    where
        S: IntoTextStyle<'style>,
    {
        let style = AxisTextStyle::from_plotters(style.into_text_style(self.chart.plotting_area()));
        for axis in &mut self.axes {
            axis.name_style = style.clone();
            axis.tick_label_style = style.clone();
        }
        self.corner_label_style = style;
        self
    }
    pub fn axis_name_style<'style, S>(mut self, style: S) -> Self
    where
        S: IntoTextStyle<'style>,
    {
        let style = AxisTextStyle::from_plotters(style.into_text_style(self.chart.plotting_area()));
        for axis in &mut self.axes {
            axis.name_style = style.clone();
        }
        self
    }
    pub fn corner_label_style<'style, S>(mut self, style: S) -> Self
    where
        S: IntoTextStyle<'style>,
    {
        self.corner_label_style =
            AxisTextStyle::from_plotters(style.into_text_style(self.chart.plotting_area()));
        self
    }
    pub fn axis_a_name<S: Into<String>>(mut self, value: S) -> Self {
        self.axes[0].name = Some(value.into());
        self
    }
    pub fn axis_b_name<S: Into<String>>(mut self, value: S) -> Self {
        self.axes[1].name = Some(value.into());
        self
    }
    pub fn axis_c_name<S: Into<String>>(mut self, value: S) -> Self {
        self.axes[2].name = Some(value.into());
        self
    }
    pub fn corner_a_name<S: Into<String>>(mut self, value: S) -> Self {
        self.corner_names[0] = Some(value.into());
        self
    }
    pub fn corner_b_name<S: Into<String>>(mut self, value: S) -> Self {
        self.corner_names[1] = Some(value.into());
        self
    }
    pub fn corner_c_name<S: Into<String>>(mut self, value: S) -> Self {
        self.corner_names[2] = Some(value.into());
        self
    }
    pub fn axis_label_offset(mut self, value: u32) -> Self {
        for axis in &mut self.axes {
            axis.name_offset = value;
        }
        self
    }
    pub fn corner_label_offset(mut self, value: u32) -> Self {
        self.corner_label_offset = value;
        self
    }
    pub fn corner_label_visibility(mut self, value: CornerLabelVisibility) -> Self {
        self.corner_visibility = value;
        self
    }
    pub fn hide_axis_names(mut self) -> Self {
        for axis in &mut self.axes {
            axis.name = None;
        }
        self
    }
    pub fn hide_corner_names(mut self) -> Self {
        self.draw_corner_names = false;
        self
    }
    pub fn hide_grid_lines(mut self) -> Self {
        self.draw_grid = false;
        self
    }
    pub fn hide_triangle_boundary(mut self) -> Self {
        self.draw_boundary = false;
        self
    }
    pub fn hide_ticks(mut self) -> Self {
        for axis in &mut self.axes {
            axis.ticks = false;
        }
        self
    }
    pub fn hide_tick_labels(mut self) -> Self {
        for axis in &mut self.axes {
            axis.tick_labels = false;
        }
        self
    }

    /// Draw every mesh phase at native resolution.
    pub fn draw(self) -> Result<(), TernaryChartError<DB::ErrorType>> {
        self.draw_phase(MeshPhase::All, 1)
    }
    /// Draw only vector/geometry primitives at native resolution.
    pub fn draw_geometry(self) -> Result<(), TernaryChartError<DB::ErrorType>> {
        self.draw_phase(MeshPhase::Geometry, 1)
    }
    /// Draw only geometry with final-layout pixel values scaled for PNG supersampling.
    pub fn draw_geometry_scaled(self, scale: u32) -> Result<(), TernaryChartError<DB::ErrorType>> {
        self.draw_phase(MeshPhase::Geometry, scale.max(1))
    }
    /// Draw only final-resolution text. This emits no line, grid, or boundary geometry.
    pub fn draw_text(self) -> Result<(), TernaryChartError<DB::ErrorType>> {
        self.draw_phase(MeshPhase::Text, 1)
    }

    fn draw_phase(
        self,
        phase: MeshPhase,
        scale: u32,
    ) -> Result<(), TernaryChartError<DB::ErrorType>> {
        let prepared = prepare_axes(self.chart, &self.axes)?;
        if phase.geometry() {
            self.draw_geometry_phase(&prepared, scale)?;
        }
        if phase.text() {
            self.draw_text_phase(&prepared)?;
        }
        Ok(())
    }
    fn draw_geometry_phase(
        &self,
        prepared: &[PreparedAxis; 3],
        scale: u32,
    ) -> Result<(), TernaryChartError<DB::ErrorType>> {
        // Stable drawing order: minor grid -> major grid -> boundary -> ticks.
        if self.draw_grid {
            for axis in prepared {
                let config = &self.axes[axis.axis.index()];
                if config.visible && config.minor_grid {
                    self.draw_grid(axis.axis, &axis.minor_grid, config.minor_grid_style, scale)?;
                }
            }
            for axis in prepared {
                let config = &self.axes[axis.axis.index()];
                if config.visible && config.major_grid {
                    self.draw_grid(axis.axis, &axis.major_grid, config.major_grid_style, scale)?;
                }
            }
        }
        if self.draw_boundary {
            for visible in self
                .chart
                .geometry
                .visible_edges(self.chart.viewport, self.chart.tolerance)?
            {
                let segment = visible.segment;
                self.chart.plotting_area().draw(&PathElement::new(
                    [
                        (segment.start.x, segment.start.y),
                        (segment.end.x, segment.end.y),
                    ],
                    scaled_style(self.boundary_style, scale),
                ))?;
            }
        }
        for axis in prepared {
            let config = &self.axes[axis.axis.index()];
            if config.visible && config.ticks {
                self.draw_ticks(axis, &axis.minor, config.minor_tick, scale)?;
            }
        }
        for axis in prepared {
            let config = &self.axes[axis.axis.index()];
            if config.visible && config.ticks {
                self.draw_ticks(axis, &axis.major, config.major_tick, scale)?;
            }
        }
        Ok(())
    }
    fn draw_grid(
        &self,
        axis: TernaryAxis,
        ticks: &[PreparedTick],
        style: ShapeStyle,
        scale: u32,
    ) -> Result<(), TernaryChartError<DB::ErrorType>> {
        for tick in ticks {
            if is_endpoint(tick.value, self.chart.tolerance) {
                continue;
            }
            if let Some(segment) = self.chart.geometry.visible_component_isoline(
                axis.component(),
                tick.value,
                self.chart.viewport,
                self.chart.tolerance,
            )? {
                self.chart.plotting_area().draw(&PathElement::new(
                    [
                        (segment.start.x, segment.start.y),
                        (segment.end.x, segment.end.y),
                    ],
                    scaled_style(style, scale),
                ))?;
            }
        }
        Ok(())
    }
    fn draw_ticks(
        &self,
        axis: &PreparedAxis,
        ticks: &[PreparedTick],
        style: TickStyle,
        scale: u32,
    ) -> Result<(), TernaryChartError<DB::ErrorType>> {
        let Some(edge) = axis.edge else {
            return Ok(());
        };
        for tick in ticks {
            let normal = outward_normal(self.chart, tick.anchor);
            let (start, end) =
                tick_endpoints(normal, style.length.saturating_mul(scale), style.direction);
            let element = EmptyElement::<_, DB>::at((tick.anchor.x, tick.anchor.y))
                + PathElement::new([start, end], scaled_style(style.style, scale));
            self.chart.plotting_area().draw(&element)?;
        }
        let _ = edge; // The edge proves physical ticks are never synthesized on a viewport side.
        Ok(())
    }

    fn draw_text_phase(
        &self,
        prepared: &[PreparedAxis; 3],
    ) -> Result<(), TernaryChartError<DB::ErrorType>> {
        // Tick labels -> axis names -> corners. PNG geometry passes never reach here.
        for axis in prepared {
            let config = &self.axes[axis.axis.index()];
            if config.visible && config.tick_labels {
                self.draw_tick_labels(axis, config)?;
            }
        }
        for axis in prepared {
            let config = &self.axes[axis.axis.index()];
            if config.visible {
                self.draw_axis_name(axis, config)?;
            }
        }
        if self.draw_corner_names && self.corner_visibility != CornerLabelVisibility::Hidden {
            draw_corner_names(
                self.chart,
                &self.corner_names,
                &self.corner_label_style,
                self.corner_label_offset,
                self.corner_visibility,
            )?;
        }
        Ok(())
    }
    fn draw_tick_labels(
        &self,
        axis: &PreparedAxis,
        config: &TernaryAxisConfig,
    ) -> Result<(), TernaryChartError<DB::ErrorType>> {
        if axis.edge.is_none() {
            return Ok(());
        }
        for tick in &axis.major {
            if !show_tick_label(
                axis.axis,
                tick.value,
                config.endpoint_labels,
                self.chart,
                &self.corner_names,
                self.corner_visibility,
            ) {
                continue;
            }
            let normal = outward_normal(self.chart, tick.anchor);
            let offset = pixel_offset(normal, config.tick_label_offset);
            let position = Pos::new(
                if normal.0 < 0.0 {
                    HPos::Right
                } else if normal.0 > 0.0 {
                    HPos::Left
                } else {
                    HPos::Center
                },
                if normal.1 < 0.0 {
                    VPos::Bottom
                } else {
                    VPos::Top
                },
            );
            let text = EmptyElement::<_, DB>::at((tick.anchor.x, tick.anchor.y))
                + Text::new(
                    config.label_format.format(tick.value),
                    offset,
                    config.tick_label_style.plotters_style().pos(position),
                );
            self.chart.plotting_area().draw(&text)?;
        }
        Ok(())
    }
    fn draw_axis_name(
        &self,
        axis: &PreparedAxis,
        config: &TernaryAxisConfig,
    ) -> Result<(), TernaryChartError<DB::ErrorType>> {
        let Some(name) = &config.name else {
            return Ok(());
        };
        let anchor = match config.name_position {
            AxisNamePosition::Auto => {
                if axis.edge.is_none() {
                    return Ok(());
                }
                axis_name_anchor(self.chart.geometry, axis.axis.component())
            }
            AxisNamePosition::Logical(anchor) => anchor,
            AxisNamePosition::Hidden => return Ok(()),
        };
        if !self.chart.viewport.contains(anchor, self.chart.tolerance)? {
            return Ok(());
        }
        let edge = semantic_axis_edge(self.chart.geometry, axis.axis.component());
        let layout = axis_name_layout(self.chart, edge, anchor, config.name_offset);
        if layout.angle.abs() <= 1.0e-12 {
            let text = EmptyElement::<_, DB>::at((anchor.x, anchor.y))
                + Text::new(
                    name.clone(),
                    layout.offset,
                    config
                        .name_style
                        .plotters_style()
                        .pos(Pos::new(HPos::Center, VPos::Center)),
                );
            self.chart.plotting_area().draw(&text)?;
        } else {
            self.chart.plotting_area().draw(&RotatedText::new(
                (anchor.x, anchor.y),
                name.clone(),
                config.name_style.clone(),
                layout.angle,
                layout.offset,
            ))?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum MeshPhase {
    All,
    Geometry,
    Text,
}
impl MeshPhase {
    const fn geometry(self) -> bool {
        matches!(self, Self::All | Self::Geometry)
    }
    const fn text(self) -> bool {
        matches!(self, Self::All | Self::Text)
    }
}
#[derive(Clone, Copy)]
struct PreparedEdge {
    start: f64,
    end: f64,
}
#[derive(Clone, Copy)]
struct PreparedTick {
    value: f64,
    anchor: TernaryCartesian,
}
struct PreparedAxis {
    axis: TernaryAxis,
    edge: Option<PreparedEdge>,
    // Grid values remain available in full-range interior viewports even when
    // no original edge is visible. Physical ticks use only edge-visible values.
    major_grid: Vec<PreparedTick>,
    minor_grid: Vec<PreparedTick>,
    major: Vec<PreparedTick>,
    minor: Vec<PreparedTick>,
}

fn prepare_axes<DB: DrawingBackend>(
    chart: &TernaryChart<'_, DB>,
    configs: &[TernaryAxisConfig; 3],
) -> Result<[PreparedAxis; 3], TernaryChartError<DB::ErrorType>> {
    let mut result = Vec::with_capacity(3);
    for axis in TernaryAxis::ALL {
        let config = &configs[axis.index()];
        let source = semantic_axis_edge(chart.geometry, axis.component());
        let edge = clip_segment_with_parameters(source, chart.viewport, chart.tolerance)?.map(
            |fragment| PreparedEdge {
                start: fragment.parameter_start,
                end: fragment.parameter_end,
            },
        );
        let range = match (config.range_mode, edge) {
            (TickRangeMode::FullCompositionRange, _) => Some((0.0, 1.0)),
            (TickRangeMode::VisibleRange, Some(edge)) => Some((edge.start, edge.end)),
            (TickRangeMode::VisibleRange, None) => None,
        };
        let major_values = match range {
            Some(range) => resolve_tick_values(&config.major_ticks, range, chart.tolerance)?,
            None => Vec::new(),
        };
        let minor_values = match (&config.minor_ticks, range) {
            (Some(spec), Some(range)) => resolve_tick_values(spec, range, chart.tolerance)?,
            _ => Vec::new(),
        };
        let minor_values: Vec<_> = minor_values
            .into_iter()
            .filter(|minor| {
                !major_values
                    .iter()
                    .any(|major| chart.tolerance.is_close(*major, *minor))
            })
            .collect();
        let major_grid = to_ticks(source, &major_values);
        let minor_grid = to_ticks(source, &minor_values);
        let edge_visible = |value: f64| match edge {
            Some(edge) => {
                value >= edge.start - chart.tolerance.absolute
                    && value <= edge.end + chart.tolerance.absolute
            }
            None => false,
        };
        let major = to_ticks(
            source,
            &major_values
                .iter()
                .copied()
                .filter(|value| edge_visible(*value))
                .collect::<Vec<_>>(),
        );
        let minor = to_ticks(
            source,
            &minor_values
                .iter()
                .copied()
                .filter(|value| edge_visible(*value))
                .collect::<Vec<_>>(),
        );
        result.push(PreparedAxis {
            axis,
            edge,
            major_grid,
            minor_grid,
            major,
            minor,
        });
    }
    match result.try_into() {
        Ok(axes) => Ok(axes),
        Err(_) => unreachable!("three semantic axes"),
    }
}
fn semantic_axis_edge(geometry: TernaryGeometry, component: Component) -> CartesianSegment {
    let [first, second] = component.others();
    CartesianSegment::new(geometry.vertex(first), geometry.vertex(second))
}
fn to_ticks(source: CartesianSegment, values: &[f64]) -> Vec<PreparedTick> {
    values
        .iter()
        .map(|&value| PreparedTick {
            value,
            anchor: source.point_at(value),
        })
        .collect()
}

/// Backend-neutral tick resolution. Explicit values are sorted and deduplicated.
pub(crate) fn resolve_tick_values<E: std::error::Error + Send + Sync>(
    spec: &TickSpec,
    range: (f64, f64),
    tolerance: Tolerance,
) -> Result<Vec<f64>, TernaryChartError<E>> {
    let (low, high) = range;
    if !low.is_finite() || !high.is_finite() || low > high + tolerance.absolute {
        return Err(TernaryChartError::InvalidTickValue { value: low });
    }
    let mut values: Vec<f64> = match spec {
        TickSpec::Count(0) => return Err(TernaryChartError::InvalidTickCount { count: 0 }),
        TickSpec::Count(count) => (0..=*count)
            .map(|i| low + (high - low) * (i as f64 / *count as f64))
            .collect(),
        TickSpec::Step(step) => {
            if !step.is_finite()
                || *step <= 0.0
                || *step > 1.0
                || (1.0 / *step).ceil() as usize > MAX_TICK_INTERVALS
            {
                return Err(TernaryChartError::InvalidTickStep { value: *step });
            }
            let count = (1.0 / *step).floor() as usize;
            (0..=count)
                .map(|i| i as f64 * *step)
                .filter(|v| *v >= low - tolerance.absolute && *v <= high + tolerance.absolute)
                .collect()
        }
        TickSpec::Values(input) => {
            let mut values = Vec::new();
            for &value in input {
                if !value.is_finite()
                    || value < -tolerance.absolute
                    || value > 1.0 + tolerance.absolute
                {
                    return Err(TernaryChartError::InvalidTickValue { value });
                }
                if value >= low - tolerance.absolute && value <= high + tolerance.absolute {
                    values.push(snap_endpoint(value, tolerance));
                }
            }
            values
        }
    };
    for value in &mut values {
        *value = snap_endpoint(*value, tolerance);
    }
    values.sort_by(f64::total_cmp);
    values.dedup_by(|a, b| tolerance.is_close(*a, *b));
    Ok(values)
}
#[allow(dead_code)]
pub(crate) fn major_grid_values<E: std::error::Error + Send + Sync>(
    step: f64,
) -> Result<Vec<f64>, TernaryChartError<E>> {
    resolve_tick_values(&TickSpec::Step(step), (0.0, 1.0), Tolerance::default()).map(|values| {
        values
            .into_iter()
            .filter(|v| !is_endpoint(*v, Tolerance::default()))
            .collect()
    })
}
fn snap_endpoint(value: f64, tolerance: Tolerance) -> f64 {
    if tolerance.is_close(value, 0.0) {
        0.0
    } else if tolerance.is_close(value, 1.0) {
        1.0
    } else {
        value
    }
}
fn is_endpoint(value: f64, tolerance: Tolerance) -> bool {
    tolerance.is_close(value, 0.0) || tolerance.is_close(value, 1.0)
}
fn scaled_style(style: ShapeStyle, scale: u32) -> ShapeStyle {
    style.stroke_width(style.stroke_width.saturating_mul(scale))
}

fn show_tick_label<DB: DrawingBackend>(
    axis: TernaryAxis,
    value: f64,
    policy: EndpointLabelPolicy,
    chart: &TernaryChart<'_, DB>,
    corner_names: &[Option<String>; 3],
    corner_visibility: CornerLabelVisibility,
) -> bool {
    if !is_endpoint(value, chart.tolerance) {
        return policy != EndpointLabelPolicy::None;
    }
    let requested = match policy {
        EndpointLabelPolicy::Both | EndpointLabelPolicy::AutoAvoidDuplicates => true,
        EndpointLabelPolicy::MinimumOnly => chart.tolerance.is_close(value, 0.0),
        EndpointLabelPolicy::MaximumOnly => chart.tolerance.is_close(value, 1.0),
        EndpointLabelPolicy::InteriorOnly | EndpointLabelPolicy::None => false,
    };
    if !requested {
        return false;
    }
    let [first, second] = axis.component().others();
    let corner = if chart.tolerance.is_close(value, 0.0) {
        first
    } else {
        second
    };
    let named = corner_visibility != CornerLabelVisibility::Hidden
        && corner_names[corner.index()].is_some()
        && chart
            .viewport
            .contains(chart.geometry.vertex(corner), chart.tolerance)
            .unwrap_or(false);
    if named {
        return false;
    }
    // Stable A/B/C ownership avoids duplicate endpoint labels at every shared corner.
    TernaryAxis::ALL
        .into_iter()
        .filter(|candidate| *candidate != TernaryAxis::from_component(corner))
        .min()
        == Some(axis)
}

fn draw_corner_names<DB: DrawingBackend>(
    chart: &TernaryChart<'_, DB>,
    names: &[Option<String>; 3],
    style: &AxisTextStyle,
    offset: u32,
    visibility: CornerLabelVisibility,
) -> Result<(), TernaryChartError<DB::ErrorType>> {
    for component in Component::ALL {
        let Some(name) = &names[component.index()] else {
            continue;
        };
        let vertex = chart.geometry.vertex(component);
        if visibility != CornerLabelVisibility::Always
            && !chart.viewport.contains(vertex, chart.tolerance)?
        {
            continue;
        }
        let layout = corner_label_layout(chart, component, offset);
        let text = EmptyElement::<_, DB>::at((vertex.x, vertex.y))
            + Text::new(
                name.clone(),
                layout.offset,
                style.plotters_style().pos(layout.position),
            );
        chart.plotting_area().draw(&text)?;
    }
    Ok(())
}
pub(crate) fn axis_name_anchor(
    geometry: TernaryGeometry,
    component: Component,
) -> TernaryCartesian {
    let [first, second] = component.others();
    midpoint(geometry.vertex(first), geometry.vertex(second))
}
fn midpoint(a: TernaryCartesian, b: TernaryCartesian) -> TernaryCartesian {
    TernaryCartesian::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0)
}
fn centroid(geometry: TernaryGeometry) -> TernaryCartesian {
    let [a, b, c] = geometry.vertices();
    TernaryCartesian::new((a.x + b.x + c.x) / 3.0, (a.y + b.y + c.y) / 3.0)
}
#[derive(Clone, Copy)]
struct LabelLayout {
    offset: (i32, i32),
    angle: f64,
    position: Pos,
}
fn pixel_offset(vector: (f64, f64), distance: u32) -> (i32, i32) {
    (
        (vector.0 * f64::from(distance)).round() as i32,
        (vector.1 * f64::from(distance)).round() as i32,
    )
}
fn outward_normal<DB: DrawingBackend>(
    chart: &TernaryChart<'_, DB>,
    anchor: TernaryCartesian,
) -> (f64, f64) {
    let c = centroid(chart.geometry);
    let cp = chart.plotting_area().map_coordinate(&(c.x, c.y));
    let ap = chart.plotting_area().map_coordinate(&(anchor.x, anchor.y));
    let dx = f64::from(ap.0 - cp.0);
    let dy = f64::from(ap.1 - cp.1);
    let len = dx.hypot(dy);
    if len <= f64::EPSILON {
        (0.0, -1.0)
    } else {
        (dx / len, dy / len)
    }
}
fn corner_label_layout<DB: DrawingBackend>(
    chart: &TernaryChart<'_, DB>,
    component: Component,
    distance: u32,
) -> LabelLayout {
    let vertex = chart.geometry.vertex(component);
    let normal = outward_normal(chart, vertex);
    let offset = pixel_offset(normal, distance);
    let position = Pos::new(
        if offset.0 < 0 {
            HPos::Right
        } else if offset.0 > 0 {
            HPos::Left
        } else {
            HPos::Center
        },
        if offset.1 < 0 {
            VPos::Bottom
        } else {
            VPos::Top
        },
    );
    LabelLayout {
        offset,
        angle: 0.0,
        position,
    }
}
fn axis_name_layout<DB: DrawingBackend>(
    chart: &TernaryChart<'_, DB>,
    edge: CartesianSegment,
    anchor: TernaryCartesian,
    distance: u32,
) -> LabelLayout {
    let first = chart
        .plotting_area()
        .map_coordinate(&(edge.start.x, edge.start.y));
    let second = chart
        .plotting_area()
        .map_coordinate(&(edge.end.x, edge.end.y));
    let (mut dx, mut dy) = (f64::from(second.0 - first.0), f64::from(second.1 - first.1));
    if dx < 0.0 || (dx.abs() <= f64::EPSILON && dy < 0.0) {
        dx = -dx;
        dy = -dy;
    }
    LabelLayout {
        offset: pixel_offset(outward_normal(chart, anchor), distance),
        angle: dy.atan2(dx),
        position: Pos::new(HPos::Center, VPos::Center),
    }
}
fn tick_endpoints(
    normal: (f64, f64),
    length: u32,
    direction: TickDirection,
) -> ((i32, i32), (i32, i32)) {
    let vector = |amount: f64| {
        (
            (normal.0 * amount).round() as i32,
            (normal.1 * amount).round() as i32,
        )
    };
    let length = f64::from(length);
    match direction {
        TickDirection::Outward => ((0, 0), vector(length)),
        TickDirection::Inward => (vector(-length), (0, 0)),
        TickDirection::Both => (vector(-length / 2.0), vector(length / 2.0)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EQUILATERAL_TRIANGLE_HEIGHT, TernaryViewport, TriangleOrientation, VertexOrder};
    #[test]
    fn count_step_and_values_are_deterministic() {
        let t = Tolerance::default();
        assert_eq!(
            resolve_tick_values::<std::io::Error>(&TickSpec::Count(4), (0.0, 1.0), t).unwrap(),
            vec![0.0, 0.25, 0.5, 0.75, 1.0]
        );
        assert_eq!(
            resolve_tick_values::<std::io::Error>(&TickSpec::Step(0.25), (0.0, 1.0), t).unwrap(),
            vec![0.0, 0.25, 0.5, 0.75, 1.0]
        );
        assert_eq!(
            resolve_tick_values::<std::io::Error>(
                &TickSpec::Values(vec![1.0, 0.5, 0.5 + 1e-13, 0.0]),
                (0.0, 1.0),
                t
            )
            .unwrap(),
            vec![0.0, 0.5, 1.0]
        );
    }
    #[test]
    fn invalid_ticks_are_errors() {
        let t = Tolerance::default();
        assert!(matches!(
            resolve_tick_values::<std::io::Error>(&TickSpec::Count(0), (0.0, 1.0), t),
            Err(TernaryChartError::InvalidTickCount { .. })
        ));
        for value in [0.0, -0.1, 1.1, f64::NAN] {
            assert!(matches!(
                resolve_tick_values::<std::io::Error>(&TickSpec::Step(value), (0.0, 1.0), t),
                Err(TernaryChartError::InvalidTickStep { .. })
            ));
        }
    }
    #[test]
    fn formatting_preserves_percent_and_unicode() {
        assert_eq!(
            AxisLabelFormat::Percentage { precision: 0 }.format(0.25),
            "25%"
        );
        assert_eq!(
            AxisLabelFormat::Custom(Arc::new(|v| format!("SiO\u{2082} {v:.1}"))).format(0.5),
            "SiO\u{2082} 0.5"
        );
    }
    #[test]
    fn semantic_edges_follow_custom_order() {
        let g = TernaryGeometry::new(
            TriangleOrientation::Up,
            VertexOrder::new(Component::C, Component::A, Component::B).unwrap(),
        );
        let edge = semantic_axis_edge(g, Component::A);
        assert_eq!(edge.start, g.vertex(Component::B));
        assert_eq!(edge.end, g.vertex(Component::C));
        assert_eq!(
            axis_name_anchor(g, Component::A),
            TernaryCartesian::new(0.25, EQUILATERAL_TRIANGLE_HEIGHT / 2.0)
        );
    }
    #[test]
    fn visible_edge_ranges_do_not_use_viewport_sides() {
        let g = TernaryGeometry::default();
        let v = TernaryViewport::new(0.3, 0.7, 0.15, 0.35).unwrap();
        let source = semantic_axis_edge(g, Component::A);
        assert!(
            clip_segment_with_parameters(source, v, Tolerance::default())
                .unwrap()
                .is_none()
        );
    }
}
