use std::error::Error;

use plotters::backend::DrawingBackend;
use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters::prelude::*;
use plotters_ternary::{
    MarkerShape, Normalization, TernaryChartBuilder, TernaryInterpolation, TernaryLineSeries,
    TernaryPoint, TernaryPointSeries, TernarySmoothSeries, TernaryViewport,
};

use crate::output_support::{BitmapQuality, BitmapRenderOptions, render_png, render_svg, scaled};

const OUTPUT_SIZE: (u32, u32) = (1_000, 800);
const BITMAP_QUALITY: BitmapQuality = BitmapQuality::Supersampled { factor: 3 };

#[derive(Clone, Copy)]
enum RenderPass {
    Geometry,
    Text,
}

impl RenderPass {
    const fn draws_geometry(self) -> bool {
        matches!(self, Self::Geometry)
    }

    const fn draws_text(self) -> bool {
        matches!(self, Self::Text)
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum SeriesExample {
    LinesPointsLegend,
    CroppedCrossing,
}

impl SeriesExample {
    fn stem(self) -> &'static str {
        match self {
            Self::LinesPointsLegend => "lines_points_legend",
            Self::CroppedCrossing => "cropped_crossing_series",
        }
    }
}

pub fn write_outputs(example: SeriesExample) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all("examples/output/png")?;
    std::fs::create_dir_all("examples/output/svg")?;

    let png_path = format!("examples/output/png/{}.png", example.stem());
    render_png(
        &png_path,
        BitmapRenderOptions::new(OUTPUT_SIZE, BITMAP_QUALITY),
        |root, scale| render(root, example, RenderPass::Geometry, scale),
        |root| render(root, example, RenderPass::Text, 1),
    )?;

    let svg_path = format!("examples/output/svg/{}.svg", example.stem());
    render_svg(
        &svg_path,
        OUTPUT_SIZE,
        |root| render(root, example, RenderPass::Geometry, 1),
        |root| render(root, example, RenderPass::Text, 1),
    )?;

    println!("Wrote {png_path} and {svg_path}");
    Ok(())
}

fn render<DB>(
    root: DrawingArea<DB, Shift>,
    example: SeriesExample,
    pass: RenderPass,
    scale: u32,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    if pass.draws_geometry() {
        root.fill(&WHITE)?;
    }
    match example {
        SeriesExample::LinesPointsLegend => render_full(root, pass, scale)?,
        SeriesExample::CroppedCrossing => render_cropped(root, pass, scale)?,
    }
    Ok(())
}

