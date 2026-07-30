use plotters::backend::DrawingBackend;
use plotters::chart::ChartContext;
use plotters::coord::cartesian::Cartesian2d;
use plotters::coord::types::RangedCoordf64;
use plotters::drawing::DrawingArea;

use crate::coord::{TernaryGeometry, TernaryViewport, Tolerance, ViewportAlignment, ViewportFit};

use super::TernaryMeshConfig;

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
    pub fn configure_mesh(&mut self) -> TernaryMeshConfig<'_, 'a, 'static, DB> {
        TernaryMeshConfig::new(self)
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
