use std::error::Error;

use plotters::backend::DrawingBackend;
use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters::prelude::*;
use plotters_ternary::{
    TernaryChartBuilder, TernaryGeometry, TernaryViewport, ViewportAlignment, ViewportFit,
};

use crate::output_support::{BitmapQuality, BitmapRenderOptions, render_png, render_svg, scaled};

const OUTPUT_SIZE: (u32, u32) = (1_000, 800);
const BITMAP_QUALITY: BitmapQuality = BitmapQuality::Supersampled { factor: 3 };

#[derive(Clone, Copy)]
pub(crate) enum RenderPass {
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
pub enum ExampleView {
    Full,
    CroppedRight,
    Interior,
}

impl ExampleView {
    fn stem(self) -> &'static str {
        match self {
            Self::Full => "full_triangle",
            Self::CroppedRight => "cropped_right",
            Self::Interior => "interior_view",
        }
    }

    fn caption(self) -> &'static str {
        match self {
            Self::Full => "Full ternary diagram",
            Self::CroppedRight => "Right-cropped ternary diagram",
            Self::Interior => "Interior ternary viewport",
        }
    }

    const fn margin(self) -> u32 {
        match self {
            Self::Full => 100,
            Self::CroppedRight | Self::Interior => 55,
        }
    }
    fn viewport(self, geometry: TernaryGeometry) -> TernaryViewport {
        match self {
            Self::Full => TernaryViewport::full(geometry),
            Self::CroppedRight => TernaryViewport::new(0.55, 1.02, -0.03, 0.65).unwrap(),
            Self::Interior => TernaryViewport::new(0.30, 0.70, 0.15, 0.35).unwrap(),
        }
    }
}

pub fn write_outputs(view: ExampleView) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all("examples/output/png")?;
    std::fs::create_dir_all("examples/output/svg")?;

    let png_path = format!("examples/output/png/{}.png", view.stem());
    render_png(
        &png_path,
        BitmapRenderOptions::new(OUTPUT_SIZE, BITMAP_QUALITY),
        |root, scale| render(root, view, RenderPass::Geometry, scale),
        |root| render(root, view, RenderPass::Text, 1),
    )?;

    let svg_path = format!("examples/output/svg/{}.svg", view.stem());
    render_svg(
        &svg_path,
        OUTPUT_SIZE,
        |root| render(root, view, RenderPass::Geometry, 1),
        |root| render(root, view, RenderPass::Text, 1),
    )?;

    println!("Wrote {png_path} and {svg_path}");
    Ok(())
}

pub(crate) fn render<DB>(
    root: DrawingArea<DB, Shift>,
    view: ExampleView,
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

    let geometry = TernaryGeometry::default();
    let viewport = view.viewport(geometry);
    let caption_color = if pass.draws_text() {
        BLACK.mix(1.0)
    } else {
        BLACK.mix(0.0)
    };
    let mut chart = TernaryChartBuilder::on(&root)
        .caption(
            view.caption(),
            (
                "sans-serif",
                scaled(32, scale),
                FontStyle::Bold,
                &caption_color,
            ),
        )
        .margin(scaled(view.margin(), scale))
        .geometry(geometry)
        .viewport(viewport)
        .viewport_fit(ViewportFit::PreserveAspect)
        .viewport_alignment(ViewportAlignment::Center)
        .build()?;

    let mesh = chart
        .configure_mesh()
        .major_step(0.1)
        .boundary_style(RGBColor(35, 45, 60).stroke_width(scaled(3, scale)))
        .major_grid_style(RGBColor(166, 177, 190).stroke_width(scale))
        .axis_a_name("Component A axis")
        .axis_b_name("Component B axis")
        .axis_c_name("Component C axis")
        .corner_a_name("Pure A corner")
        .corner_b_name("Pure B corner")
        .corner_c_name("Pure C corner")
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

    drop(chart);
    root.present()?;
    Ok(())
}
