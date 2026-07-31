//! Phase-region polygons and composition-anchored scientific annotations.

mod legend_support;
mod output_support;

use std::error::Error;

use plotters::backend::DrawingBackend;
use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters::prelude::*;
use plotters_ternary::{
    AnnotationClipMode, AxisLabelFormat, AxisTextStyle, EndpointLabelPolicy, MarkerShape,
    MarkerStyle, TernaryChart, TernaryChartBuilder, TernaryLineSeries, TernaryPoint,
    TernaryPointSeries, TernaryPolygon, TernaryText, TextAnchor, TickDirection, TickSpec,
};

use crate::legend_support::{
    LEGEND_OUTER_PADDING, LEGEND_SYMBOL_LABEL_GAP, LEGEND_SYMBOL_SLOT_WIDTH, LEGEND_TEXT_SIZE,
    LegendRowLayout,
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
        "examples/output/png/regions_annotations.png",
        BitmapRenderOptions::new(OUTPUT_SIZE, QUALITY),
        |root, scale| render(root, Pass::Geometry, scale),
        |root| render(root, Pass::Text, 1),
    )?;
    render_svg(
        "examples/output/svg/regions_annotations.svg",
        OUTPUT_SIZE,
        |root| render(root, Pass::Geometry, 1),
        |root| render(root, Pass::Text, 1),
    )?;
    println!("Wrote phase-region PNG and SVG outputs");
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
    let caption = "Ternary phase regions and composition annotations";
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
    let navy = RGBColor(25, 35, 50);
    let axis_text = AxisTextStyle::sans_serif(25, FontStyle::Bold, navy.to_rgba());
    let tick_text = AxisTextStyle::sans_serif(18, FontStyle::Normal, navy.to_rgba());
    let mut chart = builder.margin(scaled(74, scale)).build()?;
    let mesh = chart
        .configure_mesh()
        .boundary_style(navy.stroke_width(scaled(3, scale)))
        .corner_a_name("A")
        .corner_b_name("B")
        .corner_c_name("C")
        .corner_label_style(("sans-serif", scaled(28, scale), FontStyle::Bold, &navy))
        .corner_label_offset(scaled(24, scale))
        .axis_a(|axis| {
            axis.axis_name("A component")
                .axis_name_style(axis_text.clone())
                .major_ticks(TickSpec::Step(0.2))
                .minor_ticks(TickSpec::Step(0.1))
                .draw_major_grid(true)
                .draw_minor_grid(true)
                .draw_ticks(true)
                .draw_tick_labels(true)
                .major_tick_length(9)
                .minor_tick_length(4)
                .major_tick_direction(TickDirection::Outward)
                .tick_label_style(tick_text.clone())
                .label_format(AxisLabelFormat::Percentage { precision: 0 })
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        })
        .axis_b(|axis| {
            axis.axis_name("B component")
                .axis_name_style(axis_text.clone())
                .major_ticks(TickSpec::Step(0.2))
                .draw_major_grid(true)
                .draw_ticks(true)
                .draw_tick_labels(true)
                .tick_label_style(tick_text.clone())
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        })
        .axis_c(|axis| {
            axis.axis_name("C component")
                .axis_name_style(axis_text)
                .major_ticks(TickSpec::Values(vec![0.0, 0.25, 0.5, 0.75, 1.0]))
                .draw_major_grid(true)
                .draw_ticks(true)
                .draw_tick_labels(true)
                .tick_label_style(tick_text)
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        });
    let mesh = mesh.build();
    if pass.geometry() {
        mesh.draw_background_scaled(&mut chart, scale)?;
    } else {
        mesh.draw_text(&mut chart)?;
    }

    let alpha_fill = RGBColor(226, 93, 92)
        .mix(if pass.geometry() { 0.30 } else { 0.0 })
        .filled();
    let beta_fill = RGBColor(76, 145, 205)
        .mix(if pass.geometry() { 0.28 } else { 0.0 })
        .filled();
    let liquid_fill = RGBColor(235, 182, 63)
        .mix(if pass.geometry() { 0.30 } else { 0.0 })
        .filled();
    let alpha_border = RGBColor(155, 45, 48)
        .mix(if pass.geometry() { 1.0 } else { 0.0 })
        .stroke_width(scaled(2, scale));
    let beta_border = RGBColor(35, 92, 155)
        .mix(if pass.geometry() { 1.0 } else { 0.0 })
        .stroke_width(scaled(2, scale));
    let liquid_border = RGBColor(165, 115, 20)
        .mix(if pass.geometry() { 1.0 } else { 0.0 })
        .stroke_width(scaled(2, scale));

    add_region(
        &mut chart,
        pass,
        scale,
        alpha_region(),
        alpha_fill,
        alpha_border,
        "α phase field",
    )?;
    add_region(
        &mut chart,
        pass,
        scale,
        beta_region(),
        beta_fill,
        beta_border,
        "β phase field",
    )?;
    add_region(
        &mut chart,
        pass,
        scale,
        liquid_region(),
        liquid_fill,
        liquid_border,
        "Liquid + α",
    )?;

    let boundary = RGBColor(95, 55, 160)
        .mix(if pass.geometry() { 1.0 } else { 0.0 })
        .stroke_width(scaled(3, scale));
    let line_annotation = chart.draw_series(TernaryLineSeries::new(
        [
            p(0.76, 0.12, 0.12),
            p(0.56, 0.25, 0.19),
            p(0.37, 0.39, 0.24),
            p(0.20, 0.50, 0.30),
        ],
        boundary,
    ))?;
    line_annotation
        .label("Measured boundary")
        .legend(move |anchor| {
            let (start, end) =
                LegendRowLayout::from_plotters_anchor(anchor, scale).line_endpoints();
            PathElement::new([start, end], boundary)
        });
    if pass.geometry() {
        let marker = MarkerStyle::quadrants(
            MarkerShape::Circle,
            0.0,
            [RED, GREEN, BLUE, YELLOW],
            Some(navy.stroke_width(scaled(2, scale))),
        )?
        .scaled(scale);
        chart.draw_series(
            TernaryPointSeries::new([
                p(0.54, 0.28, 0.18),
                p(0.40, 0.34, 0.26),
                p(0.29, 0.40, 0.31),
            ])
            .size(scaled(10, scale))
            .marker_style(marker),
        )?;
    }

    if pass.text() {
        let label = AxisTextStyle::sans_serif(22, FontStyle::Bold, navy.to_rgba());
        chart.draw_series(
            TernaryText::new(p(0.54, 0.30, 0.16), "α phase")
                .style(label.clone())
                .anchor(TextAnchor::center())
                .offset((0, -8)),
        )?;
        chart.draw_series(
            TernaryText::new(p(0.25, 0.20, 0.55), "β phase")
                .style(label.clone())
                .anchor(TextAnchor::center())
                .offset((4, 10)),
        )?;
        chart.draw_series(
            TernaryText::new(p(0.34, 0.49, 0.17), "Liquid + α")
                .style(label)
                .anchor(TextAnchor::center())
                .offset((0, 10))
                .clip_mode(AnnotationClipMode::Anchor),
        )?;
    }
    if pass.geometry() {
        mesh.draw_foreground_scaled(&mut chart, scale)?;
    }
    draw_legend(&mut chart, pass, scale)?;
    drop(chart);
    root.present()?;
    Ok(())
}

