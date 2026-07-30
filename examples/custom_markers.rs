//! Permanent scientific-marker gallery generated directly for PNG and SVG.

mod legend_support;
mod output_support;

use std::error::Error;

use plotters::backend::DrawingBackend;
use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters::prelude::*;
use plotters_ternary::{
    MarkerElement, MarkerShape, MarkerSlice, MarkerStyle, SweepDirection, TernaryChart,
    TernaryChartBuilder, TernaryLineSeries, TernaryPoint, TernaryPointSeries,
};

use crate::legend_support::{
    LEGEND_OUTER_PADDING, LEGEND_SYMBOL_LABEL_GAP, LEGEND_SYMBOL_SLOT_WIDTH, LEGEND_TEXT_SIZE,
    LegendRowLayout,
};
use crate::output_support::{
    BitmapQuality, BitmapRenderOptions, render_png, render_svg, reserve_final_caption_space, scaled,
};

const OUTPUT_SIZE: (u32, u32) = (1_200, 960);
const BITMAP_QUALITY: BitmapQuality = BitmapQuality::Supersampled { factor: 3 };

#[derive(Clone, Copy)]
enum RenderPass {
    Geometry,
    Text,
}

impl RenderPass {
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
        "examples/output/png/custom_markers.png",
        BitmapRenderOptions::new(OUTPUT_SIZE, BITMAP_QUALITY),
        |root, scale| render(root, RenderPass::Geometry, scale),
        |root| render(root, RenderPass::Text, 1),
    )?;
    render_svg(
        "examples/output/svg/custom_markers.svg",
        OUTPUT_SIZE,
        |root| render(root, RenderPass::Geometry, 1),
        |root| render(root, RenderPass::Text, 1),
    )?;
    Ok(())
}

fn render<DB>(
    root: DrawingArea<DB, Shift>,
    pass: RenderPass,
    scale: u32,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    if pass.geometry() {
        root.fill(&WHITE)?;
    }
    let caption = "Scientific marker gallery: shapes, partitions, and phase combinations";
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
    let mut chart = builder.margin(scaled(54, scale)).build()?;
    let mesh = chart
        .configure_mesh()
        .major_step(0.1)
        .boundary_style(RGBColor(35, 45, 60).stroke_width(scaled(3, scale)))
        .major_grid_style(RGBColor(214, 220, 228).stroke_width(scale))
        .axis_a_name("Component A")
        .axis_b_name("Component B")
        .axis_c_name("Component C")
        .corner_a_name("A")
        .corner_b_name("B")
        .corner_c_name("C")
        .axis_name_style((
            "sans-serif",
            scaled(26, scale),
            FontStyle::Bold,
            &RGBColor(25, 35, 50),
        ))
        .corner_label_style((
            "sans-serif",
            scaled(28, scale),
            FontStyle::Bold,
            &RGBColor(25, 35, 50),
        ))
        .axis_label_offset(scaled(35, scale))
        .corner_label_offset(scaled(25, scale));
    let mesh = if pass.geometry() {
        mesh
    } else {
        mesh.hide_grid_lines().hide_triangle_boundary()
    };
    let mesh = if pass.text() {
        mesh
    } else {
        mesh.hide_axis_names().hide_corner_names()
    };
    mesh.draw()?;

    let marker_styles = gallery_styles()?;
    let marker_points = triangular_lattice(marker_styles.len());
    let fallback = marker_styles[0].clone();
    let styles_for_points = marker_styles.clone();
    let series_scale = scale;
    let series_pass = pass;
    chart
        .draw_series(
            TernaryPointSeries::new(marker_points)
                .size(scaled(10, scale))
                .marker_style(fallback)
                .point_style_provider(move |index, _composition| {
                    style_for_pass(
                        &styles_for_points[index % styles_for_points.len()],
                        series_pass,
                        series_scale,
                    )
                }),
        )?
        .label("Scientific marker library")
        .legend(legend_marker(
            pass,
            scale,
            MarkerStyle::quadrants(
                MarkerShape::RoundedSquare { corner_ratio: 0.24 },
                0.0,
                [RED, BLUE, GREEN, YELLOW],
                Some(BLACK.stroke_width(2)),
            )?,
        ));

    let phase_styles = phase_styles()?;
    let phase_fallback = phase_styles[0].clone();
    let phase_for_points = phase_styles.clone();
    let phase_pass = pass;
    chart
        .draw_series(
            TernaryPointSeries::new([
                composition(0.50, 0.20, 0.30),
                composition(0.41, 0.32, 0.27),
                composition(0.30, 0.42, 0.28),
                composition(0.22, 0.34, 0.44),
            ])
            .size(scaled(13, scale))
            .marker_style(phase_fallback)
            .point_style_provider(move |index, _| {
                style_for_pass(&phase_for_points[index], phase_pass, scale)
            }),
        )?
        .label("Four-phase experimental points")
        .legend(legend_marker(pass, scale, phase_styles[0].clone()));

    let boundary_style = RGBColor(105, 55, 160).stroke_width(scaled(3, scale));
    let text_boundary_style = RGBColor(105, 55, 160)
        .mix(if pass.geometry() { 1.0 } else { 0.0 })
        .stroke_width(scaled(3, scale));
    chart
        .draw_series(TernaryLineSeries::new(
            [
                composition(0.72, 0.16, 0.12),
                composition(0.55, 0.25, 0.20),
                composition(0.36, 0.38, 0.26),
                composition(0.17, 0.53, 0.30),
            ],
            if pass.geometry() {
                boundary_style
            } else {
                text_boundary_style
            },
        ))?
        .label("Reference phase boundary")
        .legend(move |anchor| {
            let layout = LegendRowLayout::from_plotters_anchor(anchor, scale);
            let (start, end) = layout.line_endpoints();
            PathElement::new(
                [start, end],
                if pass.geometry() {
                    boundary_style
                } else {
                    text_boundary_style
                },
            )
        });

    let empty_style =
        MarkerStyle::empty(MarkerShape::Circle, RGBColor(35, 45, 60).stroke_width(2))?;
    chart
        .draw_series(
            TernaryPointSeries::new([composition(0.45, 0.11, 0.44), composition(0.29, 0.15, 0.56)])
                .size(scaled(12, scale))
                .marker_style(style_for_pass(&empty_style, pass, scale)),
        )?
        .label("Empty contour markers")
        .legend(legend_marker(pass, scale, empty_style));

    draw_legend(&mut chart, pass, scale)?;
    drop(chart);
    root.present()?;
    Ok(())
}

