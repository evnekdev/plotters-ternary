use plotters::backend::DrawingBackend;
use plotters::chart::SeriesAnno;
use plotters::element::{EmptyElement, PathElement, Text};
use plotters::style::{Color, ShapeStyle};

use crate::chart::{TernaryChart, TernaryChartError};
use crate::coord::TernaryPoint;

use super::{
    AnnotationClipMode, AnnotationError, MarkerElement, MarkerStyle, PointMarkerStyleProvider,
    PolygonElement, SeriesError, TernaryContourBandSeries, TernaryContourSeries, TernaryLineSeries,
    TernaryPointSeries, TernaryPolygon, TernaryScalarMapSeries, TernarySmoothSeries, TernaryText,
    prepare_points_with_source, prepare_polygon, prepare_polyline,
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
        let parts = self.into_parts();
        if matches!(parts.legend, super::ContourLegendPolicy::None) {
            let mut elements = Vec::new();
            for (index, level) in parts.contours.levels.iter().enumerate() {
                let style = parts.styles.style_for(index, level.value)?;
                elements.extend(contour_level_elements(chart, level, style)?);
            }
            return chart.context.draw_series(elements).map_err(Into::into);
        }

        for (index, level) in parts.contours.levels.iter().enumerate() {
            let style = parts.styles.style_for(index, level.value)?;
            let elements = contour_level_elements(chart, level, style)?;
            let annotation = chart.context.draw_series(elements)?;
            if parts.legend.selected(index, level.value)? {
                annotation
                    .label((parts.formatter)(level.value))
                    .legend(move |(x, y)| PathElement::new([(x, y), (x + 24, y)], style));
            }
        }
        chart
            .context
            .draw_series(std::iter::empty::<PathElement<(f64, f64)>>())
            .map_err(Into::into)
    }
}

fn contour_level_elements<DB: DrawingBackend>(
    chart: &TernaryChart<'_, DB>,
    level: &crate::ContourLevel,
    style: plotters::style::ShapeStyle,
) -> Result<Vec<PathElement<(f64, f64)>>, SeriesError> {
    let mut elements = Vec::new();
    for path in &level.paths {
        let mut source = path
            .points
            .iter()
            .copied()
            .map(|point| TernaryPoint::from(point.as_array()))
            .collect::<Vec<_>>();
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
    Ok(elements)
}

impl<'bands, DB> TernarySeries<DB> for TernaryContourBandSeries<'bands>
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
        let (bands, styles, border) = self.into_parts();
        let mut elements = Vec::new();
        for (index, band) in bands.bands.iter().enumerate() {
            let fill = styles.style_for(index, band)?;
            for region in &band.regions {
                let polygon = prepare_polygon(
                    region
                        .exterior
                        .iter()
                        .copied()
                        .map(|point| TernaryPoint::from(point.as_array())),
                    chart.geometry,
                    chart.viewport,
                    crate::coord::Normalization::RequireUnitSum,
                    chart.tolerance,
                )
                .map_err(SeriesError::Polygon)?;
                elements.push(PolygonElement::new(
                    polygon
                        .vertices()
                        .iter()
                        .map(|point| (point.x, point.y))
                        .collect(),
                    Some(fill),
                    border,
                ));
            }
        }
        chart.context.draw_series(elements).map_err(Into::into)
    }
}

impl<'field, DB> TernarySeries<DB> for TernaryScalarMapSeries<'field>
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
        let (field, requested_range, resolution, opacity, color_map) = self.into_parts();
        let intervals = resolution.intervals()?;
        if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
            return Err(SeriesError::InvalidScalarMapOpacity { opacity }.into());
        }
        let (minimum, maximum) = requested_range.unwrap_or_else(|| {
            field.values().iter().copied().fold(
                (f64::INFINITY, f64::NEG_INFINITY),
                |(minimum, maximum), value| (minimum.min(value), maximum.max(value)),
            )
        });
        if !minimum.is_finite() || !maximum.is_finite() || maximum <= minimum {
            return Err(SeriesError::InvalidContourColorRange {
                minimum,
                maximum,
                stroke_width: 1,
            }
            .into());
        }

        let mut elements = Vec::new();
        for triangle in field
            .elementary_triangles()
            .map_err(SeriesError::ScalarMapField)?
        {
            let coordinates = [
                TernaryPoint::from(
                    field
                        .composition(triangle.vertices[0])
                        .map_err(SeriesError::ScalarMapField)?,
                ),
                TernaryPoint::from(
                    field
                        .composition(triangle.vertices[1])
                        .map_err(SeriesError::ScalarMapField)?,
                ),
                TernaryPoint::from(
                    field
                        .composition(triangle.vertices[2])
                        .map_err(SeriesError::ScalarMapField)?,
                ),
            ];
            let values = [
                field
                    .value(triangle.vertices[0])
                    .map_err(SeriesError::ScalarMapField)?,
                field
                    .value(triangle.vertices[1])
                    .map_err(SeriesError::ScalarMapField)?,
                field
                    .value(triangle.vertices[2])
                    .map_err(SeriesError::ScalarMapField)?,
            ];
            for micro in microtriangles(coordinates, values, intervals) {
                let average = (micro.values[0] + micro.values[1] + micro.values[2]) / 3.0;
                let normalized = ((average - minimum) / (maximum - minimum)).clamp(0.0, 1.0);
                let polygon = prepare_polygon(
                    micro.points,
                    chart.geometry,
                    chart.viewport,
                    crate::coord::Normalization::RequireUnitSum,
                    chart.tolerance,
                )
                .map_err(SeriesError::Polygon)?;
                let color = color_map(normalized).mix(opacity);
                elements.push(PolygonElement::new(
                    polygon
                        .vertices()
                        .iter()
                        .map(|point| (point.x, point.y))
                        .collect(),
                    Some(ShapeStyle::from(color).filled()),
                    None,
                ));
            }
        }
        chart.context.draw_series(elements).map_err(Into::into)
    }
}

#[derive(Clone, Copy)]
struct ScalarMicroTriangle {
    points: [TernaryPoint; 3],
    values: [f64; 3],
}

fn microtriangles(
    points: [TernaryPoint; 3],
    values: [f64; 3],
    intervals: usize,
) -> Vec<ScalarMicroTriangle> {
    let mut result = Vec::with_capacity(intervals * intervals);
    let sample = |i: usize, j: usize| {
        let u = i as f64 / intervals as f64;
        let v = j as f64 / intervals as f64;
        let w = 1.0 - u - v;
        let coordinates = points.map(TernaryPoint::as_array);
        let value = values[0] * w + values[1] * u + values[2] * v;
        (
            TernaryPoint::new(
                coordinates[0][0] * w + coordinates[1][0] * u + coordinates[2][0] * v,
                coordinates[0][1] * w + coordinates[1][1] * u + coordinates[2][1] * v,
                coordinates[0][2] * w + coordinates[1][2] * u + coordinates[2][2] * v,
            ),
            value,
        )
    };
    for i in 0..intervals {
        for j in 0..(intervals - i) {
            let (a, av) = sample(i, j);
            let (b, bv) = sample(i + 1, j);
            let (c, cv) = sample(i, j + 1);
            result.push(ScalarMicroTriangle {
                points: [a, b, c],
                values: [av, bv, cv],
            });
            if i + j + 1 < intervals {
                let (d, dv) = sample(i + 1, j + 1);
                result.push(ScalarMicroTriangle {
                    points: [b, d, c],
                    values: [bv, dv, cv],
                });
            }
        }
    }
    result
}
