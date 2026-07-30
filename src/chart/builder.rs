use plotters::backend::DrawingBackend;
use plotters::chart::ChartBuilder;
use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters::style::{IntoTextStyle, TextStyle};

use crate::coord::{
    PixelRect, TernaryGeometry, TernaryViewport, Tolerance, ViewportAlignment, ViewportFit,
    ViewportTransform,
};

use super::{TernaryChart, TernaryChartError};

const DEFAULT_MARGIN: u32 = 20;

/// Builder for an owned, Cartesian-backed [`TernaryChart`].
///
/// The implicit viewport is resolved at build time from the current geometry.
/// Once [`TernaryChartBuilder::viewport`] is called, later geometry changes do
/// not replace that explicit viewport.
pub struct TernaryChartBuilder<'root, DB: DrawingBackend> {
    root: &'root DrawingArea<DB, Shift>,
    geometry: TernaryGeometry,
    viewport: Option<TernaryViewport>,
    fit: ViewportFit,
    alignment: ViewportAlignment,
    caption: Option<(String, TextStyle<'root>)>,
    margin: u32,
    tolerance: Tolerance,
}

impl<'root, DB: DrawingBackend> TernaryChartBuilder<'root, DB> {
    /// Begin a ternary chart in a Plotters root drawing area.
    pub fn on(root: &'root DrawingArea<DB, Shift>) -> Self {
        Self {
            root,
            geometry: TernaryGeometry::default(),
            viewport: None,
            fit: ViewportFit::default(),
            alignment: ViewportAlignment::default(),
            caption: None,
            margin: DEFAULT_MARGIN,
            tolerance: Tolerance::default(),
        }
    }

    /// Set the triangle geometry.
    ///
    /// An implicit full viewport follows this geometry at build time. An
    /// explicitly selected viewport is preserved.
    pub const fn geometry(mut self, geometry: TernaryGeometry) -> Self {
        self.geometry = geometry;
        self
    }

    /// Select an explicit logical clipping viewport.
    pub const fn viewport(mut self, viewport: TernaryViewport) -> Self {
        self.viewport = Some(viewport);
        self
    }

    /// Select aspect preservation or explicit stretching.
    pub const fn viewport_fit(mut self, fit: ViewportFit) -> Self {
        self.fit = fit;
        self
    }

    /// Align an aspect-fitted plotting subarea.
    pub const fn viewport_alignment(mut self, alignment: ViewportAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Add a Plotters-rendered caption outside the fitted ternary viewport.
    pub fn caption<S, Style>(mut self, caption: S, style: Style) -> Self
    where
        S: AsRef<str>,
        Style: IntoTextStyle<'root>,
    {
        self.caption = Some((
            caption.as_ref().to_owned(),
            style.into_text_style(self.root),
        ));
        self
    }

    /// Set equal outer margins in backend pixels.
    pub const fn margin(mut self, margin: u32) -> Self {
        self.margin = margin;
        self
    }

    /// Build the owned Cartesian chart on an aspect-fitted Plotters subarea.
    pub fn build<'chart>(self) -> Result<TernaryChart<'chart, DB>, TernaryChartError<DB::ErrorType>>
    where
        DB: 'chart,
    {
        let viewport = self.resolved_viewport();
        let mut allocated = self.root.clone();
        if let Some((caption, style)) = self.caption {
            allocated = allocated.titled(&caption, style)?;
        }

        let (available_width, available_height) = allocated.dim_in_pixel();
        let total_margin = self.margin.saturating_mul(2);
        if available_width <= total_margin || available_height <= total_margin {
            return Err(TernaryChartError::InsufficientDrawingArea {
                width: available_width.saturating_sub(total_margin),
                height: available_height.saturating_sub(total_margin),
            });
        }
        allocated = allocated.margin(self.margin, self.margin, self.margin, self.margin);

        let (width, height) = allocated.dim_in_pixel();

        let pixel_rect = PixelRect::new(0, 0, width, height)?;
        let transform = ViewportTransform::new(viewport, pixel_rect, self.fit, self.alignment)?;
        let fitted = integer_fitted_rect(transform, width, height)?;
        let fitted_area = allocated.shrink(
            (fitted.x, fitted.y),
            (fitted.width as i32, fitted.height as i32),
        );

        let context = ChartBuilder::on(&fitted_area).build_cartesian_2d(
            viewport.x_min()..viewport.x_max(),
            viewport.y_min()..viewport.y_max(),
        )?;

        Ok(TernaryChart {
            context,
            geometry: self.geometry,
            viewport,
            fit: self.fit,
            alignment: self.alignment,
            tolerance: self.tolerance,
        })
    }

    pub(crate) fn resolved_viewport(&self) -> TernaryViewport {
        self.viewport
            .unwrap_or_else(|| TernaryViewport::full(self.geometry))
    }
}

fn integer_fitted_rect<E: std::error::Error + Send + Sync>(
    transform: ViewportTransform,
    allocated_width: u32,
    allocated_height: u32,
) -> Result<PixelRect, TernaryChartError<E>> {
    let bounds = transform.fitted_pixel_bounds();
    let x_min = bounds.x_min.round().clamp(0.0, f64::from(allocated_width)) as i32;
    let y_min = bounds.y_min.round().clamp(0.0, f64::from(allocated_height)) as i32;
    let x_max = bounds.x_max.round().clamp(0.0, f64::from(allocated_width)) as i32;
    let y_max = bounds.y_max.round().clamp(0.0, f64::from(allocated_height)) as i32;
    let width = (x_max - x_min).max(0) as u32;
    let height = (y_max - y_min).max(0) as u32;
    if width == 0 || height == 0 {
        return Err(TernaryChartError::InsufficientDrawingArea { width, height });
    }
    PixelRect::new(x_min, y_min, width, height).map_err(TernaryChartError::from)
}

#[cfg(test)]
mod tests {
    use plotters::prelude::*;