fn gallery_styles() -> Result<Vec<MarkerStyle>, Box<dyn Error>> {
    let dark = RGBColor(35, 45, 60).stroke_width(2);
    Ok(vec![
        MarkerStyle::empty(MarkerShape::Circle, dark)?,
        MarkerStyle::solid(
            MarkerShape::Circle,
            RGBColor(210, 60, 62),
            RGBColor(85, 25, 30).stroke_width(2),
        )?,
        MarkerStyle::solid(
            MarkerShape::Ellipse { aspect_ratio: 1.65 },
            RGBColor(65, 135, 205),
            dark,
        )?,
        MarkerStyle::solid(MarkerShape::Square, RGBColor(75, 165, 105), dark)?,
        MarkerStyle::solid(
            MarkerShape::Rectangle { aspect_ratio: 1.5 },
            RGBColor(100, 165, 185),
            dark,
        )?,
        MarkerStyle::solid(
            MarkerShape::RoundedSquare { corner_ratio: 0.25 },
            RGBColor(180, 120, 190),
            dark,
        )?,
        MarkerStyle::solid(MarkerShape::Diamond, RGBColor(240, 155, 45), dark)?,
        MarkerStyle::solid(MarkerShape::TriangleUp, RGBColor(215, 75, 65), dark)?,
        MarkerStyle::solid(MarkerShape::TriangleDown, RGBColor(70, 145, 205), dark)?,
        MarkerStyle::solid(MarkerShape::TriangleLeft, RGBColor(70, 170, 115), dark)?,
        MarkerStyle::solid(MarkerShape::TriangleRight, RGBColor(235, 165, 50), dark)?,
        MarkerStyle::solid(MarkerShape::pentagon(), RGBColor(200, 90, 145), dark)?,
        MarkerStyle::fact_sage(MarkerShape::hexagon(), RGBColor(80, 105, 195))?,
        MarkerStyle::solid(MarkerShape::octagon(), RGBColor(110, 160, 80), dark)?,
        MarkerStyle::solid(MarkerShape::star4(), RGBColor(225, 110, 55), dark)?,
        MarkerStyle::solid(MarkerShape::star5(), RGBColor(150, 85, 185), dark)?,
        MarkerStyle::solid(MarkerShape::star6(), RGBColor(50, 150, 175), dark)?,
        MarkerStyle::solid(MarkerShape::star8(), RGBColor(210, 70, 100), dark)?,
        MarkerStyle::empty(MarkerShape::Plus, RGBColor(30, 115, 190).stroke_width(3))?,
        MarkerStyle::empty(MarkerShape::Cross, RGBColor(175, 65, 140).stroke_width(3))?,
        MarkerStyle::empty(
            MarkerShape::Asterisk { arms: 6 },
            RGBColor(45, 130, 85).stroke_width(2),
        )?,
        MarkerStyle::horizontal(MarkerShape::Circle, RED, BLUE, Some(dark))?,
        MarkerStyle::vertical(MarkerShape::Square, GREEN, YELLOW, Some(dark))?,
        MarkerStyle::diagonal_forward(
            MarkerShape::Diamond,
            RGBColor(235, 95, 50),
            RGBColor(65, 135, 210),
            Some(dark),
        )?,
        MarkerStyle::diagonal_backward(
            MarkerShape::RoundedSquare { corner_ratio: 0.2 },
            RGBColor(90, 170, 105),
            RGBColor(230, 160, 50),
            Some(dark),
        )?,
        MarkerStyle::equal_radial(MarkerShape::Circle, [RED, BLUE], Some(dark))?,
        MarkerStyle::equal_radial(MarkerShape::Circle, [RED, GREEN, BLUE], Some(dark))?,
        MarkerStyle::equal_radial(MarkerShape::Circle, [RED, GREEN, BLUE, YELLOW], Some(dark))?,
        MarkerStyle::weighted_radial(
            MarkerShape::Circle,
            90.0,
            SweepDirection::Clockwise,
            vec![
                MarkerSlice::new(1.0, RED),
                MarkerSlice::new(2.0, GREEN),
                MarkerSlice::new(3.0, BLUE),
            ],
            Some(WHITE.stroke_width(1)),
            Some(dark),
        )?,
        MarkerStyle::quadrants(
            MarkerShape::Diamond,
            20.0,
            [RED, GREEN, BLUE, YELLOW],
            Some(dark),
        )?,
    ])
}

