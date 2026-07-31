//! Cropped phase regions prove polygon clipping and annotation policies.

mod output_support;

use std::error::Error;

use plotters::backend::DrawingBackend;
use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters::prelude::*;
use plotters_ternary::{
    AnnotationClipMode, AxisLabelFormat, AxisNamePosition, AxisTextStyle, CornerLabelVisibility,
    EndpointLabelPolicy, TernaryCartesian, TernaryChartBuilder, TernaryPoint, TernaryPolygon,
    TernaryText, TernaryViewport, TextAnchor, TickDirection, TickRangeMode, TickSpec,
};

use crate::output_support::{
    BitmapQuality, BitmapRenderOptions, render_png, render_svg, reserve_final_caption_space, scaled,
};

const OUTPUT_SIZE: (u32, u32) = (1_000, 800);
const QUALITY: BitmapQuality = BitmapQuality::Supersampled { factor: 3 };

#[derive(Clone, Copy)]
enum Pass {
    Geometry,
    Text,
}
impl Pass {
    const fn geometry(self) -> bool {
        matches!(self, Self::Geometry)
    }
    const fn text(self) -> bool {
        matches!(self, Self::Text)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all("examples/output/png")?;
    std::fs::create_dir_all("examples/output/svg")?;
    render_png(
        "examples/output/png/cropped_regions.png",
        BitmapRenderOptions::new(OUTPUT_SIZE, QUALITY),
        |root, scale| render(root, Pass::Geometry, scale),
        |root| render(root, Pass::Text, 1),
    )?;
    render_svg(
        "examples/output/svg/cropped_regions.svg",
        OUTPUT_SIZE,
        |root| render(root, Pass::Geometry, 1),
        |root| render(root, Pass::Text, 1),
    )?;
    println!("Wrote cropped phase-region PNG and SVG outputs");
    Ok(())
}

fn render<DB>(root: DrawingArea<DB, Shift>, pass: Pass, scale: u32) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    if pass.geometry() {
        root.fill(&WHITE)?;
    }
    let caption = "Cropped phase regions: mathematical viewport clipping";
    let chart_root = if pass.geometry() {
        reserve_final_caption_space(&root, caption, 32, scale)?
    } else {
        root.clone()
    };
    let builder = TernaryChartBuilder::on(&chart_root);
    let builder = if pass.text() {
        builder.caption(caption, ("sans-serif", 32, FontStyle::Bold, &BLACK))
    } else {
        builder
    };
    let viewport = TernaryViewport::new(0.45, 0.85, 0.12, 0.55)?;
    let navy = RGBColor(25, 35, 50);
    let name = AxisTextStyle::sans_serif(25, FontStyle::Bold, navy.to_rgba());
    let ticks = AxisTextStyle::sans_serif(18, FontStyle::Normal, navy.to_rgba());
    let mut chart = builder
        .margin(scaled(76, scale))
        .viewport(viewport)
        .build()?;
    let mesh = chart
        .configure_mesh()
        .boundary_style(navy.stroke_width(scaled(3, scale)))
        .corner_label_visibility(CornerLabelVisibility::Auto)
        .corner_c_name("C")
        .corner_label_style(("sans-serif", scaled(28, scale), FontStyle::Bold, &navy))
        .axis_a(|axis| {
            axis.axis_name("A (B–C edge)")
                .axis_name_style(name.clone())
                .major_ticks(TickSpec::Step(0.1))
                .minor_ticks(TickSpec::Step(0.05))
                .tick_range_mode(TickRangeMode::VisibleRange)
                .draw_major_grid(true)
                .draw_minor_grid(true)
                .draw_ticks(true)
                .draw_tick_labels(true)
                .major_tick_direction(TickDirection::Outward)
                .major_tick_length(9)
                .minor_tick_length(4)
                .tick_label_style(ticks.clone())
                .label_format(AxisLabelFormat::Percentage { precision: 0 })
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        })
        .axis_b(|axis| {
            axis.axis_name("B component")
                .axis_name_style(name.clone())
                .major_ticks(TickSpec::Step(0.1))
                .tick_range_mode(TickRangeMode::VisibleRange)
                .draw_major_grid(true)
                .draw_ticks(true)
                .draw_tick_labels(true)
                .tick_label_style(ticks.clone())
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        })
        .axis_c(|axis| {
            axis.axis_name("C, manual")
                .axis_name_style(name)
                .axis_name_position(AxisNamePosition::Logical(TernaryCartesian::new(0.75, 0.47)))
                .axis_name_offset(scaled(22, scale))
                .major_ticks(TickSpec::Step(0.1))
                .tick_range_mode(TickRangeMode::VisibleRange)
                .draw_major_grid(true)
                .draw_ticks(true)
                .draw_tick_labels(true)
                .tick_label_style(ticks)
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        });
    let mesh = mesh.build();
    if pass.geometry() {
        mesh.draw_background_scaled(&mut chart, scale)?;
    } else {
        mesh.draw_text(&mut chart)?;
    }

    if pass.geometry() {
        // The source triangle has no vertex inside this viewport, but it still
        // covers a visible clipped area and exercises all four clip sides.
        chart.draw_series(
            TernaryPolygon::new([p(1.0, 0.0, 0.0), p(0.0, 1.0, 0.0), p(0.0, 0.0, 1.0)])
                .fill_style(RGBColor(90, 145, 210).mix(0.23).filled())
                .border_style(RGBColor(30, 85, 150).stroke_width(scaled(2, scale))),
        )?;
        chart.draw_series(
            TernaryPolygon::new([
                p(0.72, 0.23, 0.05),
                p(0.60, 0.02, 0.38),
                p(0.28, 0.18, 0.54),
                p(0.22, 0.46, 0.32),
                p(0.52, 0.41, 0.07),
            ])
            .fill_style(RGBColor(225, 105, 75).mix(0.36).filled())
            .border_style(RGBColor(150, 48, 35).stroke_width(scaled(3, scale))),
        )?;
    }
    if pass.text() {
        let label = AxisTextStyle::sans_serif(23, FontStyle::Bold, navy.to_rgba());
        chart.draw_series(
            TernaryText::new(p(0.47, 0.25, 0.28), "Visible clipped region")
                .style(label.clone())
                .anchor(TextAnchor::center())
                .offset((8, -9))
                .clip_mode(AnnotationClipMode::Anchor),
        )?;
        chart.draw_series(
            TernaryText::new(p(0.88, 0.06, 0.06), "Omitted anchor label")
                .style(label.clone())
                .clip_mode(AnnotationClipMode::Anchor),
        )?;
        chart.draw_series(
            TernaryText::new(p(0.30, 0.45, 0.25), "Outside-anchor note")
                .style(label)
                .offset((12, -8))
                .clip_mode(AnnotationClipMode::None),
        )?;
    }
    if pass.geometry() {
        mesh.draw_foreground_scaled(&mut chart, scale)?;
    }
    drop(chart);
    root.present()?;
    Ok(())
}

const fn p(a: f64, b: f64, c: f64) -> TernaryPoint {
    TernaryPoint::new(a, b, c)
}
