//! Publication-axis example with independent semantic A/B/C configurations.
mod legend_support;
mod output_support;

use crate::legend_support::{
    LEGEND_OUTER_PADDING, LEGEND_SYMBOL_LABEL_GAP, LEGEND_SYMBOL_SLOT_WIDTH, LEGEND_TEXT_SIZE,
    LegendRowLayout,
};
use crate::output_support::{
    BitmapQuality, BitmapRenderOptions, render_png, render_svg, reserve_final_caption_space, scaled,
};
use plotters::backend::DrawingBackend;
use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters::prelude::*;
use plotters_ternary::{
    AxisLabelFormat, AxisTextStyle, EndpointLabelPolicy, MarkerElement, MarkerShape, MarkerStyle,
    TernaryChart, TernaryChartBuilder, TernaryLineSeries, TernaryPoint, TernaryPointSeries,
    TickDirection, TickSpec,
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
        "examples/output/png/custom_axes.png",
        BitmapRenderOptions::new(OUTPUT_SIZE, QUALITY),
        |root, scale| render(root, Pass::Geometry, scale),
        |root| render(root, Pass::Text, 1),
    )?;
    render_svg(
        "examples/output/svg/custom_axes.svg",
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
    let caption = "Independent publication-quality ternary axes";
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
    let navy = RGBColor(25, 35, 50);
    let axis_text = AxisTextStyle::sans_serif(26, FontStyle::Bold, navy.to_rgba());
    let tick_text = AxisTextStyle::sans_serif(19, FontStyle::Normal, navy.to_rgba());
    let mut chart = builder.margin(scaled(78, scale)).build()?;
    let mesh = chart
        .configure_mesh()
        .boundary_style(navy.stroke_width(3))
        .corner_a_name("A")
        .corner_b_name("B")
        .corner_c_name("C")
        .corner_label_style(("sans-serif", 28, FontStyle::Bold, &navy))
        .corner_label_offset(scaled(26, scale))
        .axis_a(|axis| {
            axis.axis_name("A ? apex component")
                .axis_name_style(axis_text.clone())
                .axis_name_offset(scaled(42, scale))
                .major_ticks(TickSpec::Count(5))
                .minor_ticks(TickSpec::Step(0.05))
                .draw_major_grid(true)
                .draw_minor_grid(true)
                .major_grid_style(RGBColor(170, 190, 215).stroke_width(1))
                .minor_grid_style(RGBColor(225, 235, 245).stroke_width(1))
                .draw_ticks(true)
                .draw_tick_labels(true)
                .major_tick_length(10)
                .minor_tick_length(5)
                .major_tick_direction(TickDirection::Outward)
                .tick_label_style(tick_text.clone())
                .tick_label_offset(scaled(10, scale))
                .label_format(AxisLabelFormat::Percentage { precision: 0 })
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        })
        .axis_b(|axis| {
            axis.axis_name("B ? lower-left component")
                .axis_name_style(axis_text.clone())
                .axis_name_offset(scaled(42, scale))
                .major_ticks(TickSpec::Values(vec![0.0, 0.2, 0.5, 0.8, 1.0]))
                .minor_ticks(TickSpec::Step(0.1))
                .draw_major_grid(true)
                .draw_minor_grid(true)
                .major_grid_style(RGBColor(190, 180, 220).stroke_width(1))
                .minor_grid_style(RGBColor(235, 230, 245).stroke_width(1))
                .draw_ticks(true)
                .draw_tick_labels(true)
                .major_tick_length(10)
                .minor_tick_length(5)
                .tick_label_style(tick_text.clone())
                .label_formatter(|value| format!("B={value:.1}"))
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        })
        .axis_c(|axis| {
            axis.axis_name("C ? lower-right component")
                .axis_name_style(axis_text)
                .axis_name_offset(scaled(42, scale))
                .major_ticks(TickSpec::Step(0.25))
                .minor_ticks(TickSpec::Step(0.05))
                .draw_major_grid(true)
                .draw_minor_grid(true)
                .major_grid_style(RGBColor(180, 210, 185).stroke_width(1))
                .minor_grid_style(RGBColor(230, 242, 232).stroke_width(1))
                .draw_ticks(true)
                .draw_tick_labels(true)
                .major_tick_length(10)
                .minor_tick_length(5)
                .tick_label_style(tick_text)
                .label_format(AxisLabelFormat::Decimal { precision: 2 })
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        });
    if pass.geometry() {
        mesh.draw_geometry_scaled(scale)?;
    } else {
        mesh.draw_text()?;
    }
    let line = RGBColor(0, 95, 170)
        .mix(if pass.geometry() { 1.0 } else { 0.0 })
        .stroke_width(scaled(4, scale));
    chart
        .draw_series(TernaryLineSeries::new(
            [
                p(0.78, 0.12, 0.10),
                p(0.62, 0.20, 0.18),
                p(0.45, 0.34, 0.21),
                p(0.28, 0.47, 0.25),
            ],
            line,
        ))?
        .label("Liquidus trend")
        .legend(move |anchor| {
            let l = LegendRowLayout::from_plotters_anchor(anchor, scale);
            let (a, b) = l.line_endpoints();
            PathElement::new([a, b], line)
        });
    let marker = if pass.geometry() {
        MarkerStyle::solid(
            MarkerShape::Diamond,
            RGBColor(220, 75, 60),
            RGBColor(90, 30, 35).stroke_width(scaled(2, scale)),
        )?
    } else {
        MarkerStyle::solid(
            MarkerShape::Diamond,
            RGBColor(220, 75, 60).mix(0.0),
            RGBColor(90, 30, 35).mix(0.0).stroke_width(scaled(2, scale)),
        )?
    };
    chart
        .draw_series(
            TernaryPointSeries::new([
                p(0.58, 0.24, 0.18),
                p(0.43, 0.37, 0.20),
                p(0.31, 0.45, 0.24),
            ])
            .size(scaled(9, scale))
            .marker_style(marker.clone()),
        )?
        .label("Experimental analyses")
        .legend(move |anchor| {
            MarkerElement::new(
                LegendRowLayout::from_plotters_anchor(anchor, scale).custom_symbol(|centre| centre),
                scaled(8, scale),
                marker.clone(),
            )
            .expect("valid marker")
        });
    draw_legend(&mut chart, pass, scale)?;
    drop(chart);
    root.present()?;
    Ok(())
}
fn draw_legend<'a, DB: DrawingBackend + 'a>(
    chart: &mut TernaryChart<'a, DB>,
    pass: Pass,
    scale: u32,
) -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>> {
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(WHITE.mix(if pass.geometry() { 0.9 } else { 0.0 }))
        .border_style(
            BLACK
                .mix(if pass.geometry() { 1.0 } else { 0.0 })
                .stroke_width(scale),
        )
        .label_font((
            "sans-serif",
            scaled(LEGEND_TEXT_SIZE, scale),
            FontStyle::Normal,
            &BLACK.mix(if pass.text() { 1.0 } else { 0.0 }),
        ))
        .legend_area_size(scaled(
            LEGEND_SYMBOL_SLOT_WIDTH + LEGEND_SYMBOL_LABEL_GAP,
            scale,
        ))
        .margin(scaled(LEGEND_OUTER_PADDING, scale))
        .draw()?;
    Ok(())
}
const fn p(a: f64, b: f64, c: f64) -> TernaryPoint {
    TernaryPoint::new(a, b, c)
}
