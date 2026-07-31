use std::error::Error;

use plotters::coord::Shift;
use plotters::drawing::DrawingArea;
use plotters::prelude::*;
use plotters_ternary::{
    AxisTextStyle, ContourColorBar, ContourColorBarOrientation, ContourColorBarPosition,
    ContourLabelAnchor, ContourLabelConfig, ContourLabelMode, ContourLabelPlacement,
    ContourLabelStyle, ContourLegendPolicy, ContourOptions, ContourSet, EndpointLabelPolicy,
    RegularTernaryScalarField, TernaryChart, TernaryChartBuilder, TernaryContourSeries,
    TernaryViewport, TickSpec,
};

use crate::output_support::{
    BitmapQuality, BitmapRenderOptions, render_png, render_svg, reserve_final_caption_space, scaled,
};

const OUTPUT_SIZE: (u32, u32) = (1000, 800);
const QUALITY: BitmapQuality = BitmapQuality::Supersampled { factor: 3 };

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
pub enum AdvancedContourExample {
    LevelLegend,
    ColorBar,
    TangentLabels,
    CurvedLabels,
    CroppedLabels,
    ManualLabels,
    RepeatedLabels,
}
impl AdvancedContourExample {
    fn stem(self) -> &'static str {
        match self {
            Self::LevelLegend => "contour_level_legend",
            Self::ColorBar => "contour_color_bar",
            Self::TangentLabels => "contour_labels",
            Self::CurvedLabels => "curved_contour_labels",
            Self::CroppedLabels => "cropped_contour_labels",
            Self::ManualLabels => "manual_contour_labels",
            Self::RepeatedLabels => "repeated_contour_labels",
        }
    }
    fn caption(self) -> &'static str {
        match self {
            Self::LevelLegend => "Heatmap-coloured contours with level legend",
            Self::ColorBar => "Heatmap-coloured contours with continuous colour bar",
            Self::TangentLabels => "Tangent-aligned contour labels",
            Self::CurvedLabels => "Curved labels following contour arc length",
            Self::CroppedLabels => "Collision-aware labels in a cropped viewport",
            Self::ManualLabels => "Manual semantic contour-label anchors",
            Self::RepeatedLabels => "Repeated labels along long contour components",
        }
    }
    fn cropped(self) -> bool {
        matches!(self, Self::CroppedLabels)
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Pass {
    Geometry,
    Text,
}

pub fn run(example: AdvancedContourExample) -> Result<(), Box<dyn Error>> {
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
    println!("Wrote {png} and {svg}");
    Ok(())
}

fn render<DB>(
    root: DrawingArea<DB, Shift>,
    example: AdvancedContourExample,
    pass: Pass,
    scale: u32,
) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    if pass == Pass::Geometry {
        root.fill(&WHITE)?;
    }
    let chart_root = if pass == Pass::Geometry {
        reserve_final_caption_space(&root, example.caption(), 32, scale)?
    } else {
        root.clone()
    };
    let builder = TernaryChartBuilder::on(&chart_root);
    let builder = if pass == Pass::Text {
        builder.caption(
            example.caption(),
            ("sans-serif", 32, FontStyle::Bold, &BLACK),
        )
    } else {
        builder
    };
    let viewport = example
        .cropped()
        .then(|| TernaryViewport::new(0.42, 0.88, 0.08, 0.58))
        .transpose()?;
    let builder = if let Some(viewport) = viewport {
        builder.viewport(viewport)
    } else {
        builder
    };
    let mut chart = builder
        .margin(scaled(if example.cropped() { 82 } else { 68 }, scale))
        .build()?;
    draw_mesh(&mut chart, pass, scale, example.cropped())?;
    let contours = contours()?;
    draw_contours(&mut chart, &contours, example, pass, scale)?;
    if pass == Pass::Text {
        draw_labels(&chart, &contours, example)?;
    }
    draw_color_bar(&chart, example, pass, scale)?;
    if matches!(example, AdvancedContourExample::LevelLegend) {
        draw_legend(&mut chart, pass, scale)?;
    }
    drop(chart);
    root.present()?;
    Ok(())
}

