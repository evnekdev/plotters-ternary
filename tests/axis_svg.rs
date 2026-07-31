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
fn sloping_axis_names_are_searchable_native_svg_text() {
    let custom = fs::read_to_string("examples/output/svg/custom_axes.svg")
        .expect("committed custom-axis SVG reference output");
    let cropped = fs::read_to_string("examples/output/svg/cropped_axes.svg")
        .expect("committed cropped-axis SVG reference output");

    for (svg, labels) in [
        (
            &custom,
            [
                "B \u{2192} lower-left component",
                "C \u{2192} lower-right component",
            ],
        ),
        (&cropped, ["B axis", "C axis (manual)"]),
    ] {
        let text_group = svg.find("<g id=\"ternary-text\">").unwrap();
        let geometry_group = &svg[..text_group];
        assert!(!geometry_group.contains("<text"));
        for label in labels {
            let text_start = svg.find(label).expect("native UTF-8 axis label text");
            let node_start = svg[..text_start].rfind("<text ").unwrap();
            let node = &svg[node_start..text_start];
            assert!(node.contains("transform=\"rotate("));
            assert!(node.contains("text-anchor=\"middle\""));
            assert!(node.contains("dominant-baseline=\"middle\""));
            assert!(node.contains("font-family=\"sans-serif\""));
            assert!(node.contains("font-size=\"26\""));
            assert!(node.contains("font-weight=\"bold\""));
        }
    }

    assert!(!custom.contains("<image"));
    assert!(!cropped.contains("<image"));
    // A horizontal A-axis name remains Plotters-native text without a rotation.
    assert!(custom.contains("A \u{2192} apex component"));
}

#[test]
fn cropped_axis_reference_does_not_draw_a_viewport_frame() {
    let svg = fs::read_to_string("examples/output/svg/cropped_axes.svg")
        .expect("committed cropped SVG reference output");
    assert!(svg.contains("Cropped ternary axes"));
    assert!(!svg.contains("viewport-frame"));
}
