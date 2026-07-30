use plotters::prelude::*;
use plotters_ternary::{
    AnnotationClipMode, AxisTextStyle, HorizontalAnchor, PolygonError, SeriesError,
    TernaryChartBuilder, TernaryChartError, TernaryPoint, TernaryPolygon, TernaryText,
    TernaryViewport, TextAnchor, VerticalAnchor, prepare_polygon,
};

fn point(a: f64, b: f64, c: f64) -> TernaryPoint {
    TernaryPoint::new(a, b, c)
}

#[test]
fn polygon_and_text_series_keep_native_annotations_and_svg_primitives() {
    let viewport = TernaryViewport::new(0.35, 0.75, 0.12, 0.58).unwrap();
    let mut svg = String::new();
    {
        let root = SVGBackend::with_string(&mut svg, (600, 500)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = TernaryChartBuilder::on(&root)
            .margin(40)
            .viewport(viewport)
            .build()
            .unwrap();
        chart
            .draw_series(
                TernaryPolygon::new([
                    point(1.0, 0.0, 0.0),
                    point(0.0, 1.0, 0.0),
                    point(0.0, 0.0, 1.0),
                ])
                .fill_style(BLUE.mix(0.25).filled())
                .border_style(BLUE.stroke_width(2)),
            )
            .unwrap()
            .label("Clipped region")
            .legend(|(x, y)| Rectangle::new([(x, y - 5), (x + 12, y + 5)], BLUE.filled()));
        chart
            .draw_series(
                TernaryText::new(point(0.4, 0.3, 0.3), "Liquid + α")
                    .style(AxisTextStyle::sans_serif(
                        20,
                        FontStyle::Bold,
                        BLACK.to_rgba(),
                    ))
                    .anchor(TextAnchor::new(
                        HorizontalAnchor::Center,
                        VerticalAnchor::Center,
                    ))
                    .offset((7, -9)),
            )
            .unwrap();
        chart
            .draw_series(
                TernaryText::new(point(0.9, 0.05, 0.05), "must not appear")
                    .clip_mode(AnnotationClipMode::Anchor),
            )
            .unwrap();
        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.85))
            .border_style(BLACK)
            .draw()
            .unwrap();
        drop(chart);
        root.present().unwrap();
    }

    assert!(svg.contains("Clipped region"));
    assert!(svg.contains("Liquid + α"));
    assert!(!svg.contains("must not appear"));
    assert_eq!(svg.matches("<polygon").count(), 1);
    assert_eq!(svg.matches("<polyline").count(), 1);
    assert!(svg.contains("text-anchor=\"middle\""));
    assert!(svg.contains("<text"));
    assert!(!svg.contains("<image"));
}

#[test]
fn committed_region_examples_keep_geometry_and_text_in_separate_vector_groups() {
    let regions = std::fs::read_to_string("examples/output/svg/regions_annotations.svg").unwrap();
    let cropped = std::fs::read_to_string("examples/output/svg/cropped_regions.svg").unwrap();
    for svg in [&regions, &cropped] {
        let geometry = svg
            .split("<g id=\"ternary-geometry\"")
            .nth(1)
            .unwrap()
            .split("<g id=\"ternary-text\"")
            .next()
            .unwrap();
        assert!(!geometry.contains("<text"));
        assert!(geometry.contains("<polygon"));
        assert!(!svg.contains("<image"));
    }
    let alpha = char::from_u32(0x03b1).unwrap();
    let beta = char::from_u32(0x03b2).unwrap();
    assert!(regions.contains(&(alpha.to_string() + " phase")));
    assert!(regions.contains(&(beta.to_string() + " phase")));
    assert!(regions.contains(&(String::from("Liquid + ") + &alpha.to_string())));
    assert!(cropped.contains("Visible clipped region"));
    assert!(cropped.contains("Outside-anchor note"));
    assert!(!cropped.contains("Omitted anchor label"));
}

#[test]
fn polygon_validation_keeps_source_indexes_and_rejects_crossing_loops() {
    let geometry = plotters_ternary::TernaryGeometry::default();
    let viewport = TernaryViewport::full(geometry);
    let invalid = prepare_polygon(
        [
            point(0.2, 0.3, 0.5),
            point(f64::NAN, 0.0, 1.0),
            point(0.3, 0.3, 0.4),
        ],
        geometry,
        viewport,
        plotters_ternary::Normalization::RequireUnitSum,
        plotters_ternary::Tolerance::default(),
    );
    assert!(matches!(
        invalid,
        Err(PolygonError::InvalidPoint { index: 1, .. })
    ));

    let mut svg = String::new();
    let root = SVGBackend::with_string(&mut svg, (400, 300)).into_drawing_area();
    let mut chart = TernaryChartBuilder::on(&root).build().unwrap();
    let error = match chart.draw_series(TernaryPolygon::new([
        point(0.2, 0.7, 0.1),
        point(0.6, 0.1, 0.3),
        point(0.2, 0.1, 0.7),
        point(0.6, 0.3, 0.1),
    ])) {
        Err(error) => error,
        Ok(_) => panic!("self-intersecting polygon unexpectedly drew"),
    };
    assert!(matches!(
        error,
        TernaryChartError::Series(SeriesError::Polygon(PolygonError::SelfIntersection { .. }))
    ));
}
