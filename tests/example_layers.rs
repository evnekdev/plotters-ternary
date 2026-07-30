#![allow(dead_code)]

#[path = "../examples/common/mod.rs"]
mod common;
#[path = "../examples/output_support/mod.rs"]
mod output_support;

use common::{ExampleView, RenderPass, render};
use plotters::drawing::IntoDrawingArea;
use plotters::prelude::SVGBackend;

#[test]
fn example_geometry_layer_emits_no_text_and_text_layer_emits_expected_labels() {
    let mut geometry_svg = String::new();
    render(
        SVGBackend::with_string(&mut geometry_svg, (3_000, 2_400)).into_drawing_area(),
        ExampleView::Full,
        RenderPass::Geometry,
        3,
    )
    .unwrap();
    assert!(geometry_svg.contains("<polyline"));
    assert!(!geometry_svg.contains("<text"));
    assert!(!geometry_svg.contains("Full ternary diagram"));
    assert!(!geometry_svg.contains("Pure A corner"));

    let mut text_svg = String::new();
    render(
        SVGBackend::with_string(&mut text_svg, (1_000, 800)).into_drawing_area(),
        ExampleView::Full,
        RenderPass::Text,
        1,
    )
    .unwrap();
    assert!(text_svg.contains("Full ternary diagram"));
    assert!(text_svg.contains("Pure A corner"));
    assert!(text_svg.contains("Pure B corner"));
    assert!(text_svg.contains("Pure C corner"));
    assert!(text_svg.contains("Component A axis"));
}

#[test]
fn svg_is_grouped_vector_output_independent_of_bitmap_quality() {
    let svg = output_support::render_svg_string(
        (1_000, 800),
        |root| render(root, ExampleView::Full, RenderPass::Geometry, 1),
        |root| render(root, ExampleView::Full, RenderPass::Text, 1),
    )
    .unwrap();
    assert!(!svg.contains("<image"));
    assert!(svg.contains("<g id=\"ternary-geometry\""));
    assert!(svg.contains("shape-rendering=\"geometricPrecision\""));
    assert!(svg.contains("stroke-linecap=\"round\""));
    assert!(svg.contains("stroke-linejoin=\"round\""));
    assert!(svg.contains("<g id=\"ternary-text\">"));
    assert!(!svg.contains("text-rendering="));

    let geometry_start = svg.find("<g id=\"ternary-geometry\"").unwrap();
    let text_start = svg.find("<g id=\"ternary-text\">").unwrap();
    let geometry_group = &svg[geometry_start..text_start];
    assert!(geometry_group.contains("<polyline"));
    assert!(!geometry_group.contains("<text"));
}