    use super::*;
    use crate::{TriangleOrientation, VertexOrder};

    #[test]
    fn explicit_viewport_survives_later_geometry_selection() {
        let mut buffer = vec![0; 200 * 200 * 3];
        let root = BitMapBackend::with_buffer(&mut buffer, (200, 200)).into_drawing_area();
        let viewport = TernaryViewport::new(0.6, 0.9, 0.1, 0.4).unwrap();
        let geometry = TernaryGeometry::new(TriangleOrientation::Down, VertexOrder::default());
        let builder = TernaryChartBuilder::on(&root)
            .viewport(viewport)
            .geometry(geometry);
        assert_eq!(builder.resolved_viewport(), viewport);
    }

    #[test]
    fn implicit_viewport_tracks_downward_geometry() {
        let mut buffer = vec![0; 200 * 200 * 3];
        let root = BitMapBackend::with_buffer(&mut buffer, (200, 200)).into_drawing_area();
        let geometry = TernaryGeometry::new(TriangleOrientation::Down, VertexOrder::default());
        let builder = TernaryChartBuilder::on(&root).geometry(geometry);
        assert_eq!(builder.resolved_viewport(), TernaryViewport::full(geometry));
    }

    #[test]
    fn excessive_margin_returns_a_layout_error() {
        let mut buffer = vec![0; 100 * 100 * 3];
        let root = BitMapBackend::with_buffer(&mut buffer, (100, 100)).into_drawing_area();
        let result = TernaryChartBuilder::on(&root).margin(60).build();
        assert!(matches!(
            result,
            Err(TernaryChartError::InsufficientDrawingArea {
                width: 0,
                height: 0
            })
        ));
    }

    #[test]
    fn preserve_aspect_keeps_equal_logical_scales_after_integer_rounding() {
        let geometry = TernaryGeometry::new(TriangleOrientation::Down, VertexOrder::default());
        let viewport = TernaryViewport::full(geometry);
        let transform = ViewportTransform::new(
            viewport,
            PixelRect::new(0, 0, 920, 700).unwrap(),
            ViewportFit::PreserveAspect,
            ViewportAlignment::Center,
        )
        .unwrap();
        let fitted = integer_fitted_rect::<std::io::Error>(transform, 920, 700).unwrap();
        let x_scale = f64::from(fitted.width) / viewport.width();
        let y_scale = f64::from(fitted.height) / viewport.height();
        assert!((x_scale - y_scale).abs() <= 1.0);
    }
}
