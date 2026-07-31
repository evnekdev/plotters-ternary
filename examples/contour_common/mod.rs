#![allow(dead_code)]

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
    AxisLabelFormat, AxisTextStyle, BinaryExtrapolation, ContourOptions, ContourRegularization,
    ContourSet, CubicAlphaMethod, CubicAlphaOptions, EndpointLabelPolicy, MarkerShape,
    RegularTernaryScalarField, TernaryChart, TernaryChartBuilder, TernaryContourSeries,
    TernaryPointSeries, TernaryViewport, TickRangeMode, TickSpec,
};
use std::error::Error;

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
#[derive(Clone, Copy)]
pub enum ContourExample {
    Linear,
    CubicComparison,
    Cropped,
}
impl ContourExample {
    fn stem(self) -> &'static str {
        match self {
            Self::Linear => "linear_contours",
            Self::CubicComparison => "cubic_alpha_contours",
            Self::Cropped => "cropped_contours",
        }
    }
    fn caption(self) -> &'static str {
        match self {
            Self::Linear => "Regular-grid linear ternary contours",
            Self::CubicComparison => "Linear and cubic-alpha contour comparison",
            Self::Cropped => "Cubic-alpha contours clipped by an invisible viewport",
        }
    }
}

pub fn write_outputs(example: ContourExample) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all("examples/output/png")?;
    std::fs::create_dir_all("examples/output/svg")?;
    let png = format!("examples/output/png/{}.png", example.stem());
    render_png(
        &png,
        BitmapRenderOptions::new(OUTPUT_SIZE, QUALITY),
        |root, scale| render(root, example, Pass::Geometry, scale),
        |root| render(root, example, Pass::Text, 1),
    )?;
    let svg = format!("examples/output/svg/{}.svg", example.stem());
    render_svg(
        &svg,
        OUTPUT_SIZE,
        |root| render(root, example, Pass::Geometry, 1),
        |root| render(root, example, Pass::Text, 1),
    )?;
    if !matches!(example, ContourExample::Linear) {
        let field = sample_field(9)?;
        let set = cubic_set(&field, BinaryExtrapolation::Muggianu, true)?;
        if let Some(d) = set.diagnostics() {
            println!(
                "cubic diagnostics: cubic_edges={}, linear_fallback_edges={}, refined_triangles={}, maximum_depth_hits={}, projection_failures={}",
                d.cubic_edges,
                d.linear_fallback_edges,
                d.refined_triangles,
                d.maximum_depth_hits,
                d.projection_failures
            );
        }
    }
    println!("Wrote {png} and {svg}");
    Ok(())
}

fn render<DB>(
    root: DrawingArea<DB, Shift>,
    example: ContourExample,
    pass: Pass,
    scale: u32,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    if pass.geometry() {
        root.fill(&WHITE)?;
    }
    let caption = example.caption();
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
    let viewport = if matches!(example, ContourExample::Cropped) {
        Some(TernaryViewport::new(0.46, 0.88, 0.08, 0.55)?)
    } else {
        None
    };
    let builder = if let Some(viewport) = viewport {
        builder.viewport(viewport)
    } else {
        builder
    };
    let mut chart = builder
        .margin(scaled(if viewport.is_some() { 78 } else { 68 }, scale))
        .build()?;
    let mesh = draw_mesh(&mut chart, pass, scale, viewport.is_some())?;
    if pass.geometry() {
        mesh.draw_background_scaled(&mut chart, scale)?;
    } else {
        mesh.draw_text(&mut chart)?;
    }
    let field = sample_field(9)?;
    match example {
        ContourExample::Linear => draw_linear(&mut chart, &field, pass, scale)?,
        ContourExample::CubicComparison => draw_comparison(&mut chart, &field, pass, scale)?,
        ContourExample::Cropped => draw_cropped(&mut chart, &field, pass, scale)?,
    };
    draw_samples(&mut chart, &field, pass, scale)?;
    if pass.geometry() {
        mesh.draw_foreground_scaled(&mut chart, scale)?;
    }
    draw_legend(&mut chart, pass, scale, example)?;
    drop(chart);
    root.present()?;
    Ok(())
}

