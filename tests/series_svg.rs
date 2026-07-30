use std::cell::Cell;

use plotters::prelude::*;
use plotters_ternary::{
    Component, InvalidPointPolicy, MarkerShape, Normalization, SeriesError, TernaryChartBuilder,
    TernaryChartError, TernaryInterpolation, TernaryLineSeries, TernaryPoint, TernaryPointSeries,
    TernarySmoothSeries, TernaryViewport, TriangleOrientation, VertexOrder,
};

fn point(a: f64, b: f64, c: f64) -> TernaryPoint {
    TernaryPoint::new(a, b, c)
}

#[test]
fn native_series_annotations_custom_markers_and_legends_render() {
    let mut svg = String::new();
    let custom_calls = Cell::new(0);
    {
        let root = SVGBackend::with_string(&mut svg, (700, 520)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = TernaryChartBuilder::on(&root)
            .caption("Series integration", ("sans-serif", 24))
            .margin(35)
            .build()
            .unwrap();
        chart
            .configure_mesh()
            .major_step(0.25)
            .hide_axis_names()
            .hide_corner_names()
            .draw()
            .unwrap();

        chart
            .draw_series(TernaryLineSeries::new(
                [point(0.8, 0.1, 0.1), point(0.2, 0.6, 0.2)],
                BLUE.stroke_width(3),
            ))
            .unwrap()
            .label("Native line annotation")
            .legend(|(x, y)| PathElement::new([(x, y), (x + 20, y)], BLUE.stroke_width(3)));

        chart
            .draw_series(
                TernaryPointSeries::new([point(0.2, 0.2, 0.6), point(0.4, 0.3, 0.3)])
                    .size(7)
                    .style(RED.filled())
                    .marker(MarkerShape::Circle),
            )
            .unwrap()
            .label("Built-in circles")
            .legend(|coordinate| Circle::new(coordinate, 6, RED.filled()));

        chart
            .draw_point_series(
                TernaryPointSeries::new([point(2.0, 3.0, 5.0)])
                    .normalization(Normalization::Normalize)
                    .size(8)
                    .style(GREEN.stroke_width(2)),
                |coordinate, size, style| {
                    custom_calls.set(custom_calls.get() + 1);
                    EmptyElement::at(coordinate)
                        + Cross::new((0, 0), size, style)
                        + Circle::new((0, 0), size / 2, style)
                },
            )
            .unwrap()
            .label("Custom composed marker")
            .legend(|coordinate| Cross::new(coordinate, 6, GREEN.stroke_width(2)));

        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.85))
            .border_style(BLACK)
            .label_font(("sans-serif", 16))
            .draw()
            .unwrap();
        drop(chart);
        root.present().unwrap();
    }

    assert_eq!(custom_calls.get(), 1);
    assert!(svg.contains("Native line annotation"));
    assert!(svg.contains("Built-in circles"));
    assert!(svg.contains("Custom composed marker"));
    assert!(svg.contains("<polyline"));
    assert!(svg.contains("<circle"));
    assert!(!svg.contains("<image"));
}

#[test]
fn invalid_line_point_reports_the_original_source_index() {
    let mut svg = String::new();
    let root = SVGBackend::with_string(&mut svg, (400, 300)).into_drawing_area();
    let mut chart = TernaryChartBuilder::on(&root).build().unwrap();
    let result = chart.draw_series(
        TernaryLineSeries::new(
            [
                point(0.2, 0.3, 0.5),
                point(f64::NAN, 0.0, 1.0),
                point(0.3, 0.3, 0.4),
            ],
            BLUE,
        )
        .invalid_point_policy(InvalidPointPolicy::Error),
    );
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("invalid point unexpectedly drew"),
    };
    assert!(matches!(
        error,
        TernaryChartError::Series(SeriesError::InvalidPoint { index: 1, .. })
    ));
}

#[test]
fn custom_vertex_order_and_downward_orientation_draw_without_distortion_or_error() {
    let mut svg = String::new();
    {
        let root = SVGBackend::with_string(&mut svg, (600, 500)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let order = VertexOrder::new(Component::C, Component::A, Component::B).unwrap();
        let geometry = plotters_ternary::TernaryGeometry::new(TriangleOrientation::Down, order);
        let mut chart = TernaryChartBuilder::on(&root)
            .geometry(geometry)
            .viewport(TernaryViewport::full(geometry))
            .build()
            .unwrap();
        chart
            .draw_series(TernaryLineSeries::new(
                [point(0.7, 0.2, 0.1), point(0.1, 0.2, 0.7)],
                MAGENTA.stroke_width(3),
            ))
            .unwrap()
            .label("Semantic order");
        chart
            .draw_series(
                TernaryPointSeries::new([point(0.0, 1.0, 0.0)])
                    .marker(MarkerShape::Triangle)
                    .style(BLACK.filled()),
            )
            .unwrap();
        drop(chart);
        root.present().unwrap();
    }
    assert!(svg.contains("#FF00FF"));
    assert!(svg.contains("<polygon"));
}
#[test]
fn empty_lines_are_a_native_no_op_and_zero_sized_markers_are_rejected() {
    let mut svg = String::new();
    let root = SVGBackend::with_string(&mut svg, (400, 300)).into_drawing_area();
    let mut chart = TernaryChartBuilder::on(&root).build().unwrap();

    chart
        .draw_series(TernaryLineSeries::new(Vec::<TernaryPoint>::new(), BLUE))
        .unwrap()
        .label("Empty native annotation");

    let result = chart.draw_series(TernaryPointSeries::new([point(0.3, 0.3, 0.4)]).size(0));
    assert!(matches!(
        result,
        Err(TernaryChartError::Series(SeriesError::InvalidMarkerSize {
            size: 0
        }))
    ));
}

fn styled_polylines<'a>(svg: &'a str, color: &str) -> Vec<&'a str> {
    svg.lines()
        .filter(|line| line.contains("<polyline") && line.contains(color))
        .collect()
}

