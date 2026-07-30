#![allow(dead_code)]

#[path = "../examples/legend_support/mod.rs"]
mod legend_support;
#[path = "../examples/output_support/mod.rs"]
mod output_support;

use std::cell::RefCell;
use std::error::Error;
use std::path::PathBuf;

use image::RgbImage;
use plotters::prelude::*;
use plotters_ternary::{
    MarkerElement, MarkerPartition, MarkerShape, MarkerSlice, MarkerStyle, TernaryChartBuilder,
    TernaryPoint, TernaryPointSeries,
};

use legend_support::LegendRowLayout;
use output_support::{BitmapQuality, BitmapRenderOptions, render_png, render_svg_string, scaled};

fn phase_style() -> MarkerStyle {
    MarkerStyle::partitioned(
        MarkerShape::RoundedSquare { corner_ratio: 0.25 },
        MarkerPartition::Quadrants { rotation_deg: 20.0 },
        vec![
            MarkerSlice::new(1.0, RED),
            MarkerSlice::new(1.0, GREEN),
            MarkerSlice::new(1.0, BLUE),
            MarkerSlice::new(1.0, YELLOW),
        ],
        Some(WHITE.stroke_width(1)),
        Some(BLACK.stroke_width(2)),
    )
    .unwrap()
}

#[test]
fn marker_series_keeps_native_annotation_and_original_provider_indexes()
-> Result<(), Box<dyn Error>> {
    let mut svg = String::new();
    let root = SVGBackend::with_string(&mut svg, (520, 420)).into_drawing_area();
    root.fill(&WHITE)?;
    let mut chart = TernaryChartBuilder::on(&root).margin(25).build()?;
    chart.configure_mesh().major_step(0.2).draw()?;
    let seen = RefCell::new(Vec::new());
    chart
        .draw_series(
            TernaryPointSeries::new([
                TernaryPoint::new(0.2, 0.3, 0.5),
                TernaryPoint::new(0.4, 0.3, 0.3),
                TernaryPoint::new(0.6, 0.2, 0.2),
            ])
            .size(9)
            .marker_style(phase_style())
            .point_style_provider(|index, _composition| {
                seen.borrow_mut().push(index);
                phase_style()
            }),
        )?
        .label("Partitioned phase points")
        .legend(|anchor| {
            let layout = LegendRowLayout::from_plotters_anchor(anchor, 1);
            MarkerElement::new(layout.symbol_center(), 9, phase_style()).unwrap()
        });
    chart
        .draw_point_series(
            TernaryPointSeries::new([TernaryPoint::new(0.25, 0.55, 0.20)]).size(8),
            |anchor, size, _legacy_style| MarkerElement::new(anchor, size, phase_style()).unwrap(),
        )?
        .label("MarkerElement closure")
        .legend(|anchor| {
            let layout = LegendRowLayout::from_plotters_anchor(anchor, 1);
            MarkerElement::new(layout.symbol_center(), 8, phase_style()).unwrap()
        });
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .legend_area_size(46)
        .draw()?;
    drop(chart);
    root.present()?;
    drop(root);
    assert_eq!(seen.into_inner(), vec![0, 1, 2]);
    assert!(svg.contains("Partitioned phase points"));
    assert!(!svg.contains("<image"));
    assert!(svg.matches("<polygon").count() >= 12);
    Ok(())
}

#[test]
fn geometry_is_vector_only_and_legend_uses_the_shared_symbol_slot() -> Result<(), Box<dyn Error>> {
    let svg = render_svg_string(
        (240, 160),
        |root| {
            root.fill(&WHITE)?;
            root.draw(&MarkerElement::new((80, 80), 18, phase_style())?)?;
            root.present()?;
            Ok(())
        },
        |root| {
            root.draw(&Text::new(
                "final-resolution text",
                (20, 24),
                ("sans-serif", 18),
            ))?;
            root.present()?;
            Ok(())
        },
    )?;
    assert!(svg.contains("id=\"ternary-geometry\""));
    assert!(svg.contains("shape-rendering=\"geometricPrecision\""));
    assert!(svg.contains("id=\"ternary-text\""));
    assert!(!svg.contains("text-rendering="));
    assert!(!svg.contains("<image"));
    assert!(svg.matches("<polygon").count() >= 4);

    let rows = [(300, 72), (300, 104), (300, 136)];
    let layouts: Vec<_> = rows
        .into_iter()
        .map(|anchor| LegendRowLayout::from_plotters_anchor(anchor, 1))
        .collect();
    assert!(
        layouts
            .iter()
            .all(|layout| layout.symbol_center_x == layouts[0].symbol_center_x)
    );
    assert!(
        layouts
            .iter()
            .all(|layout| layout.label_start_x == layouts[0].label_start_x)
    );
    assert!(
        layouts
            .iter()
            .all(|layout| layout.symbol_slot_width == layouts[0].symbol_slot_width)
    );
    Ok(())
}

#[test]
fn native_and_supersampled_pngs_keep_the_same_final_marker_layout() -> Result<(), Box<dyn Error>> {
    let native = temporary_path("native");
    let supersampled = temporary_path("supersampled");
    render_marker_png(&native, BitmapQuality::Native)?;
    render_marker_png(&supersampled, BitmapQuality::Supersampled { factor: 3 })?;
    let native_image = image::open(&native)?.to_rgb8();
    let supersampled_image = image::open(&supersampled)?.to_rgb8();
    assert_eq!(native_image.dimensions(), (96, 96));
    assert_eq!(supersampled_image.dimensions(), (96, 96));
    let native_bounds = coloured_bounds(&native_image).unwrap();
    let supersampled_bounds = coloured_bounds(&supersampled_image).unwrap();
    for (left, right) in [
        (native_bounds.0, supersampled_bounds.0),
        (native_bounds.1, supersampled_bounds.1),
        (native_bounds.2, supersampled_bounds.2),
        (native_bounds.3, supersampled_bounds.3),
    ] {
        assert!(
            (i32::try_from(left)? - i32::try_from(right)?).abs() <= 2,
            "native={native_bounds:?}, supersampled={supersampled_bounds:?}"
        );
    }
    std::fs::remove_file(native)?;
    std::fs::remove_file(supersampled)?;
    Ok(())
}

fn render_marker_png(path: &PathBuf, quality: BitmapQuality) -> Result<(), Box<dyn Error>> {
    render_png(
        path,
        BitmapRenderOptions::new((96, 96), quality),
        |root, scale| {
            root.fill(&WHITE)?;
            root.draw(&MarkerElement::new(
                (scaled(48, scale) as i32, scaled(48, scale) as i32),
                scaled(14, scale),
                phase_style().scaled(scale),
            )?)?;
            root.present()?;
            Ok(())
        },
        |root| {
            root.present()?;
            Ok(())
        },
    )
}

fn coloured_bounds(image: &RgbImage) -> Option<(u32, u32, u32, u32)> {
    let mut bounds: Option<(u32, u32, u32, u32)> = None;
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel.0.iter().any(|channel| *channel < 240) {
            bounds = Some(match bounds {
                None => (x, y, x, y),
                Some((min_x, min_y, max_x, max_y)) => {
                    (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
                }
            });
        }
    }
    bounds
}

fn temporary_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "plotters-ternary-marker-{name}-{}.png",
        std::process::id()
    ))
}
