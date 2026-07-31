use plotters::backend::DrawingBackend;
use plotters::chart::SeriesAnno;
use plotters::element::{EmptyElement, PathElement, Text};

use crate::chart::{TernaryChart, TernaryChartError};
use crate::coord::TernaryPoint;

use super::{
    AnnotationClipMode, AnnotationError, MarkerElement, MarkerStyle, PointMarkerStyleProvider,
    PolygonElement, SeriesError, TernaryContourSeries, TernaryLineSeries, TernaryPointSeries,
    TernaryPolygon, TernarySmoothSeries, TernaryText, prepare_points_with_source, prepare_polygon,
    prepare_polyline,
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

impl<DB, I, P> TernarySeries<DB> for TernaryPolygon<I>
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
        let (points, fill, border, normalization, tolerance) = self.into_parts();
        let polygon = prepare_polygon(
            points,
            chart.geometry,
            chart.viewport,
            normalization,
            tolerance,
        )
        .map_err(SeriesError::Polygon)?;
        let vertices = polygon
            .vertices()
            .iter()
            .map(|point| (point.x, point.y))
            .collect();
        chart
            .context
            .draw_series(std::iter::once(PolygonElement::new(vertices, fill, border)))
            .map_err(Into::into)
    }
}

impl<DB> TernarySeries<DB> for TernaryText
where
    DB: DrawingBackend,
{
    fn draw<'chart, 'series>(
        self,
        chart: &'chart mut TernaryChart<'series, DB>,
    ) -> Result<&'chart mut SeriesAnno<'series, DB>, TernaryChartError<DB::ErrorType>>
    where
        DB: 'series,
    {
        let (point, text, style, anchor, offset, clip_mode, rotation, normalization, tolerance) =
            self.into_parts();
        let logical = chart
            .geometry
            .project(point, normalization, tolerance)
            .map_err(|source| SeriesError::Annotation(AnnotationError::InvalidAnchor { source }))?;
        let visible =
            clip_mode == AnnotationClipMode::None || chart.viewport.contains(logical, tolerance)?;
        if !visible {
            return chart
                .context
                .draw_series(std::iter::empty::<PathElement<(f64, f64)>>())
                .map_err(Into::into);
        }
        let text_style = style
            .plotters_style()
            .pos(anchor.plotters_pos())
            .transform(rotation.plotters_transform());
        let element =
            EmptyElement::at((logical.x, logical.y)) + Text::new(text, offset, text_style);
        chart
            .context
            .draw_series(std::iter::once(element))
            .map_err(Into::into)
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

impl<'contour, DB> TernarySeries<DB> for TernaryContourSeries<'contour>
where
    DB: DrawingBackend,
{
    fn draw<'chart, 'series>(
        self,
        chart: &'chart mut TernaryChart<'series, DB>,
    ) -> Result<&'chart mut SeriesAnno<'series, DB>, TernaryChartError<DB::ErrorType>>
    where
        DB: 'series,
    {
        let (contours, uniform_style, style_provider) = self.into_parts();
        let mut elements = Vec::new();
        for level in &contours.levels {
            let style = style_provider
                .as_ref()
                .map_or(uniform_style, |provider| provider(level.value));
            for path in &level.paths {
                let mut source = path.points.clone();
                if path.closed && !source.is_empty() {
                    source.push(source[0]);
                }
                let visible = prepare_polyline(
                    chart.geometry,
                    chart.viewport,
                    source,
                    crate::coord::Normalization::RequireUnitSum,
                    chart.tolerance,
                    super::InvalidPointPolicy::Error,
                )?;
                elements.extend(visible.into_iter().map(|path| {
                    PathElement::new(
                        path.into_iter()
                            .map(|point| (point.x, point.y))
                            .collect::<Vec<_>>(),
                        style,
                    )
                }));
            }
        }
        chart.context.draw_series(elements).map_err(Into::into)
    }
}