fn render_full<DB>(
    root: DrawingArea<DB, Shift>,
    pass: RenderPass,
    scale: u32,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    let caption_color = BLACK.mix(if pass.draws_text() { 1.0 } else { 0.0 });
    let mut chart = TernaryChartBuilder::on(&root)
        .caption(
            "Ternary lines, points and Plotters legends",
            (
                "sans-serif",
                scaled(32, scale),
                FontStyle::Bold,
                &caption_color,
            ),
        )
        .margin(scaled(55, scale))
        .build()?;
    let mesh = chart
        .configure_mesh()
        .major_step(0.1)
        .boundary_style(RGBColor(35, 45, 60).stroke_width(scaled(3, scale)))
        .major_grid_style(RGBColor(205, 211, 220).stroke_width(scale))
        .axis_a_name("Component A")
        .axis_b_name("Component B")
        .axis_c_name("Component C")
        .corner_a_name("Pure A")
        .corner_b_name("Pure B")
        .corner_c_name("Pure C")
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
        .axis_label_offset(scaled(34, scale))
        .corner_label_offset(scaled(24, scale));
    let mesh = if pass.draws_geometry() {
        mesh
    } else {
        mesh.hide_grid_lines().hide_triangle_boundary()
    };
    let mesh = if pass.draws_text() {
        mesh
    } else {
        mesh.hide_axis_names().hide_corner_names()
    };
    mesh.draw()?;

    let liquidus_style = stroke_style(RGBColor(0, 95, 170), 4, pass, scale);
    chart
        .draw_series(TernaryLineSeries::new(
            [
                composition(0.78, 0.12, 0.10),
                composition(0.63, 0.22, 0.15),
                composition(0.48, 0.34, 0.18),
                composition(0.34, 0.45, 0.21),
                composition(0.22, 0.54, 0.24),
            ],
            liquidus_style,
        ))?
        .label("Liquidus boundary")
        .legend(move |(x, y)| {
            PathElement::new([(x, y), (x + scaled(24, scale) as i32, y)], liquidus_style)
        });

    let solvus_style = stroke_style(RGBColor(20, 145, 95), 3, pass, scale);
    chart
        .draw_series(
            TernarySmoothSeries::new(
                [
                    composition(0.14, 0.68, 0.18),
                    composition(0.20, 0.55, 0.25),
                    composition(0.28, 0.42, 0.30),
                    composition(0.38, 0.29, 0.33),
                    composition(0.50, 0.16, 0.34),
                ],
                solvus_style,
            )
            .interpolation(TernaryInterpolation::Pchip),
        )?
        .label("PCHIP solvus boundary")
        .legend(move |(x, y)| {
            PathElement::new([(x, y), (x + scaled(24, scale) as i32, y)], solvus_style)
        });

    let measured_style = fill_style(RGBColor(205, 45, 55), pass);
    chart
        .draw_series(
            TernaryPointSeries::new([
                composition(0.58, 0.25, 0.17),
                composition(0.43, 0.37, 0.20),
                composition(0.30, 0.46, 0.24),
            ])
            .size(scaled(7, scale))
            .style(measured_style)
            .marker(MarkerShape::Circle),
        )?
        .label("Measured samples")
        .legend(move |coordinate| Circle::new(coordinate, scaled(6, scale), measured_style));

    let reference_style = fill_style(RGBColor(238, 145, 25), pass);
    chart
        .draw_series(
            TernaryPointSeries::new([composition(0.56, 0.16, 0.28), composition(0.39, 0.25, 0.36)])
                .size(scaled(8, scale))
                .style(reference_style)
                .marker(MarkerShape::Triangle),
        )?
        .label("Reference samples")
        .legend(move |coordinate| {
            TriangleMarker::new(coordinate, scaled(7, scale), reference_style)
        });

    let calibration_style = stroke_style(RGBColor(125, 65, 165), 2, pass, scale);
    chart
        .draw_point_series(
            TernaryPointSeries::new([composition(0.30, 0.30, 0.40)])
                .size(scaled(9, scale))
                .style(calibration_style),
            |coordinate, size, style| {
                EmptyElement::at(coordinate)
                    + Cross::new((0, 0), size, style)
                    + Circle::new((0, 0), size / 2, style)
            },
        )?
        .label("Calibration composition")
        .legend(move |coordinate| Cross::new(coordinate, scaled(7, scale), calibration_style));

    draw_legend(&mut chart, SeriesLabelPosition::UpperRight, pass, scale)?;
    drop(chart);
    root.present()?;
    Ok(())
}

