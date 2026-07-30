use plotters::backend::DrawingBackend;
use plotters::chart::SeriesAnno;
use plotters::element::PathElement;

use crate::chart::{TernaryChart, TernaryChartError};
use crate::coord::TernaryPoint;

use super::{
    MarkerElement, MarkerStyle, PointMarkerStyleProvider, SeriesError, TernaryLineSeries,
    TernaryPointSeries, TernarySmoothSeries, prepare_points_with_source, prepare_polyline,
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

impl<DB, I, P, Provider> TernarySeries<DB> for TernaryPointSeries<I, Provider>
where
    DB: DrawingBackend,
    I: IntoIterator<Item = P>,
    P: Into<TernaryPoint>,
    Provider: PointMarkerStyleProvider,
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
            size,
            legacy_style,
            marker,
            explicit_style,
            provider,
            clip_mode,
            normalization,
            tolerance,
            invalid_policy,
        ) = self.into_parts();
        if size == 0 {
            return Err(SeriesError::InvalidMarkerSize { size }.into());
        }
        let fallback =
            explicit_style.unwrap_or(MarkerStyle::from_legacy(marker, legacy_style).map_err(
                |source| SeriesError::Marker {
                    index: None,
                    source,
                },
            )?);
        let elements = prepare_points_with_source(
            chart.geometry,
            chart.viewport,
            points,
            normalization,
            tolerance,
            invalid_policy,
            clip_mode,
        )?
        .into_iter()
        .map(|point| {
            let style = provider.marker_style(point.index, point.composition, &fallback);
            MarkerElement::new((point.cartesian.x, point.cartesian.y), size, style).map_err(
                |source| SeriesError::Marker {
                    index: Some(point.index),
                    source,
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
        chart.context.draw_series(elements).map_err(Into::into)
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
