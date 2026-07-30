//! Cropped publication axes: only original triangle-edge fragments receive ticks.
mod output_support;

use crate::output_support::{
    BitmapQuality, BitmapRenderOptions, render_png, render_svg, reserve_final_caption_space, scaled,
};
use plotters::backend::DrawingBackend;
use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters::prelude::*;
use plotters_ternary::{
    AxisLabelFormat, AxisNamePosition, AxisTextStyle, CornerLabelVisibility, EndpointLabelPolicy,
    TernaryCartesian, TernaryChartBuilder, TernaryViewport, TickDirection, TickRangeMode, TickSpec,
};
use std::error::Error;
const OUTPUT_SIZE: (u32, u32) = (1000, 800);
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
        "examples/output/png/cropped_axes.png",
        BitmapRenderOptions::new(OUTPUT_SIZE, QUALITY),
        |root, scale| render(root, Pass::Geometry, scale),
        |root| render(root, Pass::Text, 1),
    )?;
    render_svg(
        "examples/output/svg/cropped_axes.svg",
        OUTPUT_SIZE,
        |root| render(root, Pass::Geometry, 1),
        |root| render(root, Pass::Text, 1),
    )?;
    Ok(())
}
fn render<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    pass: Pass,
    scale: u32,
) -> Result<(), Box<dyn Error>>
where
    DB::ErrorType: 'static,
{
    if pass.geometry() {
        root.fill(&WHITE)?;
    }
    let caption = "Cropped ternary axes: true-edge fragments only";
    let chart_root = if pass.geometry() {
        reserve_final_caption_space(&root, caption, 34, scale)?
    } else {
        root.clone()
    };
    let builder = TernaryChartBuilder::on(&chart_root);
    let builder = if pass.text() {
        builder.caption(caption, ("sans-serif", 34, FontStyle::Bold, &BLACK))
    } else {
        builder
    };
    let viewport = TernaryViewport::new(0.58, 1.02, -0.03, 0.65)?;
    let navy = RGBColor(25, 35, 50);
    let name_style = AxisTextStyle::sans_serif(26, FontStyle::Bold, navy.to_rgba());
    let tick_style = AxisTextStyle::sans_serif(19, FontStyle::Normal, navy.to_rgba());
    let mut chart = builder
        .margin(scaled(78, scale))
        .viewport(viewport)
        .build()?;
    let mesh = chart
        .configure_mesh()
        .boundary_style(navy.stroke_width(3))
        .corner_c_name("C")
        .corner_label_style(("sans-serif", 28, FontStyle::Bold, &navy))
        .corner_label_offset(scaled(26, scale))
        .corner_label_visibility(CornerLabelVisibility::Auto)
        .axis_a(|axis| {
            axis.axis_name("A axis (B\u{2013}C edge)")
                .axis_name_style(name_style.clone())
                .axis_name_offset(scaled(36, scale))
                .major_ticks(TickSpec::Step(0.1))
                .minor_ticks(TickSpec::Step(0.05))
                .tick_range_mode(TickRangeMode::VisibleRange)
                .draw_major_grid(true)
                .draw_minor_grid(true)
                .major_grid_style(RGBColor(170, 190, 215).stroke_width(1))
                .minor_grid_style(RGBColor(225, 235, 245).stroke_width(1))
                .draw_ticks(true)
                .draw_tick_labels(true)
                .major_tick_direction(TickDirection::Outward)
                .major_tick_length(10)
                .minor_tick_length(5)
                .tick_label_style(tick_style.clone())
                .label_format(AxisLabelFormat::Percentage { precision: 0 })
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        })
        .axis_b(|axis| {
            axis.axis_name("B axis (C\u{2013}A edge)")
                .axis_name_style(name_style.clone())
                .axis_name_offset(scaled(36, scale))
                .major_ticks(TickSpec::Values(vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]))
                .minor_ticks(TickSpec::Step(0.1))
                .tick_range_mode(TickRangeMode::VisibleRange)
                .draw_major_grid(true)
                .draw_minor_grid(true)
                .major_grid_style(RGBColor(190, 180, 220).stroke_width(1))
                .minor_grid_style(RGBColor(235, 230, 245).stroke_width(1))
                .draw_ticks(true)
                .draw_tick_labels(true)
                .major_tick_direction(TickDirection::Outward)
                .major_tick_length(10)
                .minor_tick_length(5)
                .tick_label_style(tick_style.clone())
                .label_formatter(|v| format!("B={v:.1}"))
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        })
        .axis_c(|axis| {
            axis.axis_name("C axis (manual)")
                .axis_name_style(name_style)
                .axis_name_position(AxisNamePosition::Logical(TernaryCartesian::new(0.86, 0.42)))
                .axis_name_offset(scaled(28, scale))
                .major_ticks(TickSpec::Step(0.1))
                .tick_range_mode(TickRangeMode::VisibleRange)
                .draw_major_grid(false)
                .draw_ticks(true)
                .draw_tick_labels(true)
                .tick_label_style(tick_style)
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        });
    if pass.geometry() {
        mesh.draw_geometry_scaled(scale)?;
    } else {
        mesh.draw_text()?;
    }
    drop(chart);
    root.present()?;
    Ok(())
}
