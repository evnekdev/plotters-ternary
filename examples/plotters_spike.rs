use plotters::backend::DrawingBackend;
use plotters::coord::Shift;
use plotters::drawing::{DrawingArea, DrawingAreaErrorKind};
use plotters::prelude::*;

const WIDTH: u32 = 1_000;
const HEIGHT: u32 = 900;
const MARGIN: u32 = 20;
const CAPTION: &str = "Plotters Cartesian integration spike";
const TRIANGLE_HEIGHT: f64 = 0.866_025_403_784_438_6;

fn render<DB: DrawingBackend>(
    root: DrawingArea<DB, Shift>,
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
    root.fill(&WHITE)?;

    let caption_style = ("sans-serif", 32).into_font().color(&BLACK);
    let (_, caption_height) = root.estimate_text_size(CAPTION, &caption_style)?;
    let title_padding = 2 * (caption_height / 2).min(5);
    let plot_width = WIDTH - 2 * MARGIN;
    let plot_height = HEIGHT - 2 * MARGIN - caption_height - title_padding;
    let x_min = -0.1;
    let x_max = 1.1;
    let x_span = x_max - x_min;
    let y_span = x_span * f64::from(plot_height - 1) / f64::from(plot_width - 1);
    let y_centre = TRIANGLE_HEIGHT / 2.0;
    let y_min = y_centre - y_span / 2.0;
    let y_max = y_centre + y_span / 2.0;

    let mut chart = ChartBuilder::on(&root)
        .caption(CAPTION, caption_style)
        .margin(MARGIN)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart
        .draw_series(std::iter::once(PathElement::new(
            [(0.0, 0.0), (1.0, 0.0), (0.5, TRIANGLE_HEIGHT), (0.0, 0.0)],
            BLACK.stroke_width(3),
        )))?
        .label("Triangle boundary")
        .legend(|(x, y)| PathElement::new([(x - 10, y), (x + 10, y)], BLACK.stroke_width(3)));

    let mut grid_lines = Vec::new();
    for fraction in [0.2, 0.4, 0.6, 0.8] {
        let y = fraction * TRIANGLE_HEIGHT;
        grid_lines.push(vec![(fraction / 2.0, y), (1.0 - fraction / 2.0, y)]);
        grid_lines.push(vec![
            (fraction, 0.0),
            (0.5 + fraction / 2.0, TRIANGLE_HEIGHT * (1.0 - fraction)),
        ]);
        grid_lines.push(vec![
            (1.0 - fraction, 0.0),
            (0.5 - fraction / 2.0, TRIANGLE_HEIGHT * (1.0 - fraction)),
        ]);
    }
    chart.draw_series(
        grid_lines
            .into_iter()
            .map(|line| PathElement::new(line, RGBColor(180, 180, 180).stroke_width(1))),
    )?;

    let guide = [
        (0.14, 0.07),
        (0.28, 0.31),
        (0.50, 0.39),
        (0.72, 0.25),
        (0.83, 0.11),
    ];
    chart
        .draw_series(std::iter::once(PathElement::new(
            guide,
            RED.stroke_width(3),
        )))?
        .label("Ordinary Plotters series")
        .legend(|(x, y)| PathElement::new([(x - 10, y), (x + 10, y)], RED.stroke_width(3)));

    chart.draw_series(std::iter::once(
        EmptyElement::at((0.47, 0.32))
            + Circle::new((0, 0), 7, BLUE.filled())
            + Text::new(
                "Composable marker",
                (12, -12),
                ("sans-serif", 18).into_font().color(&BLUE),
            ),
    ))?;

    chart
        .configure_series_labels()
        .background_style(WHITE.mix(0.8))
        .border_style(BLACK)
        .label_font(("sans-serif", 18))
        .position(SeriesLabelPosition::UpperRight)
        .draw()?;

    root.present()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("examples/output/png")?;
    std::fs::create_dir_all("examples/output/svg")?;

    let png_path = "examples/output/png/plotters_spike.png";
    render(BitMapBackend::new(png_path, (WIDTH, HEIGHT)).into_drawing_area())?;

    let svg_path = "examples/output/svg/plotters_spike.svg";
    render(SVGBackend::new(svg_path, (WIDTH, HEIGHT)).into_drawing_area())?;

    println!("Wrote {png_path} and {svg_path}");
    Ok(())
}