fn add_region<'a, DB>(
    chart: &mut TernaryChart<'a, DB>,
    pass: Pass,
    scale: u32,
    points: Vec<TernaryPoint>,
    fill: ShapeStyle,
    border: ShapeStyle,
    label: &'static str,
) -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>>
where
    DB: DrawingBackend + 'a,
{
    let annotation = if pass.geometry() {
        chart.draw_series(
            TernaryPolygon::new(points)
                .fill_style(fill)
                .border_style(border),
        )?
    } else {
        chart.draw_series(TernaryLineSeries::new(
            Vec::<TernaryPoint>::new(),
            BLACK.mix(0.0),
        ))?
    };
    annotation.label(label).legend(move |anchor| {
        LegendRowLayout::from_plotters_anchor(anchor, scale).custom_symbol(|centre| {
            let extent = scaled(8, scale) as i32;
            Rectangle::new(
                [
                    (centre.0 - extent, centre.1 - extent),
                    (centre.0 + extent, centre.1 + extent),
                ],
                fill,
            )
        })
    });
    Ok(())
}

fn draw_legend<'a, DB>(
    chart: &mut TernaryChart<'a, DB>,
    pass: Pass,
    scale: u32,
) -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>>
where
    DB: DrawingBackend + 'a,
{
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(WHITE.mix(if pass.geometry() { 0.90 } else { 0.0 }))
        .border_style(navy_style(pass, scale))
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

fn navy_style(pass: Pass, scale: u32) -> ShapeStyle {
    RGBColor(35, 45, 60)
        .mix(if pass.geometry() { 1.0 } else { 0.0 })
        .stroke_width(scale)
}

fn alpha_region() -> Vec<TernaryPoint> {
    vec![
        p(0.82, 0.10, 0.08),
        p(0.58, 0.30, 0.12),
        p(0.48, 0.26, 0.26),
        p(0.57, 0.10, 0.33),
        p(0.76, 0.08, 0.16),
    ]
}
fn beta_region() -> Vec<TernaryPoint> {
    vec![
        p(0.14, 0.70, 0.16),
        p(0.10, 0.22, 0.68),
        p(0.32, 0.18, 0.50),
        p(0.40, 0.34, 0.26),
        p(0.25, 0.50, 0.25),
    ]
}
fn liquid_region() -> Vec<TernaryPoint> {
    vec![
        p(0.40, 0.34, 0.26),
        p(0.25, 0.50, 0.25),
        p(0.16, 0.42, 0.42),
        p(0.24, 0.22, 0.54),
        p(0.48, 0.18, 0.34),
    ]
}
const fn p(a: f64, b: f64, c: f64) -> TernaryPoint {
    TernaryPoint::new(a, b, c)
}
