use std::error::Error;

use plotters::backend::DrawingBackend;
use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters::prelude::*;
use plotters_ternary::{
    TernaryChartBuilder, TernaryGeometry, TernaryViewport, ViewportAlignment, ViewportFit,
};

const SIZE: (u32, u32) = (1_000, 800);

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
    render(
        BitMapBackend::new(&png_path, SIZE).into_drawing_area(),
        view,
    )?;

    let svg_path = format!("examples/output/svg/{}.svg", view.stem());
    render(SVGBackend::new(&svg_path, SIZE).into_drawing_area(), view)?;

    println!("Wrote {png_path} and {svg_path}");
    Ok(())
}

fn render<DB>(root: DrawingArea<DB, Shift>, view: ExampleView) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    root.fill(&WHITE)?;

    let geometry = TernaryGeometry::default();
    let viewport = view.viewport(geometry);
    let mut chart = TernaryChartBuilder::on(&root)
        .caption(view.caption(), ("sans-serif", 30, FontStyle::Bold))
        .margin(55)
        .geometry(geometry)
        .viewport(viewport)
        .viewport_fit(ViewportFit::PreserveAspect)
        .viewport_alignment(ViewportAlignment::Center)
        .build()?;

    chart
        .configure_mesh()
        .major_step(0.1)
        .boundary_style(RGBColor(35, 45, 60).stroke_width(3))
        .major_grid_style(RGBColor(166, 177, 190).stroke_width(1))
        .axis_a_name("Component A axis")
        .axis_b_name("Component B axis")
        .axis_c_name("Component C axis")
        .corner_a_name("Pure A corner")
        .corner_b_name("Pure B corner")
        .corner_c_name("Pure C corner")
        .text_style(("sans-serif", 19, &RGBColor(25, 35, 50)))
        .draw()?;

    drop(chart);
    root.present()?;
    Ok(())
}