fn draw_mesh<'a, DB: DrawingBackend + 'a>(
    chart: &mut TernaryChart<'a, DB>,
    pass: Pass,
    scale: u32,
    cropped: bool,
) -> Result<plotters_ternary::TernaryMesh, plotters_ternary::TernaryChartError<DB::ErrorType>> {
    let navy = RGBColor(25, 35, 50);
    let name = AxisTextStyle::sans_serif(25, FontStyle::Bold, navy.to_rgba());
    let ticks = AxisTextStyle::sans_serif(17, FontStyle::Normal, navy.to_rgba());
    let mesh = chart
        .configure_mesh()
        .boundary_style(navy.stroke_width(scaled(3, scale)))
        .corner_a_name("A")
        .corner_b_name("B")
        .corner_c_name("C")
        .corner_label_style(("sans-serif", scaled(28, scale), FontStyle::Bold, &navy))
        .axis_a(|axis| {
            axis.axis_name("A component")
                .axis_name_style(name.clone())
                .major_ticks(TickSpec::Step(0.2))
                .minor_ticks(TickSpec::Step(0.1))
                .tick_range_mode(if cropped {
                    TickRangeMode::VisibleRange
                } else {
                    TickRangeMode::FullCompositionRange
                })
                .draw_major_grid(true)
                .draw_minor_grid(true)
                .draw_ticks(true)
                .draw_tick_labels(true)
                .tick_label_style(ticks.clone())
                .label_format(AxisLabelFormat::Percentage { precision: 0 })
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        })
        .axis_b(|axis| {
            axis.axis_name("B component")
                .axis_name_style(name.clone())
                .major_ticks(TickSpec::Step(0.2))
                .tick_range_mode(if cropped {
                    TickRangeMode::VisibleRange
                } else {
                    TickRangeMode::FullCompositionRange
                })
                .draw_major_grid(true)
                .draw_ticks(true)
                .draw_tick_labels(true)
                .tick_label_style(ticks.clone())
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        })
        .axis_c(|axis| {
            axis.axis_name("C component")
                .axis_name_style(name)
                .major_ticks(TickSpec::Step(0.2))
                .tick_range_mode(if cropped {
                    TickRangeMode::VisibleRange
                } else {
                    TickRangeMode::FullCompositionRange
                })
                .draw_major_grid(true)
                .draw_ticks(true)
                .draw_tick_labels(true)
                .tick_label_style(ticks)
                .endpoint_label_policy(EndpointLabelPolicy::AutoAvoidDuplicates);
        });
    let _ = pass;
    Ok(mesh.build())
}

