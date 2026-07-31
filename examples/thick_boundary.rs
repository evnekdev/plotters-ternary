//! Demonstrate joined foreground simplex boundaries over data that reaches the axes.
mod output_support;

use std::error::Error;

use output_support::{
    BitmapQuality, BitmapRenderOptions, render_png, render_svg, reserve_final_caption_space, scaled,
};
use plotters::backend::DrawingBackend;
use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters::prelude::*;
use plotters_ternary::{
    ContourOptions, ContourSet, MarkerShape, RegularTernaryScalarField, TernaryChartBuilder,
    TernaryContourSeries, TernaryLineSeries, TernaryPoint, TernaryPointSeries,
};

const OUTPUT_SIZE: (u32, u32) = (1000, 800);
const QUALITY: BitmapQuality = BitmapQuality::Supersampled { factor: 3 };
const CAPTION: &str = "Joined foreground triangle frame";

#[derive(Clone, Copy)]
enum Pass {
    Geometry,
    Text,
}

impl Pass {
    const fn geometry(self) -> bool {
        matches!(self, Self::Geometry)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all("examples/output/png")?;
    std::fs::create_dir_all("examples/output/svg")?;
    render_png(
        "examples/output/png/thick_boundary.png",
        BitmapRenderOptions::new(OUTPUT_SIZE, QUALITY),
        |root, scale| render(root, Pass::Geometry, scale),
        |root| render(root, Pass::Text, 1),
    )?;
    render_svg(
        "examples/output/svg/thick_boundary.svg",
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
    let chart_root = if pass.geometry() {
        reserve_final_caption_space(&root, CAPTION, 34, scale)?
    } else {
        root.clone()
    };
    let builder = TernaryChartBuilder::on(&chart_root);
    let builder = if pass.geometry() {
        builder
    } else {
        builder.caption(CAPTION, ("sans-serif", 34, FontStyle::Bold, &BLACK))
    };
    let mut chart = builder.margin(scaled(68, scale)).build()?;
    let mesh = chart
        .configure_mesh()
        .major_step(0.2)
        .boundary_style(RGBColor(20, 30, 44).stroke_width(30))
        .major_grid_style(RGBColor(210, 216, 224).stroke_width(1))
        .axis_a_name("A")
        .axis_b_name("B")
        .axis_c_name("C")
        .corner_a_name("A")
        .corner_b_name("B")
        .corner_c_name("C")
        .build();

    if pass.geometry() {
        mesh.draw_background_scaled(&mut chart, scale)?;
        chart.draw_series(TernaryLineSeries::new(
            [
                TernaryPoint::new(1.0, 0.0, 0.0),
                TernaryPoint::new(0.0, 0.5, 0.5),
                TernaryPoint::new(0.0, 1.0, 0.0),
                TernaryPoint::new(1.0, 0.0, 0.0),
            ],
            RGBColor(205, 45, 55).stroke_width(scaled(12, scale)),
        ))?;
        let field = RegularTernaryScalarField::from_fn(9, |[a, b, c]| 2.0 * a - b + 0.35 * c)?;
        let contours = ContourSet::compute(&field, &[-0.35, 0.15, 0.65], ContourOptions::linear())?;
        chart.draw_series(
            TernaryContourSeries::new(&contours)
                .style(RGBColor(20, 110, 180).stroke_width(scaled(6, scale))),
        )?;
        chart.draw_series(
            TernaryPointSeries::new([
                TernaryPoint::new(0.0, 0.65, 0.35),
                TernaryPoint::new(0.0, 0.35, 0.65),
            ])
            .size(scaled(11, scale))
            .style(RGBColor(238, 145, 25).filled())
            .marker(MarkerShape::Diamond),
        )?;
        mesh.draw_foreground_scaled(&mut chart, scale)?;
    } else {
        mesh.draw_text(&mut chart)?;
    }
    drop(chart);
    root.present()?;
    Ok(())
}
