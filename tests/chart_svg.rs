use plotters::prelude::*;
use plotters_ternary::{TernaryChartBuilder, TernaryGeometry, TernaryViewport};

fn render(viewport: TernaryViewport, caption: &str) -> String {
    let mut svg = String::new();
    {
        let root = SVGBackend::with_string(&mut svg, (600, 480)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = TernaryChartBuilder::on(&root)
            .caption(caption, ("sans-serif", 24))
            .margin(35)
            .viewport(viewport)
            .build()
            .unwrap();
        chart
            .configure_mesh()
            .major_step(0.25)
            .boundary_style(RGBColor(1, 2, 3).stroke_width(3))
            .major_grid_style(RGBColor(170, 187, 204))
            .axis_a_name("Axis A")
            .axis_b_name("Axis B")
            .axis_c_name("Axis C")
            .corner_a_name("Corner A")
            .corner_b_name("Corner B")
            .corner_c_name("Corner C")
            .draw()
            .unwrap();
        drop(chart);
        root.present().unwrap();
    }
    svg
}

#[test]
fn full_svg_is_vector_only_and_contains_boundary_grid_caption_and_names() {
    let geometry = TernaryGeometry::default();
    let svg = render(TernaryViewport::full(geometry), "Structural full chart");
    assert!(!svg.contains("<image"));
    assert!(svg.contains("Structural full chart"));
    assert!(svg.contains("Axis A"));
    assert!(svg.contains("Corner C"));
    assert!(svg.contains("<polyline"));
    assert!(svg.contains("#010203"));
    assert!(svg.matches("<rect").count() > 1);
    assert!(svg.contains("width=\"1\" height=\"1\""));
}

#[test]
fn interior_svg_has_grid_but_no_boundary_names_or_viewport_frame() {
    let svg = render(
        TernaryViewport::new(0.30, 0.70, 0.15, 0.35).unwrap(),
        "Structural interior chart",
    );
    assert!(!svg.contains("<image"));
    assert!(svg.contains("Structural interior chart"));
    assert!(svg.contains("<polyline"));
    assert!(svg.contains("#AABBCC"));
    assert!(!svg.contains("#010203"));
    assert!(!svg.contains("Axis A"));
    assert!(!svg.contains("Corner A"));
    assert_eq!(svg.matches("<rect").count(), 1);
}
