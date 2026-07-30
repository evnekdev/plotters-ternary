use plotters::backend::DrawingBackend;
use plotters::chart::{ChartContext, SeriesAnno, SeriesLabelStyle};
use plotters::coord::cartesian::Cartesian2d;
use plotters::coord::types::RangedCoordf64;
use plotters::drawing::DrawingArea;
use plotters::element::{Drawable, PointCollection};
use plotters::style::ShapeStyle;

use crate::coord::{TernaryGeometry, TernaryViewport, Tolerance, ViewportAlignment, ViewportFit};
use crate::series::{SeriesError, TernaryPointSeries, TernarySeries, prepare_points};

use super::{TernaryChartError, TernaryMeshConfig};

/// The concrete Cartesian context used internally by a ternary chart.
pub type CartesianChartContext<'a, DB> =
    ChartContext<'a, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>;

/// The concrete logical plotting area used internally by a ternary chart.
pub type CartesianPlottingArea<DB> = DrawingArea<DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>;

/// A ternary chart backed by an owned Plotters Cartesian chart context.
///
/// The lifetime parameter belongs to Plotters' stored series annotations. The
/// context owns a cloned drawing area and does not borrow the builder's root.
pub struct TernaryChart<'a, DB: DrawingBackend> {
    pub(crate) context: CartesianChartContext<'a, DB>,
    pub(crate) geometry: TernaryGeometry,
    pub(crate) viewport: TernaryViewport,
    pub(crate) fit: ViewportFit,
    pub(crate) alignment: ViewportAlignment,
    pub(crate) tolerance: Tolerance,
}

impl<'a, DB: DrawingBackend> TernaryChart<'a, DB> {
    /// Configure and draw the Milestone 3 boundary, grid, and basic names.
    pub fn configure_mesh(&mut self) -> TernaryMeshConfig<'_, 'a, 'static, 'static, DB> {
        TernaryMeshConfig::new(self)
    }
    /// Draw a ternary-aware series and return Plotters' native annotation.
    pub fn draw_series<'chart, S>(
        &'chart mut self,
        series: S,
    ) -> Result<&'chart mut SeriesAnno<'a, DB>, TernaryChartError<DB::ErrorType>>
    where
        DB: 'a,
        S: TernarySeries<DB>,
    {
        series.draw(self)
    }

    /// Draw points with an owned ordinary Plotters element from a custom closure.
    ///
    /// The closure receives the projected logical anchor, configured backend
    /// pixel size, and Plotters `ShapeStyle`. Composable elements can begin at
    /// `EmptyElement::at(anchor)` and use local backend coordinates thereafter.
    pub fn draw_point_series<'chart, I, P, F, E>(
        &'chart mut self,
        series: TernaryPointSeries<I>,
        make_marker: F,
    ) -> Result<&'chart mut SeriesAnno<'a, DB>, TernaryChartError<DB::ErrorType>>
    where
        DB: 'a,
        I: IntoIterator<Item = P>,
        P: Into<crate::coord::TernaryPoint>,
        F: Fn((f64, f64), u32, ShapeStyle) -> E,
        for<'element> &'element E: PointCollection<'element, (f64, f64)>,
        E: Drawable<DB>,
    {
        let (points, size, style, _marker, clip_mode, normalization, tolerance, invalid_policy) =
            series.into_parts();
        if size == 0 {
            return Err(SeriesError::InvalidMarkerSize { size }.into());
        }
        let anchors = prepare_points(
            self.geometry,
            self.viewport,
            points,
            normalization,
            tolerance,
            invalid_policy,
            clip_mode,
        )?;
        let elements = anchors
            .into_iter()
            .map(move |point| make_marker((point.x, point.y), size, style));
        self.context.draw_series(elements).map_err(Into::into)
    }

    /// Forward ordinary Plotters legend configuration without wrapping it.
    pub fn configure_series_labels<'chart>(
        &'chart mut self,
    ) -> SeriesLabelStyle<'a, 'chart, DB, Cartesian2d<RangedCoordf64, RangedCoordf64>>
    where
        DB: 'a,
    {
        self.context.configure_series_labels()
    }

    /// Borrow the underlying Cartesian Plotters chart.
    pub const fn cartesian_chart(&self) -> &CartesianChartContext<'a, DB> {
        &self.context
    }

    /// Mutably borrow the underlying Cartesian Plotters chart.
    pub fn cartesian_chart_mut(&mut self) -> &mut CartesianChartContext<'a, DB> {
        &mut self.context
    }

    /// Borrow the underlying logical Cartesian plotting area.
    pub fn plotting_area(&self) -> &CartesianPlottingArea<DB> {
        self.context.plotting_area()
    }

    /// Return the triangle geometry used by this chart.
    pub const fn geometry(&self) -> TernaryGeometry {
        self.geometry
    }

    /// Return the requested logical clipping viewport.
    pub const fn viewport(&self) -> TernaryViewport {
        self.viewport
    }

    /// Return the selected fitting policy.
    pub const fn viewport_fit(&self) -> ViewportFit {
        self.fit
    }

    /// Return the selected alignment inside unused allocated space.
    pub const fn viewport_alignment(&self) -> ViewportAlignment {
        self.alignment
    }
}
