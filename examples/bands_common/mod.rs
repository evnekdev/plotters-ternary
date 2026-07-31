#![allow(dead_code)]

use std::error::Error;

use plotters::backend::DrawingBackend;
use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters::prelude::*;
use plotters_ternary::{
    ContourBandOptions, ContourBandSet, ContourOptions, ContourSet, ScalarMapResolution,
    TernaryChartBuilder, TernaryContourBandSeries, TernaryContourSeries, TernaryScalarMapSeries,
    TernaryViewport,
};

use crate::output_support::{
    BitmapQuality, BitmapRenderOptions, render_png, render_svg, reserve_final_caption_space, scaled,
};

const SIZE: (u32, u32) = (1000, 800);
const QUALITY: BitmapQuality = BitmapQuality::Supersampled { factor: 3 };

#[derive(Clone, Copy)]
pub enum Example {
    Bands,
    Stepped,
    ScalarMap,
    LabelledMap,
    CroppedBands,
    HoleBands,
    Resolution,
}

impl Example {
    fn stem(self) -> &'static str {
        match self {
            Self::Bands => "filled_contour_bands",
            Self::Stepped => "filled_bands_color_bar",
            Self::ScalarMap => "continuous_scalar_map",
            Self::LabelledMap => "labelled_scalar_map",
            Self::CroppedBands => "cropped_filled_bands",
            Self::HoleBands => "disconnected_band_hole",
            Self::Resolution => "scalar_map_resolution",
        }
    }
    fn caption(self) -> &'static str {
        match self {
            Self::Bands => "Discrete linear filled contour bands",
            Self::Stepped => "Filled bands with stepped scalar intervals",
            Self::ScalarMap => "Continuous piecewise-linear scalar map",
            Self::LabelledMap => "Scalar map with isoline overlay",
            Self::CroppedBands => "Filled bands in an invisible cropped viewport",
            Self::HoleBands => "Disconnected linear bands and an interior hole",
            Self::Resolution => "Coarse scalar-map microtriangles",
        }
    }
}

pub fn write_outputs(example: Example) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all("examples/output/png")?;
    std::fs::create_dir_all("examples/output/svg")?;
    let png = format!("examples/output/png/{}.png", example.stem());
    render_png(
        &png,
        BitmapRenderOptions::new(SIZE, QUALITY),
        |root, scale| render(root, example, true, scale),
        |root| render(root, example, false, 1),
    )?;
    let svg = format!("examples/output/svg/{}.svg", example.stem());
    render_svg(
        &svg,
        SIZE,
        |root| render(root, example, true, 1),
        |root| render(root, example, false, 1),
    )?;
    println!("Wrote {png} and {svg}");
    Ok(())
}

fn render<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
    example: Example,
    geometry_pass: bool,
    scale: u32,
) -> Result<(), Box<dyn Error>>
where
    DB::ErrorType: 'static,
{
    if geometry_pass {
        root.fill(&WHITE)?;
    }
    let caption = example.caption();
    let chart_root = if geometry_pass {
        reserve_final_caption_space(&root, caption, 32, scale)?
    } else {
        root.clone()
    };
    let builder = if matches!(example, Example::CroppedBands) {
        TernaryChartBuilder::on(&chart_root).viewport(TernaryViewport::new(0.32, 0.86, 0.05, 0.58)?)
    } else {
        TernaryChartBuilder::on(&chart_root)
    };
    let builder = if geometry_pass {
        builder
    } else {
        builder.caption(caption, ("sans-serif", 32, FontStyle::Bold, &BLACK))
    };
    let mut chart = builder.margin(scaled(58, scale)).build()?;

    if geometry_pass {
        chart
            .configure_mesh()
            .boundary_style(BLACK.stroke_width(scaled(2, scale)))
            .major_step(0.2)
            .draw_geometry_scaled(scale)?;
        let field = field()?;
        let bands = ContourBandSet::compute(
            &field,
            &[0.05, 0.11, 0.19, 0.29],
            ContourBandOptions::linear(),
        )?;
        if matches!(
            example,
            Example::ScalarMap | Example::LabelledMap | Example::Resolution
        ) {
            let resolution = if matches!(example, Example::Resolution) {
                ScalarMapResolution::Fixed {
                    subdivisions_per_edge: 2,
                }
            } else {
                ScalarMapResolution::Fixed {
                    subdivisions_per_edge: 8,
                }
            };
            chart.draw_series(
                TernaryScalarMapSeries::new(&field)
                    .resolution(resolution)
                    .color_map(|t| HSLColor(0.68 - 0.68 * t, 0.72, 0.56).to_rgba()),
            )?;
        } else {
            chart.draw_series(
                TernaryContourBandSeries::new(&bands, WHITE.filled()).style_by_band(
                    |index, _, _| {
                        let colors = [
                            RGBColor(43, 113, 180),
                            RGBColor(56, 164, 132),
                            RGBColor(244, 181, 63),
                            RGBColor(210, 78, 70),
                            RGBColor(124, 65, 153),
                        ];
                        colors[index % colors.len()].mix(0.88).filled()
                    },
                ),
            )?;
        }
        let contours =
            ContourSet::compute(&field, &[0.05, 0.11, 0.19, 0.29], ContourOptions::linear())?;
        chart.draw_series(
            TernaryContourSeries::new(&contours)
                .style(BLACK.mix(0.64).stroke_width(scaled(2, scale))),
        )?;
    } else {
        chart.configure_mesh().draw_text()?;
    }
    root.present()?;
    Ok(())
}

fn field() -> Result<plotters_ternary::RegularTernaryScalarField, plotters_ternary::FieldError> {
    plotters_ternary::RegularTernaryScalarField::from_fn(14, |[a, b, c]| {
        (a - 0.38).powi(2)
            + 0.9 * (b - 0.28).powi(2)
            + 1.1 * (c - 0.34).powi(2)
            + 0.06 * (4.0 * a * b * c).sin()
    })
}