fn phase_styles() -> Result<Vec<MarkerStyle>, Box<dyn Error>> {
    let edge = RGBColor(20, 30, 42).stroke_width(2);
    Ok(vec![
        MarkerStyle::quadrants(
            MarkerShape::Circle,
            0.0,
            [RED, GREEN, BLUE, YELLOW],
            Some(edge),
        )?,
        MarkerStyle::weighted_radial(
            MarkerShape::Circle,
            90.0,
            SweepDirection::Clockwise,
            vec![
                MarkerSlice::new(4.0, RED),
                MarkerSlice::new(2.0, GREEN),
                MarkerSlice::new(1.0, BLUE),
                MarkerSlice::new(1.0, YELLOW),
            ],
            Some(WHITE.stroke_width(1)),
            Some(edge),
        )?,
        MarkerStyle::quadrants(
            MarkerShape::RoundedSquare { corner_ratio: 0.28 },
            45.0,
            [BLUE, RED, YELLOW, GREEN],
            Some(edge),
        )?,
        MarkerStyle::equal_radial(MarkerShape::Diamond, [GREEN, YELLOW, RED, BLUE], Some(edge))?,
    ])
}

fn triangular_lattice(count: usize) -> Vec<TernaryPoint> {
    let mut points = Vec::with_capacity(count);
    for row in 0..8 {
        let c = 0.08 + 0.11 * f64::from(row);
        let entries = 8 - row;
        let total = 1.0 - c;
        for column in 0..entries {
            let fraction = if entries == 1 {
                0.5
            } else {
                0.06 + 0.88 * f64::from(column) / f64::from(entries - 1)
            };
            let a = total * fraction;
            points.push(composition(a, total - a, c));
            if points.len() == count {
                return points;
            }
        }
    }
    points
}

fn style_for_pass(style: &MarkerStyle, pass: RenderPass, scale: u32) -> MarkerStyle {
    if pass.geometry() {
        style.scaled(scale)
    } else {
        transparent_style(style)
    }
}

fn transparent_style(style: &MarkerStyle) -> MarkerStyle {
    let mut style = style.clone();
    if let Some(edge) = style.edge.as_mut() {
        edge.color = transparent(edge.color);
    }
    match &mut style.fill {
        plotters_ternary::MarkerFill::Empty => {}
        plotters_ternary::MarkerFill::Solid { color } => *color = transparent(*color),
        plotters_ternary::MarkerFill::Partitioned {
            slices, divider, ..
        } => {
            for slice in slices {
                slice.color = transparent(slice.color);
            }
            if let Some(divider) = divider {
                divider.color = transparent(divider.color);
            }
        }
    }
    style
}

const fn transparent(color: RGBAColor) -> RGBAColor {
    RGBAColor(color.0, color.1, color.2, 0.0)
}

fn legend_marker(
    pass: RenderPass,
    scale: u32,
    style: MarkerStyle,
) -> impl Fn((i32, i32)) -> MarkerElement<(i32, i32)> {
    move |anchor| {
        let layout = LegendRowLayout::from_plotters_anchor(anchor, scale);
        MarkerElement::new(
            layout.custom_symbol(|centre| centre),
            scaled(10, scale),
            style_for_pass(&style, pass, scale),
        )
        .expect("gallery legend marker style is valid")
    }
}

fn draw_legend<'series, DB>(
    chart: &mut TernaryChart<'series, DB>,
    pass: RenderPass,
    scale: u32,
) -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>>
where
    DB: DrawingBackend + 'series,
{
    let text_color = BLACK.mix(if pass.text() { 1.0 } else { 0.0 });
    let background = WHITE.mix(if pass.geometry() { 0.90 } else { 0.0 });
    let border = RGBColor(35, 45, 60)
        .mix(if pass.geometry() { 1.0 } else { 0.0 })
        .stroke_width(scale);
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperRight)
        .background_style(background)
        .border_style(border)
        .label_font((
            "sans-serif",
            scaled(LEGEND_TEXT_SIZE, scale),
            FontStyle::Normal,
            &text_color,
        ))
        .legend_area_size(scaled(
            LEGEND_SYMBOL_SLOT_WIDTH + LEGEND_SYMBOL_LABEL_GAP,
            scale,
        ))
        .margin(scaled(LEGEND_OUTER_PADDING, scale))
        .draw()?;
    Ok(())
}

const fn composition(a: f64, b: f64, c: f64) -> TernaryPoint {
    TernaryPoint::new(a, b, c)
}