fn contours() -> Result<ContourSet, Box<dyn Error>> {
    let field = RegularTernaryScalarField::from_fn(28, |[a, b, c]| {
        (a - 0.43).powi(2) + 0.85 * (b - 0.30).powi(2) + 1.10 * (c - 0.27).powi(2) + 0.025 * (b - c)
    })?;
    Ok(ContourSet::compute(
        &field,
        &[0.025, 0.055, 0.095, 0.15, 0.22],
        ContourOptions::linear(),
    )?)
}
fn heat(t: f64) -> RGBAColor {
    let t = t.clamp(0.0, 1.0);
    let r = (40.0 + 210.0 * t).round() as u8;
    let g = (70.0 + 130.0 * (1.0 - (2.0 * t - 1.0).abs())).round() as u8;
    let b = (230.0 - 190.0 * t).round() as u8;
    RGBColor(r, g, b).to_rgba()
}
fn style(level: f64, scale: u32, visible: bool) -> ShapeStyle {
    let t = (level - 0.025) / (0.22 - 0.025);
    let mut color = heat(t);
    if !visible {
        color.3 = 0.0;
    }
    color.stroke_width(scaled(3, scale))
}

fn draw_contours<'a, DB: DrawingBackend + 'a>(
    chart: &mut TernaryChart<'a, DB>,
    set: &ContourSet,
    example: AdvancedContourExample,
    pass: Pass,
    scale: u32,
) -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>> {
    let visible = pass == Pass::Geometry;
    let heatmapped = matches!(
        example,
        AdvancedContourExample::LevelLegend | AdvancedContourExample::ColorBar
    );
    let series = if heatmapped {
        TernaryContourSeries::new(set).style_by_level(move |level| style(level, scale, visible))
    } else {
        TernaryContourSeries::new(set).style(
            RGBColor(28, 38, 52)
                .mix(if visible { 1.0 } else { 0.0 })
                .stroke_width(scaled(2, scale)),
        )
    };
    let series = if matches!(example, AdvancedContourExample::LevelLegend) {
        series
            .legend_policy(ContourLegendPolicy::EveryLevel)
            .level_formatter(|v| format!("{:.0} °C", v * 1000.0))
    } else {
        series
    };
    chart.draw_series(series)?;
    Ok(())
}
fn draw_labels<'a, DB: DrawingBackend + 'a>(
    chart: &TernaryChart<'a, DB>,
    set: &ContourSet,
    example: AdvancedContourExample,
) -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>> {
    if matches!(
        example,
        AdvancedContourExample::LevelLegend | AdvancedContourExample::ColorBar
    ) {
        return Ok(());
    }
    let placement = match example {
        AdvancedContourExample::RepeatedLabels => {
            ContourLabelPlacement::Repeated { spacing: 175.0 }
        }
        AdvancedContourExample::ManualLabels => ContourLabelPlacement::Manual(vec![
            ContourLabelAnchor::new(0.055, 0, 0.32),
            ContourLabelAnchor::new(0.15, 0, 0.68),
        ]),
        _ => ContourLabelPlacement::Automatic,
    };
    let mode = if matches!(example, AdvancedContourExample::CurvedLabels) {
        ContourLabelMode::Curved
    } else {
        ContourLabelMode::Tangent
    };
    let labels = ContourLabelConfig::new()
        .mode(mode)
        .placement(placement)
        .formatter(|v| format!("{:.0} °C", v * 1000.0))
        .style(
            ContourLabelStyle::new(AxisTextStyle::sans_serif(
                19,
                FontStyle::Bold,
                RGBColor(25, 35, 50).to_rgba(),
            ))
            .halo(WHITE, 2),
        )
        .minimum_visible_length(55.0)
        .endpoint_clearance(7.0)
        .viewport_clearance(10.0)
        .maximum_curvature_degrees(42.0);
    chart.draw_contour_labels(set, &labels)
}
fn bar() -> Result<ContourColorBar<'static>, plotters_ternary::ContourDisplayError> {
    ContourColorBar::new(0.025, 0.22, heat).map(|bar| {
        bar.title("Scalar level")
            .formatter(|v| format!("{:.0}", v * 1000.0))
            .position(ContourColorBarPosition::UpperRight)
            .orientation(ContourColorBarOrientation::Vertical)
            .size(210, 20)
    })
}
fn draw_color_bar<'a, DB: DrawingBackend + 'a>(
    chart: &TernaryChart<'a, DB>,
    example: AdvancedContourExample,
    pass: Pass,
    scale: u32,
) -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>> {
    if !matches!(example, AdvancedContourExample::ColorBar) {
        return Ok(());
    }
    let bar = bar().map_err(plotters_ternary::SeriesError::ContourDisplay)?;
    if pass == Pass::Geometry {
        chart.draw_contour_color_bar_geometry(&bar, scale)
    } else {
        chart.draw_contour_color_bar_text(&bar)
    }
}
fn draw_mesh<'a, DB: DrawingBackend + 'a>(
    chart: &mut TernaryChart<'a, DB>,
    pass: Pass,
    scale: u32,
    cropped: bool,
) -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>> {
    let navy = RGBColor(28, 38, 52);
    let name = AxisTextStyle::sans_serif(24, FontStyle::Bold, navy.to_rgba());
    let mesh = chart
        .configure_mesh()
        .boundary_style(navy.stroke_width(scaled(3, scale)))
        .corner_a_name("A")
        .corner_b_name("B")
        .corner_c_name("C")
        .corner_label_style(("sans-serif", scaled(27, scale), FontStyle::Bold, &navy))
        .axis_a(|axis| {
            axis.axis_name("A component")
                .axis_name_style(name.clone())
                .major_ticks(TickSpec::Step(0.2))
                .draw_major_grid(true)
                .draw_ticks(false)
                .draw_tick_labels(false)
                .endpoint_label_policy(EndpointLabelPolicy::InteriorOnly);
        })
        .axis_b(|axis| {
            axis.axis_name("B component")
                .axis_name_style(name.clone())
                .major_ticks(TickSpec::Step(0.2))
                .draw_major_grid(true)
                .draw_ticks(false)
                .draw_tick_labels(false)
                .endpoint_label_policy(EndpointLabelPolicy::InteriorOnly);
        })
        .axis_c(|axis| {
            axis.axis_name("C component")
                .axis_name_style(name)
                .major_ticks(TickSpec::Step(0.2))
                .draw_major_grid(true)
                .draw_ticks(false)
                .draw_tick_labels(false)
                .endpoint_label_policy(EndpointLabelPolicy::InteriorOnly);
        });
    let _ = cropped;
    if pass == Pass::Geometry {
        mesh.draw_geometry_scaled(scale)
    } else {
        mesh.draw_text()
    }
}
fn draw_legend<'a, DB: DrawingBackend + 'a>(
    chart: &mut TernaryChart<'a, DB>,
    pass: Pass,
    scale: u32,
) -> Result<(), plotters_ternary::TernaryChartError<DB::ErrorType>> {
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .background_style(WHITE.mix(if pass == Pass::Geometry { 0.92 } else { 0.0 }))
        .border_style(
            BLACK
                .mix(if pass == Pass::Geometry { 1.0 } else { 0.0 })
                .stroke_width(scale),
        )
        .label_font((
            "sans-serif",
            scaled(18, scale),
            FontStyle::Normal,
            &BLACK.mix(if pass == Pass::Text { 1.0 } else { 0.0 }),
        ))
        .legend_area_size(scaled(34, scale))
        .margin(scaled(8, scale))
        .draw()?;
    Ok(())
}