fn polyline_points(element: &str) -> Vec<(i32, i32)> {
    let points = element
        .split_once("points=\"")
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0;
    points
        .split_ascii_whitespace()
        .map(|point| {
            let (x, y) = point.split_once(',').unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect()
}

#[test]
fn svg_line_paths_preserve_source_points_without_cosmetic_subdivision() {
    let mut svg = String::new();
    {
        let root = SVGBackend::with_string(&mut svg, (600, 500)).into_drawing_area();
        let mut chart = TernaryChartBuilder::on(&root).margin(30).build().unwrap();
        chart
            .draw_series(TernaryLineSeries::new(
                [
                    point(0.0, 1.0, 0.0),
                    point(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0),
                    point(0.0, 0.0, 1.0),
                ],
                RGBColor(10, 20, 30).stroke_width(3),
            ))
            .unwrap();
        drop(chart);
        root.present().unwrap();
    }

    let paths = styled_polylines(&svg, "#0A141E");
    assert_eq!(paths.len(), 1);
    let points = polyline_points(paths[0]);
    assert_eq!(points.len(), 3);
    assert!(points[0].0 < points[1].0 && points[1].0 < points[2].0);
    assert!(!svg.contains("<path"));
}

#[test]
fn svg_visible_subpaths_remain_separate_and_directed_after_clipping() {
    let geometry = plotters_ternary::TernaryGeometry::default();
    let tolerance = plotters_ternary::Tolerance::default();
    let viewport = TernaryViewport::new(0.4, 0.6, 0.2, 0.4).unwrap();
    let composition = |x, y| {
        geometry
            .unproject(plotters_ternary::TernaryCartesian::new(x, y), tolerance)
            .unwrap()
    };
    let mut svg = String::new();
    {
        let root = SVGBackend::with_string(&mut svg, (600, 500)).into_drawing_area();
        let mut chart = TernaryChartBuilder::on(&root)
            .margin(30)
            .viewport(viewport)
            .build()
            .unwrap();
        chart
            .draw_series(TernaryLineSeries::new(
                [
                    composition(0.45, 0.3),
                    composition(0.5, 0.6),
                    composition(0.55, 0.3),
                ],
                RGBColor(40, 50, 60).stroke_width(3),
            ))
            .unwrap();
        drop(chart);
        root.present().unwrap();
    }

    let paths = styled_polylines(&svg, "#28323C");
    assert_eq!(paths.len(), 2);
    let first = polyline_points(paths[0]);
    let second = polyline_points(paths[1]);
    assert_eq!(first.len(), 2);
    assert_eq!(second.len(), 2);
    assert!(first[0].0 < first[1].0);
    assert!(second[0].0 < second[1].0);
    assert!(first[1].0 < second[0].0);
}
#[test]
fn smooth_series_is_an_explicit_vector_path_without_generated_markers() {
    let mut svg = String::new();
    {
        let root = SVGBackend::with_string(&mut svg, (600, 500)).into_drawing_area();
        let mut chart = TernaryChartBuilder::on(&root).margin(30).build().unwrap();
        chart
            .draw_series(
                TernarySmoothSeries::new(
                    [
                        point(0.70, 0.20, 0.10),
                        point(0.40, 0.35, 0.25),
                        point(0.15, 0.45, 0.40),
                    ],
                    RGBColor(70, 80, 90).stroke_width(3),
                )
                .interpolation(TernaryInterpolation::Pchip)
                .samples_per_interval(4),
            )
            .unwrap()
            .label("Explicit smooth series");
        drop(chart);
        root.present().unwrap();
    }

    let paths = styled_polylines(&svg, "#46505A");
    assert_eq!(paths.len(), 1);
    assert_eq!(polyline_points(paths[0]).len(), 9);
    assert!(!svg.contains("<circle"));
    assert!(!svg.contains("<polygon"));
}
