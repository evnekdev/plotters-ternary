#[allow(dead_code)]
#[path = "../examples/output_support/mod.rs"]
mod output_support;

use plotters::prelude::*;
use plotters_ternary::{
    AxisTextStyle, ContourColorBar, ContourLabelConfig, ContourLabelMode, ContourLabelPlacement,
    ContourLabelStyle, ContourLegendPolicy, ContourOptions, ContourSet, RegularTernaryScalarField,
    TernaryChartBuilder, TernaryContourSeries,
};

fn field() -> RegularTernaryScalarField {
    RegularTernaryScalarField::from_fn(12, |[a, b, c]| 2.0 * a - 3.0 * b + 5.0 * c).unwrap()
}
fn contours() -> ContourSet {
    ContourSet::compute(&field(), &[-1.0, 0.0, 1.0, 2.0], ContourOptions::linear()).unwrap()
}
fn map(t: f64) -> RGBAColor {
    RGBColor((230.0 * t) as u8, 45, (230.0 * (1.0 - t)) as u8).to_rgba()
}

#[test]
fn per_level_entries_use_native_plotters_legends_and_formatting() {
    let contours = contours();
    let mut svg = String::new();
    {
        let root = SVGBackend::with_string(&mut svg, (720, 560)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = TernaryChartBuilder::on(&root).margin(45).build().unwrap();
        chart
            .draw_series(
                TernaryContourSeries::new(&contours)
                    .color_map(-1.0, 2.0, 3, map)
                    .unwrap()
                    .legend_policy(ContourLegendPolicy::EveryNth(2))
                    .level_formatter(|value| format!("{value:.0} °C")),
            )
            .unwrap();
        chart.configure_series_labels().draw().unwrap();
        drop(chart);
        root.present().unwrap();
    }
    assert!(svg.contains("-1 °C"));
    assert!(svg.contains("1 °C"));
    assert!(!svg.contains("0 °C"));
    assert!(svg.contains("<polyline"));
}

#[test]
fn labels_and_colour_bar_are_native_svg_text_and_do_not_mutate_contours() {
    let contours = contours();
    let before = contours.clone();
    let svg = output_support::render_svg_string(
        (800, 620),
        |root| {
            root.fill(&WHITE)?;
            let mut chart = TernaryChartBuilder::on(&root).margin(55).build()?;
            chart
                .draw_series(TernaryContourSeries::new(&contours).color_map(-1.0, 2.0, 3, map)?)?;
            let bar = ContourColorBar::new(-1.0, 2.0, map)?
                .title("Temperature / °C")
                .formatter(|value| format!("{value:.0}"));
            chart.draw_contour_color_bar_geometry(&bar, 1)?;
            drop(chart);
            root.present()?;
            Ok(())
        },
        |root| {
            let chart = TernaryChartBuilder::on(&root).margin(55).build()?;
            let labels = ContourLabelConfig::new()
                .formatter(|value| format!("{value:.0} °C"))
                .style(ContourLabelStyle::new(AxisTextStyle::sans_serif(
                    17,
                    FontStyle::Bold,
                    BLACK.to_rgba(),
                )))
                .minimum_visible_length(40.0)
                .endpoint_clearance(5.0)
                .maximum_curvature_degrees(45.0);
            chart.draw_contour_labels(&contours, &labels)?;
            let bar = ContourColorBar::new(-1.0, 2.0, map)?
                .title("Temperature / °C")
                .formatter(|value| format!("{value:.0}"));
            chart.draw_contour_color_bar_text(&bar)?;
            drop(chart);
            root.present()?;
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(contours, before);
    assert!(svg.contains("Temperature / °C"));
    assert!(svg.contains("transform=\"rotate("));
    assert!(svg.contains("<text"));
    assert!(!svg.contains("<image"));
}

#[test]
fn curved_and_repeated_labels_emit_deterministic_vector_glyphs() {
    let contours = contours();
    let render = || {
        output_support::render_svg_string(
            (760, 580),
            |root| {
                root.fill(&WHITE)?;
                let mut chart = TernaryChartBuilder::on(&root).margin(50).build()?;
                chart.draw_series(
                    TernaryContourSeries::new(&contours).style(BLACK.stroke_width(2)),
                )?;
                drop(chart);
                root.present()?;
                Ok(())
            },
            |root| {
                let chart = TernaryChartBuilder::on(&root).margin(50).build()?;
                let labels = ContourLabelConfig::new()
                    .mode(ContourLabelMode::Curved)
                    .placement(ContourLabelPlacement::Repeated { spacing: 190.0 })
                    .formatter(|value| format!("L{value:.0}"))
                    .minimum_visible_length(35.0)
                    .endpoint_clearance(4.0)
                    .maximum_curvature_degrees(45.0);
                chart.draw_contour_labels(&contours, &labels)?;
                drop(chart);
                root.present()?;
                Ok(())
            },
        )
        .unwrap()
    };
    let first = render();
    let second = render();
    assert_eq!(first, second);
    assert!(first.matches("transform=\"rotate(").count() >= 3);
    assert!(!first.contains("<image"));
}
#[test]
fn committed_advanced_contour_svgs_are_vector_outputs_with_expected_text() {
    let cases = [
        (
            "contour_level_legend",
            "Heatmap-coloured contours with level legend",
            "25 °C",
            false,
        ),
        (
            "contour_color_bar",
            "Heatmap-coloured contours with continuous colour bar",
            "Scalar level",
            false,
        ),
        (
            "contour_labels",
            "Tangent-aligned contour labels",
            "25 °C",
            true,
        ),
        (
            "curved_contour_labels",
            "Curved labels following contour arc length",
            ">°</text>",
            true,
        ),
        (
            "cropped_contour_labels",
            "Collision-aware labels in a cropped viewport",
            "°C",
            true,
        ),
        (
            "manual_contour_labels",
            "Manual semantic contour-label anchors",
            "55 °C",
            true,
        ),
        (
            "repeated_contour_labels",
            "Repeated labels along long contour components",
            "°C",
            true,
        ),
    ];
    for (stem, caption, expected, rotated_label) in cases {
        let svg = std::fs::read_to_string(format!("examples/output/svg/{stem}.svg")).unwrap();
        assert!(svg.contains("<g id=\"ternary-geometry\""), "{stem}");
        assert!(
            svg.contains("shape-rendering=\"geometricPrecision\""),
            "{stem}"
        );
        assert!(svg.contains("<g id=\"ternary-text\">"), "{stem}");
        assert!(svg.contains("<polyline"), "{stem}");
        assert!(svg.contains("<text"), "{stem}");
        assert!(svg.contains(caption), "{stem}");
        assert!(svg.contains(expected), "{stem}");
        assert!(!svg.contains("<image"), "{stem}");
        if rotated_label {
            assert!(svg.matches("transform=\"rotate(").count() >= 3, "{stem}");
        }
    }
}
#[test]
fn horizontal_colour_bar_uses_vector_geometry_and_native_text() {
    use plotters_ternary::{ContourColorBarOrientation, ContourColorBarPosition};

    let svg = output_support::render_svg_string(
        (640, 480),
        |root| {
            root.fill(&WHITE)?;
            let chart = TernaryChartBuilder::on(&root).margin(45).build()?;
            let bar = ContourColorBar::new(-2.0, 2.0, map)?
                .orientation(ContourColorBarOrientation::Horizontal)
                .position(ContourColorBarPosition::LowerLeft)
                .title("ΔG / kJ mol⁻¹")
                .tick_values(vec![-2.0, 0.0, 2.0]);
            chart.draw_contour_color_bar_geometry(&bar, 1)?;
            drop(chart);
            root.present()?;
            Ok(())
        },
        |root| {
            let chart = TernaryChartBuilder::on(&root).margin(45).build()?;
            let bar = ContourColorBar::new(-2.0, 2.0, map)?
                .orientation(ContourColorBarOrientation::Horizontal)
                .position(ContourColorBarPosition::LowerLeft)
                .title("ΔG / kJ mol⁻¹")
                .tick_values(vec![-2.0, 0.0, 2.0]);
            chart.draw_contour_color_bar_text(&bar)?;
            drop(chart);
            root.present()?;
            Ok(())
        },
    )
    .unwrap();
    assert!(svg.contains("ΔG / kJ mol⁻¹"));
    assert!(svg.contains("<rect"));
    assert!(svg.contains("<text"));
    assert!(!svg.contains("<image"));
}
