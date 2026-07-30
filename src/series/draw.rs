use plotters::backend::DrawingBackend;
use plotters::chart::SeriesAnno;
use plotters::element::{Circle, Cross, PathElement, TriangleMarker};

use crate::chart::{TernaryChart, TernaryChartError};
use crate::coord::TernaryPoint;

use super::{
    MarkerShape, SeriesError, TernaryLineSeries, TernaryPointSeries, TernarySmoothSeries,
    prepare_points, prepare_polyline,
    smooth::{SmoothPreparation, prepare_smooth_polyline},
};

/// A ternary-aware series that can submit owned Plotters elements to a chart.
///
/// This trait exists so [`TernaryChart::draw_series`] can accept both line and
/// built-in point series while returning Plotters' native annotation object.
pub trait TernarySeries<DB: DrawingBackend> {
    #[doc(hidden)]
    fn draw<'chart, 'series>(
        self,
        chart: &'chart mut TernaryChart<'series, DB>,
    ) -> Result<&'chart mut SeriesAnno<'series, DB>, TernaryChartError<DB::ErrorType>>
    where
        DB: 'series;
}

impl<DB, I, P> TernarySeries<DB> for TernaryLineSeries<I>
where
    DB: DrawingBackend,
    I: IntoIterator<Item = P>,
    P: Into<TernaryPoint>,
{
    fn draw<'chart, 'series>(
        self,
        chart: &'chart mut TernaryChart<'series, DB>,
    ) -> Result<&'chart mut SeriesAnno<'series, DB>, TernaryChartError<DB::ErrorType>>
    where
        DB: 'series,
    {
        let (points, style, normalization, tolerance, invalid_policy) = self.into_parts();
        let paths = prepare_polyline(
            chart.geometry,
            chart.viewport,
            points,
            normalization,
            tolerance,
            invalid_policy,
        )?;
        let elements = paths.into_iter().map(|path| {
            let coordinates: Vec<_> = path.into_iter().map(|point| (point.x, point.y)).collect();
            PathElement::new(coordinates, style)
        });
        chart.context.draw_series(elements).map_err(Into::into)
    }
}

impl<DB, I, P> TernarySeries<DB> for TernaryPointSeries<I>
where
    DB: DrawingBackend,
    I: IntoIterator<Item = P>,
    P: Into<TernaryPoint>,
{
    fn draw<'chart, 'series>(
        self,
        chart: &'chart mut TernaryChart<'series, DB>,
    ) -> Result<&'chart mut SeriesAnno<'series, DB>, TernaryChartError<DB::ErrorType>>
    where
        DB: 'series,
    {
        let (points, size, style, marker, clip_mode, normalization, tolerance, invalid_policy) =
            self.into_parts();
        if size == 0 {
            return Err(SeriesError::InvalidMarkerSize { size }.into());
        }
        let anchors = prepare_points(
            chart.geometry,
            chart.viewport,
            points,
            normalization,
            tolerance,
            invalid_policy,
            clip_mode,
        )?
        .into_iter()
        .map(|point| (point.x, point.y));

        match marker {
            MarkerShape::Circle => chart
                .context
                .draw_series(anchors.map(|point| Circle::new(point, size, style)))
                .map_err(Into::into),
            MarkerShape::Cross => chart
                .context
                .draw_series(anchors.map(|point| Cross::new(point, size, style)))
                .map_err(Into::into),
            MarkerShape::Triangle => chart
                .context
                .draw_series(anchors.map(|point| TriangleMarker::new(point, size, style)))
                .map_err(Into::into),
        }
    }
}
impl<DB, I, P> TernarySeries<DB> for TernarySmoothSeries<I>
where
    DB: DrawingBackend,
    I: IntoIterator<Item = P>,
    P: Into<TernaryPoint>,
{
    fn draw<'chart, 'series>(
        self,
        chart: &'chart mut TernaryChart<'series, DB>,
    ) -> Result<&'chart mut SeriesAnno<'series, DB>, TernaryChartError<DB::ErrorType>>
    where
        DB: 'series,
    {
        let (
            points,
            style,
            interpolation,
            samples_per_interval,
            normalization,
            tolerance,
            invalid_policy,
        ) = self.into_parts();
        let paths = prepare_smooth_polyline(
            chart.geometry,
            chart.viewport,
            points,
            SmoothPreparation {
                interpolation,
                samples_per_interval,
                normalization,
                tolerance,
                invalid_policy,
            },
        )?;
        let elements = paths.into_iter().map(|path| {
            let coordinates: Vec<_> = path.into_iter().map(|point| (point.x, point.y)).collect();
            PathElement::new(coordinates, style)
        });
        chart.context.draw_series(elements).map_err(Into::into)
    }
}
