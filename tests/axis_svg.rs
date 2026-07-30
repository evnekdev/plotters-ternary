use std::fs;

#[test]
fn publication_axis_examples_keep_vector_geometry_and_native_text_groups() {
    for stem in ["custom_axes", "cropped_axes"] {
        let path = format!("examples/output/svg/{stem}.svg");
        let svg = fs::read_to_string(path).expect("committed SVG reference output");
        assert!(svg.contains("id=\"ternary-geometry\""));
        assert!(svg.contains("shape-rendering=\"geometricPrecision\""));
        assert!(svg.contains("id=\"ternary-text\""));
        assert!(!svg.contains("<image"));
        assert!(svg.contains("<text"));
    }
}

#[test]
fn cropped_axis_reference_does_not_draw_a_viewport_frame() {
    let svg = fs::read_to_string("examples/output/svg/cropped_axes.svg")
        .expect("committed cropped SVG reference output");
    assert!(svg.contains("Cropped ternary axes"));
    // Sloped axis names use the documented vector-glyph rotation fallback;
    // their source string is therefore intentionally not asserted as SVG text.
    // The SVG output helper only emits Plotters geometry and text groups; the
    // viewport itself has no rectangle element or diagnostic id.
    assert!(!svg.contains("viewport-frame"));
}