fn draw_linear<'a, DB: DrawingBackend + 'a>(
    chart: &mut TernaryChart<'a, DB>,
    field: &RegularTernaryScalarField,
    pass: Pass,
    scale: u32,
) -> Result<(), Box<dyn Error>>
where
    DB::ErrorType: 'static,
{
    let contours =
        ContourSet::compute(field, &[0.035, 0.08, 0.16, 0.28], ContourOptions::linear())?;
    let color = RGBColor(35, 105, 180);
    let style = color
        .mix(if pass.geometry() { 1.0 } else { 0.0 })
        .stroke_width(scaled(3, scale));
    chart
        .draw_series(
            TernaryContourSeries::new(&contours).style_for_level(move |level| {
                style.stroke_width(scaled(if level < 0.1 { 4 } else { 3 }, scale))
            }),
        )?
        .label("Piecewise-linear contours")
        .legend(move |anchor| {
            let (start, end) =
                LegendRowLayout::from_plotters_anchor(anchor, scale).line_endpoints();
            PathElement::new([start, end], style)
        });
    Ok(())
}
fn draw_comparison<'a, DB: DrawingBackend + 'a>(
    chart: &mut TernaryChart<'a, DB>,
    field: &RegularTernaryScalarField,
    pass: Pass,
    scale: u32,
) -> Result<(), Box<dyn Error>>
where
    DB::ErrorType: 'static,
{
    let levels = [0.05, 0.10, 0.18];
    let linear = ContourSet::compute(field, &levels, ContourOptions::linear())?;
    let akima = cubic_custom(
        field,
        CubicAlphaMethod::Akima,
        BinaryExtrapolation::Muggianu,
        false,
    )?;
    let steffen = cubic_custom(
        field,
        CubicAlphaMethod::Steffen,
        BinaryExtrapolation::Kohler,
        true,
    )?;
    add_contours(
        chart,
        &linear,
        "Linear",
        RGBColor(110, 110, 120),
        2,
        pass,
        scale,
    )?;
    add_contours(
        chart,
        &akima,
        "Akima + Muggianu",
        RGBColor(220, 105, 35),
        3,
        pass,
        scale,
    )?;
    add_contours(
        chart,
        &steffen,
        "Steffen + Kohler, regularized",
        RGBColor(20, 105, 190),
        3,
        pass,
        scale,
    )?;
    Ok(())
}
fn draw_cropped<'a, DB: DrawingBackend + 'a>(
    chart: &mut TernaryChart<'a, DB>,
    field: &RegularTernaryScalarField,
    pass: Pass,
    scale: u32,
) -> Result<(), Box<dyn Error>>
where
    DB::ErrorType: 'static,
{
    let contours = cubic_custom(
        field,
        CubicAlphaMethod::Makima,
        BinaryExtrapolation::Kohler,
        true,
    )?;
    add_contours(
        chart,
        &contours,
        "MAKIMA cubic-alpha contours",
        RGBColor(155, 45, 125),
        4,
        pass,
        scale,
    )?;
    Ok(())
}
fn add_contours<'a, DB: DrawingBackend + 'a>(
    chart: &mut TernaryChart<'a, DB>,
    contours: &ContourSet,
    label: &'static str,
    color: RGBColor,
    width: u32,
    pass: Pass,
    scale: u32,
) -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>> {
    let style = color
        .mix(if pass.geometry() { 1.0 } else { 0.0 })
        .stroke_width(scaled(width, scale));
    chart
        .draw_series(TernaryContourSeries::new(contours).style(style))?
        .label(label)
        .legend(move |anchor| {
            let (start, end) =
                LegendRowLayout::from_plotters_anchor(anchor, scale).line_endpoints();
            PathElement::new([start, end], style)
        });
    Ok(())
}
fn draw_samples<'a, DB: DrawingBackend + 'a>(
    chart: &mut TernaryChart<'a, DB>,
    field: &RegularTernaryScalarField,
    pass: Pass,
    scale: u32,
) -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>> {
    if pass.geometry() {
        let points = (0..field.vertex_count())
            .filter(|index| index % 3 == 0)
            .map(|index| field.composition_at(index).expect("grid index"));
        chart.draw_series(
            TernaryPointSeries::new(points)
                .size(scaled(3, scale))
                .style(RGBColor(30, 40, 55).mix(0.55).filled())
                .marker(MarkerShape::Circle),
        )?;
    }
    Ok(())
}
fn draw_legend<'a, DB: DrawingBackend + 'a>(
    chart: &mut TernaryChart<'a, DB>,
    pass: Pass,
    scale: u32,
    example: ContourExample,
) -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>> {
    chart
        .configure_series_labels()
        .position(if matches!(example, ContourExample::Cropped) {
            SeriesLabelPosition::LowerRight
        } else {
            SeriesLabelPosition::UpperRight
        })
        .background_style(WHITE.mix(if pass.geometry() { 0.90 } else { 0.0 }))
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

fn sample_field(
    subdivisions: usize,
) -> Result<RegularTernaryScalarField, plotters_ternary::ContourError> {
    let count = (subdivisions + 1) * (subdivisions + 2) / 2;
    let blank = RegularTernaryScalarField::new(subdivisions, vec![0.0; count])?;
    let values = (0..count)
        .map(|index| {
            let [a, b, c] = blank.composition_at(index).expect("grid index");
            (a - 0.36).powi(2)
                + 0.85 * (b - 0.31).powi(2)
                + 1.15 * (c - 0.33).powi(2)
                + 0.035 * (b - c)
                + 0.025 * (3.0 * a * b * c).sin()
        })
        .collect();
    Ok(RegularTernaryScalarField::new(subdivisions, values)?)
}
fn cubic_set(
    field: &RegularTernaryScalarField,
    extrapolation: BinaryExtrapolation,
    regularized: bool,
) -> Result<ContourSet, plotters_ternary::ContourError> {
    cubic_custom(field, CubicAlphaMethod::Steffen, extrapolation, regularized)
}
fn cubic_custom(
    field: &RegularTernaryScalarField,
    method: CubicAlphaMethod,
    extrapolation: BinaryExtrapolation,
    regularized: bool,
) -> Result<ContourSet, plotters_ternary::ContourError> {
    let mut options = CubicAlphaOptions {
        method,
        extrapolation,
        ..CubicAlphaOptions::default()
    };
    options.regularization = regularized.then(|| ContourRegularization {
        spacing: 0.014,
        ..ContourRegularization::default()
    });
    ContourSet::compute(
        field,
        &[0.05, 0.10, 0.18],
        ContourOptions::cubic_alpha(options),
    )
}