fn render_cropped<DB>(
    root: DrawingArea<DB, Shift>,
    pass: RenderPass,
    scale: u32,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    let viewport = TernaryViewport::new(0.35, 0.65, 0.15, 0.50)?;
    let caption_color = BLACK.mix(if pass.draws_text() { 1.0 } else { 0.0 });
    let mut chart = TernaryChartBuilder::on(&root)
        .caption(
            "Mathematically clipped crossing series",
            (
                "sans-serif",
                scaled(32, scale),
                FontStyle::Bold,
                &caption_color,
            ),
        )
        .margin(scaled(55, scale))
        .viewport(viewport)
        .build()?;
    let mesh = chart
        .configure_mesh()
        .major_step(0.1)
        .major_grid_style(RGBColor(210, 216, 224).stroke_width(scale))
        .hide_axis_names()
        .hide_corner_names();
    let mesh = if pass.draws_geometry() {
        mesh
    } else {
        mesh.hide_grid_lines().hide_triangle_boundary()
    };
    mesh.draw()?;

    let crossing_style = stroke_style(RGBColor(210, 55, 50), 5, pass, scale);
    chart
        .draw_series(
            TernaryLineSeries::new(
                [
                    composition(0.346_410_161_6, 0.626_794_919_2, 0.026_794_919_2),
                    composition(0.346_410_161_6, 0.026_794_919_2, 0.626_794_919_2),
                ],
                crossing_style,
            )
            .normalization(Normalization::Normalize),
        )?
        .label("Outside endpoints, visible crossing")
        .legend(move |(x, y)| {
            PathElement::new([(x, y), (x + scaled(24, scale) as i32, y)], crossing_style)
        });

    let reentry_style = stroke_style(RGBColor(0, 102, 204), 4, pass, scale);
    chart
        .draw_series(
            TernaryLineSeries::new(
                [
                    composition(0.288_675_134_6, 0.435_662_433_0, 0.275_662_433_0),
                    composition(0.808_290_376_8, 0.095_854_811_6, 0.095_854_811_6),
                    composition(0.288_675_134_6, 0.275_662_433_0, 0.435_662_433_0),
                ],
                reentry_style,
            )
            .normalization(Normalization::Normalize),
        )?
        .label("Exit and re-entry (two subpaths)")
        .legend(move |(x, y)| {
            PathElement::new([(x, y), (x + scaled(24, scale) as i32, y)], reentry_style)
        });

    let marker_style = fill_style(RGBColor(25, 150, 105), pass);
    chart
        .draw_series(
            TernaryPointSeries::new([
                composition(1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0),
                composition(0.82, 0.08, 0.10),
                composition(0.08, 0.82, 0.10),
            ])
            .size(scaled(8, scale))
            .style(marker_style)
            .marker(MarkerShape::Circle),
        )?
        .label("Centre-clipped markers")
        .legend(move |coordinate| Circle::new(coordinate, scaled(6, scale), marker_style));

    draw_legend(&mut chart, SeriesLabelPosition::UpperLeft, pass, scale)?;
    drop(chart);
    root.present()?;
    Ok(())
}

fn stroke_style(color: RGBColor, width: u32, pass: RenderPass, scale: u32) -> ShapeStyle {
    if pass.draws_geometry() {
        color.stroke_width(scaled(width, scale))
    } else {
        color.mix(0.0).stroke_width(scaled(width, scale))
    }
}

fn fill_style(color: RGBColor, pass: RenderPass) -> ShapeStyle {
    if pass.draws_geometry() {
        color.filled()
    } else {
        color.mix(0.0).filled()
    }
}

fn draw_legend<'series, DB>(
    chart: &mut plotters_ternary::TernaryChart<'series, DB>,
    position: SeriesLabelPosition,
    pass: RenderPass,
    scale: u32,
) -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>>
where
    DB: DrawingBackend + 'series,
{
    let text_color = BLACK.mix(if pass.draws_text() { 1.0 } else { 0.0 });
    let background = WHITE.mix(if pass.draws_geometry() { 0.88 } else { 0.0 });
    let border = stroke_style(RGBColor(35, 45, 60), 1, pass, scale);
    chart
        .configure_series_labels()
        .position(position)
        .background_style(background)
        .border_style(border)
        .label_font((
            "sans-serif",
            scaled(22, scale),
            FontStyle::Normal,
            &text_color,
        ))
        .legend_area_size(scaled(34, scale))
        .margin(scaled(12, scale))
        .draw()?;
    Ok(())
}

const fn composition(a: f64, b: f64, c: f64) -> TernaryPoint {
    TernaryPoint::new(a, b, c)
}
