#![allow(dead_code)]

#[path = "../examples/output_support/mod.rs"]
mod output_support;
#[path = "../examples/series_common/mod.rs"]
mod series_common;

use series_common::{LegendRowLayout, SeriesExample};

const SYMBOL_SLOT_WIDTH: i32 = 34;
const SYMBOL_LABEL_GAP: i32 = 12;
const OUTER_PADDING: i32 = 12;

fn geometry_group(svg: &str) -> &str {
    let Some(start) = svg.find("<g id=\"ternary-geometry\"") else {
        return svg;
    };
    let end = svg[start..].find("</g>").unwrap() + start;
    &svg[start..end]
}

fn elements_with<'a>(svg: &'a str, element: &str, color: &str) -> Vec<&'a str> {
    geometry_group(svg)
        .lines()
        .filter(|line| line.contains(element) && line.contains(color))
        .collect()
}

fn attribute_i32(element: &str, attribute: &str) -> i32 {
    let prefix = format!("{attribute}=\"");
    element
        .split_once(&prefix)
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .parse()
        .unwrap()
}

fn points(element: &str) -> Vec<(i32, i32)> {
    element
        .split_once("points=\"")
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .split_ascii_whitespace()
        .map(|point| {
            let (x, y) = point.split_once(',').unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect()
}

fn label_x(svg: &str, label: &str) -> i32 {
    let text_start = svg[..svg.find(label).unwrap()].rfind("<text ").unwrap();
    attribute_i32(&svg[text_start..], "x")
}

fn label_y(svg: &str, label: &str) -> i32 {
    let text_start = svg[..svg.find(label).unwrap()].rfind("<text ").unwrap();
    attribute_i32(&svg[text_start..], "y")
}

fn legend_background(svg: &str) -> &str {
    geometry_group(svg)
        .lines()
        .rfind(|line| line.contains("<rect") && line.contains("opacity=\"0.88\""))
        .unwrap()
}

#[test]
fn shared_row_layout_centres_every_symbol_and_scales_without_changing_final_layout() {
    let layout = LegendRowLayout::from_plotters_anchor((627, 115), 1);
    assert_eq!(layout.symbol_center(), (644, 116));
    assert_eq!(layout.symbol_slot_width, SYMBOL_SLOT_WIDTH as u32);
    assert_eq!(layout.label_start_x, 673);
    assert_eq!(layout.line_endpoints(), ((627, 116), (661, 116)));
    assert_eq!(layout.custom_symbol(|centre| centre), (644, 116));

    assert_eq!(LegendRowLayout::from_plotters_anchor((627, 115), 1), layout);
}

#[test]
fn series_legend_svg_has_common_centres_gaps_and_safe_padding() {
    let svg = series_common::render_svg_string_for_test(SeriesExample::LinesPointsLegend).unwrap();

    let liquidus = points(elements_with(&svg, "<polyline", "#005FAA").last().unwrap());
    let solvus = points(elements_with(&svg, "<polyline", "#14915F").last().unwrap());
    for line in [&liquidus, &solvus] {
        assert_eq!(line.len(), 2);
        assert_eq!((line[0].0 + line[1].0) / 2, 644);
        assert_eq!(line[1].0 - line[0].0, SYMBOL_SLOT_WIDTH);
        assert_eq!(line[0].1, line[1].1);
    }

    let circles = elements_with(&svg, "<circle", "#CD2D37");
    let circle = circles.last().unwrap();
    assert_eq!(attribute_i32(circle, "cx"), 644);
    let circle_y = attribute_i32(circle, "cy");

    let triangle = points(elements_with(&svg, "<polygon", "#EE9119").last().unwrap());
    let triangle_x_min = triangle.iter().map(|(x, _)| *x).min().unwrap();
    let triangle_x_max = triangle.iter().map(|(x, _)| *x).max().unwrap();
    let triangle_y_min = triangle.iter().map(|(_, y)| *y).min().unwrap();
    let triangle_y_max = triangle.iter().map(|(_, y)| *y).max().unwrap();
    assert_eq!((triangle_x_min + triangle_x_max) / 2, 644);
    let triangle_y = (triangle_y_min + triangle_y_max) / 2;

    let crosses = elements_with(&svg, "<line", "#7D41A5");
    let legend_crosses = &crosses[crosses.len() - 2..];
    let mut cross_x = Vec::new();
    let mut cross_y = Vec::new();
    for cross in legend_crosses {
        cross_x.extend([attribute_i32(cross, "x1"), attribute_i32(cross, "x2")]);
        cross_y.extend([attribute_i32(cross, "y1"), attribute_i32(cross, "y2")]);
    }
    assert_eq!(
        (cross_x.iter().min().unwrap() + cross_x.iter().max().unwrap()) / 2,
        644
    );
    let cross_y = (cross_y.iter().min().unwrap() + cross_y.iter().max().unwrap()) / 2;

    let row_centres = [liquidus[0].1, solvus[0].1, circle_y, triangle_y, cross_y];
    let row_spacings: Vec<_> = row_centres
        .windows(2)
        .map(|rows| rows[1] - rows[0])
        .collect();
    assert!(
        row_spacings
            .iter()
            .all(|spacing| (27..=28).contains(spacing))
    );
    let minimum_spacing = *row_spacings.iter().min().unwrap();
    let maximum_spacing = *row_spacings.iter().max().unwrap();
    assert!(maximum_spacing - minimum_spacing <= 1);

    let labels = [
        "Liquidus boundary",
        "PCHIP solvus boundary",
        "Measured samples",
        "Reference samples",
        "Calibration composition",
    ];
    let layouts: Vec<_> = row_centres
        .into_iter()
        .map(|row_center_y| LegendRowLayout::from_plotters_anchor((627, row_center_y), 1))
        .collect();
    for ((layout, label), row_center_y) in layouts.iter().zip(labels).zip(row_centres) {
        assert_eq!(layout.symbol_center_x, 644);
        assert_eq!(layout.symbol_slot_width, SYMBOL_SLOT_WIDTH as u32);
        assert_eq!(layout.label_start_x, 673);
        assert_eq!(label_x(&svg, label), layout.label_start_x);
        // The native SVG text pass expresses its 22 px legend text with a
        // baseline nine pixels above the same physical row centre.
        assert_eq!(row_center_y - label_y(&svg, label), 9);
        assert_eq!(
            layout.label_start_x - (layout.symbol_center_x + SYMBOL_SLOT_WIDTH / 2),
            SYMBOL_LABEL_GAP,
        );
    }

    let background = legend_background(&svg);
    let left = attribute_i32(background, "x");
    let top = attribute_i32(background, "y");
    let right = left + attribute_i32(background, "width");
    let bottom = top + attribute_i32(background, "height");
    assert_eq!(left + OUTER_PADDING, 627);
    assert!(top < row_centres[0] - 12);
    assert!(bottom > row_centres.last().unwrap() + 8);
    assert!(liquidus[0].0 >= left + OUTER_PADDING);
    assert!(liquidus[1].0 < 673);
    assert!(right - 673 >= OUTER_PADDING);
}

#[test]
fn supersampled_geometry_scales_to_the_same_final_legend_rows_as_text() {
    for scale in [2, 3, 4] {
        let svg =
            series_common::render_geometry_svg_for_test(SeriesExample::LinesPointsLegend, scale)
                .unwrap();
        let liquidus = points(elements_with(&svg, "<polyline", "#005FAA").last().unwrap());
        let solvus = points(elements_with(&svg, "<polyline", "#14915F").last().unwrap());
        let circles = elements_with(&svg, "<circle", "#CD2D37");
        let circle = circles.last().unwrap();
        let triangle = points(elements_with(&svg, "<polygon", "#EE9119").last().unwrap());
        let crosses = elements_with(&svg, "<line", "#7D41A5");
        let legend_crosses = &crosses[crosses.len() - 2..];

        let triangle_y = (triangle.iter().map(|(_, y)| *y).min().unwrap()
            + triangle.iter().map(|(_, y)| *y).max().unwrap())
            / 2;
        let cross_y = (legend_crosses
            .iter()
            .flat_map(|line| [attribute_i32(line, "y1"), attribute_i32(line, "y2")])
            .min()
            .unwrap()
            + legend_crosses
                .iter()
                .flat_map(|line| [attribute_i32(line, "y1"), attribute_i32(line, "y2")])
                .max()
                .unwrap())
            / 2;
        let high_resolution_rows = [
            liquidus[0].1,
            solvus[0].1,
            attribute_i32(circle, "cy"),
            triangle_y,
            cross_y,
        ];
        let final_rows = [116, 144, 171, 199, 226];
        for (high_resolution, final_row) in high_resolution_rows.into_iter().zip(final_rows) {
            assert_eq!(
                high_resolution / scale as i32,
                final_row,
                "scale {scale}: high row {high_resolution}, expected final row {final_row}",
            );
        }

        let final_slot_left = 627;
        let final_slot_right = 661;
        assert_eq!(
            liquidus[0].0 / scale as i32,
            final_slot_left,
            "scale {scale}: left endpoint {:?}",
            liquidus[0],
        );
        assert!((liquidus[1].0 / scale as i32 - final_slot_right).abs() <= 1);
        assert!((solvus[0].0 / scale as i32 - final_slot_left).abs() <= 1);
        assert!((solvus[1].0 / scale as i32 - final_slot_right).abs() <= 1);
    }
}

#[test]
fn cropped_svg_uses_the_same_slot_layout_as_the_full_legend() {
    let svg = series_common::render_svg_string_for_test(SeriesExample::CroppedCrossing).unwrap();
    let red = points(elements_with(&svg, "<polyline", "#D23732").last().unwrap());
    let blue = points(elements_with(&svg, "<polyline", "#0066CC").last().unwrap());
    let markers = elements_with(&svg, "<circle", "#199669");
    let marker = markers.last().unwrap();

    for line in [&red, &blue] {
        assert_eq!((line[0].0 + line[1].0) / 2, 253);
        assert_eq!(line[1].0 - line[0].0, SYMBOL_SLOT_WIDTH);
    }
    assert_eq!(attribute_i32(marker, "cx"), 253);
    let labels = [
        "Outside endpoints, visible crossing",
        "Exit and re-entry (two subpaths)",
        "Centre-clipped markers",
    ];
    let row_centres = [red[0].1, blue[0].1, attribute_i32(marker, "cy")];
    for (label, row_center_y) in labels.into_iter().zip(row_centres) {
        assert_eq!(label_x(&svg, label), 282);
        assert_eq!(row_center_y - label_y(&svg, label), 9);
    }
    assert_eq!(282 - (253 + SYMBOL_SLOT_WIDTH / 2), SYMBOL_LABEL_GAP);
}
