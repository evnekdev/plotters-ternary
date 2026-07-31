use plotters::prelude::*;
use plotters_ternary::{
    Component, ContourOptions, ContourSet, RegularTernaryScalarField, TernaryChartBuilder,
    TernaryContourSeries, TernaryGeometry, TernaryViewport, TriangleOrientation, VertexOrder,
};

fn linear_field(n: usize) -> RegularTernaryScalarField {
    let count = (n + 1) * (n + 2) / 2;
    let blank = RegularTernaryScalarField::new(n, vec![0.0; count]).unwrap();
    let values = (0..count)
        .map(|index| {
            let [a, b, c] = blank.composition_at(index).unwrap();
            2.0 * a - 3.0 * b + 5.0 * c
        })
        .collect();
    RegularTernaryScalarField::new(n, values).unwrap()
}

#[test]
fn contour_series_returns_native_annotation_and_uses_level_styles() {
    let field = linear_field(6);
    let contours = ContourSet::compute(&field, &[0.0, 2.0], ContourOptions::linear()).unwrap();
    let mut svg = String::new();
    {
        let root = SVGBackend::with_string(&mut svg, (620, 500)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = TernaryChartBuilder::on(&root).margin(35).build().unwrap();
        chart
            .draw_series(
                TernaryContourSeries::new(&contours).style_for_level(|level| {
                    if level < 1.0 {
                        RGBColor(11, 22, 33).stroke_width(3)
                    } else {
                        RGBColor(44, 55, 66).stroke_width(2)
                    }
                }),
            )
            .unwrap()
            .label("Native contour annotation")
            .legend(|(x, y)| {
                PathElement::new([(x, y), (x + 20, y)], RGBColor(11, 22, 33).stroke_width(3))
            });
        chart.configure_series_labels().draw().unwrap();
        drop(chart);
        root.present().unwrap();
    }
    assert!(svg.contains("Native contour annotation"));
    assert!(svg.contains("#0B1621"));
    assert!(svg.contains("#2C3742"));
    assert!(svg.contains("<polyline"));
    assert!(!svg.contains("<image"));
}

#[test]
fn complete_paths_are_unchanged_by_viewport_backend_and_render_settings() {
    let field = linear_field(8);
    let contours = ContourSet::compute(&field, &[0.5], ContourOptions::linear()).unwrap();
    let before = contours.clone();
    let viewport = TernaryViewport::new(0.4, 0.65, 0.15, 0.45).unwrap();
    let mut svg = String::new();
    {
        let root = SVGBackend::with_string(&mut svg, (600, 480)).into_drawing_area();
        let mut chart = TernaryChartBuilder::on(&root)
            .margin(30)
            .viewport(viewport)
            .build()
            .unwrap();
        chart
            .draw_series(TernaryContourSeries::new(&contours).style(RED.stroke_width(3)))
            .unwrap();
        drop(chart);
        root.present().unwrap();
    }
    assert_eq!(contours, before);
    assert!(svg.contains("#FF0000"));
    assert!(svg.contains("<polyline"));

    let mut bitmap = vec![255_u8; 520 * 420 * 3];
    {
        let root = BitMapBackend::with_buffer(&mut bitmap, (520, 420)).into_drawing_area();
        let mut chart = TernaryChartBuilder::on(&root).margin(20).build().unwrap();
        chart
            .draw_series(TernaryContourSeries::new(&contours).style(BLUE.stroke_width(1)))
            .unwrap();
        drop(chart);
        root.present().unwrap();
    }
    assert_eq!(contours, before);
}

#[test]
fn custom_vertex_order_and_downward_orientation_do_not_change_semantic_paths() {
    let field = linear_field(5);
    let contours =
        ContourSet::compute(&field, &[-1.0, 1.0, 3.0], ContourOptions::linear()).unwrap();
    let order = VertexOrder::new(Component::C, Component::A, Component::B).unwrap();
    let geometry = TernaryGeometry::new(TriangleOrientation::Down, order);
    let mut svg = String::new();
    {
        let root = SVGBackend::with_string(&mut svg, (620, 500)).into_drawing_area();
        let mut chart = TernaryChartBuilder::on(&root)
            .geometry(geometry)
            .viewport(TernaryViewport::full(geometry))
            .build()
            .unwrap();
        chart
            .draw_series(TernaryContourSeries::new(&contours).style(BLUE.stroke_width(2)))
            .unwrap();
        drop(chart);
        root.present().unwrap();
    }
    assert!(svg.matches("#0000FF").count() >= 3);
    assert!(!svg.contains("<image"));
}

#[test]
fn committed_contour_svgs_are_grouped_vector_outputs() {
    let cases = [
        (
            "linear_contours",
            "Regular-grid linear ternary contours",
            "Piecewise-linear contours",
        ),
        (
            "cubic_alpha_contours",
            "Linear and cubic-alpha contour comparison",
            "Steffen + Kohler, regularized",
        ),
        (
            "cropped_contours",
            "Cubic-alpha contours clipped by an invisible viewport",
            "MAKIMA cubic-alpha contours",
        ),
    ];
    for (stem, caption, legend) in cases {
        let svg = std::fs::read_to_string(format!("examples/output/svg/{stem}.svg")).unwrap();
        assert!(svg.contains("<g id=\"ternary-geometry\" shape-rendering=\"geometricPrecision\" stroke-linecap=\"round\" stroke-linejoin=\"round\">"));
        assert!(svg.contains("<g id=\"ternary-text\">"));
        assert!(svg.contains(caption));
        assert!(svg.contains(legend));
        assert!(svg.contains("<polyline"));
        assert!(svg.contains("<text"));
        assert!(!svg.contains("<image"));
        let geometry = svg
            .split("<g id=\"ternary-geometry\"")
            .nth(1)
            .unwrap()
            .split("<g id=\"ternary-text\">")
            .next()
            .unwrap();
        assert!(!geometry.contains("<text"));
    }
    let cropped = std::fs::read_to_string("examples/output/svg/cropped_contours.svg").unwrap();
    assert!(!cropped.contains("viewport-frame"));
}

#[cfg(feature = "cubic-alpha")]
#[test]
fn extrapolation_policy_is_independently_selected_for_contour_construction() {
    use plotters_ternary::{BinaryExtrapolation, ContourInterpolation, CubicAlphaOptions};
    let n = 5;
    let count = (n + 1) * (n + 2) / 2;
    let blank = RegularTernaryScalarField::new(n, vec![0.0; count]).unwrap();
    let values = (0..count)
        .map(|index| {
            let [a, b, c] = blank.composition_at(index).unwrap();
            a.powi(3) - 0.7 * b.powi(2) + 0.4 * c + 0.9 * a * b
        })
        .collect();
    let field = RegularTernaryScalarField::new(n, values).unwrap();
    let compute = |policy| {
        let options = CubicAlphaOptions {
            extrapolation: policy,
            regularization: None,
            ..CubicAlphaOptions::default()
        };
        ContourSet::compute(
            &field,
            &[0.15],
            plotters_ternary::ContourOptions {
                interpolation: ContourInterpolation::CubicAlpha(options),
                regularization: None,
                ..ContourOptions::linear()
            },
        )
        .unwrap()
    };
    let raw = compute(BinaryExtrapolation::RawBarycentric);
    let muggianu = compute(BinaryExtrapolation::Muggianu);
    let kohler = compute(BinaryExtrapolation::Kohler);
    assert_ne!(raw, muggianu);
    assert_ne!(muggianu, kohler);
    assert_eq!(compute(BinaryExtrapolation::RawBarycentric), raw);
}
