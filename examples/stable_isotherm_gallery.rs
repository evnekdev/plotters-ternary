use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use plotters::coord::Shift;
use plotters::drawing::IntoDrawingArea;
use plotters::prelude::*;
use plotters_ternary::{
    FieldInterpolation, PreparedStablePhaseEnsemble, StableContourQuantity, StablePhaseId,
    StablePhaseSource, StableScalarSource, StableUmbrellaOptions, TernaryChartBuilder,
    TernaryStableContourSeries,
};
use ternary_contours::{LiquidusFieldSpec, RegularTernaryScalarField, TernaryCoordinate};

const PANEL_SIZE: (u32, u32) = (520, 430);
const GALLERY_SIZE: (u32, u32) = (2_080, 1_720);
const PALETTE: [RGBColor; 8] = [
    RGBColor(31, 119, 180),
    RGBColor(214, 39, 40),
    RGBColor(44, 160, 44),
    RGBColor(148, 103, 189),
    RGBColor(255, 127, 14),
    RGBColor(23, 190, 207),
    RGBColor(140, 86, 75),
    RGBColor(227, 119, 194),
];

#[derive(Clone, Debug)]
struct GalleryCase {
    name: &'static str,
    title: &'static str,
    specs: Vec<LiquidusFieldSpec>,
    levels: Vec<f64>,
    subdivisions: usize,
    quantity: StableContourQuantity,
    secondary: bool,
}

impl GalleryCase {
    fn height(
        name: &'static str,
        title: &'static str,
        specs: Vec<LiquidusFieldSpec>,
        levels: Vec<f64>,
        subdivisions: usize,
    ) -> Self {
        Self {
            name,
            title,
            specs,
            levels,
            subdivisions,
            quantity: StableContourQuantity::Height,
            secondary: false,
        }
    }
    fn secondary(
        name: &'static str,
        title: &'static str,
        specs: Vec<LiquidusFieldSpec>,
        levels: Vec<f64>,
        subdivisions: usize,
    ) -> Self {
        Self {
            name,
            title,
            specs,
            levels,
            subdivisions,
            quantity: StableContourQuantity::Secondary,
            secondary: true,
        }
    }
}

fn iso(
    phase: u32,
    centre: [f64; 3],
    maximum: f64,
    steepness: f64,
    quartic: f64,
) -> LiquidusFieldSpec {
    LiquidusFieldSpec::isotropic(
        StablePhaseId(phase),
        TernaryCoordinate::new(centre[0], centre[1], centre[2]),
        maximum,
        steepness,
        quartic,
    )
}

fn cases() -> Vec<GalleryCase> {
    vec![
        GalleryCase::height(
            "corner-symmetric",
            "1. Symmetric corner phases",
            vec![
                LiquidusFieldSpec::corner_a(StablePhaseId(1), 100.0, 80.0),
                LiquidusFieldSpec::corner_b(StablePhaseId(2), 100.0, 80.0),
                LiquidusFieldSpec::corner_c(StablePhaseId(3), 100.0, 80.0),
            ],
            vec![25.0, 40.0, 55.0, 70.0, 80.0, 90.0],
            24,
        ),
        GalleryCase::height(
            "corner-steepness",
            "2. Unequal corner steepness",
            vec![
                iso(1, [1.0, 0.0, 0.0], 100.0, 42.0, 8.0),
                iso(2, [0.0, 1.0, 0.0], 100.0, 82.0, 18.0),
                iso(3, [0.0, 0.0, 1.0], 100.0, 170.0, 45.0),
            ],
            vec![25.0, 40.0, 55.0, 70.0, 82.0, 92.0],
            24,
        ),
        GalleryCase::height(
            "corner-maxima",
            "3. Unequal corner maxima",
            vec![
                iso(1, [1.0, 0.0, 0.0], 112.0, 70.0, 16.0),
                iso(2, [0.0, 1.0, 0.0], 102.0, 75.0, 12.0),
                iso(3, [0.0, 0.0, 1.0], 94.0, 75.0, 12.0),
            ],
            vec![25.0, 45.0, 65.0, 80.0, 92.0, 102.0, 108.0],
            24,
        ),
        GalleryCase::height(
            "edge-maxima",
            "4. Three binary-edge maxima",
            vec![
                LiquidusFieldSpec::edge_ab(StablePhaseId(1), 0.35, 104.0, 105.0),
                LiquidusFieldSpec::edge_ac(StablePhaseId(2), 0.55, 103.0, 105.0),
                LiquidusFieldSpec::edge_bc(StablePhaseId(3), 0.45, 102.0, 105.0),
            ],
            vec![35.0, 50.0, 65.0, 78.0, 88.0, 96.0],
            28,
        ),
        GalleryCase::height(
            "corner-edge",
            "5. Corner plus edge maxima",
            vec![
                iso(1, [1.0, 0.0, 0.0], 106.0, 78.0, 12.0),
                LiquidusFieldSpec::edge_ab(StablePhaseId(2), 0.38, 103.0, 90.0),
                LiquidusFieldSpec::edge_bc(StablePhaseId(3), 0.58, 101.0, 94.0),
            ],
            vec![30.0, 45.0, 60.0, 74.0, 86.0, 96.0],
            28,
        ),
        GalleryCase::height(
            "interior-maximum",
            "6. One interior maximum",
            vec![
                iso(1, [0.34, 0.36, 0.30], 109.0, 145.0, 320.0),
                iso(2, [1.0, 0.0, 0.0], 103.0, 60.0, 8.0),
                iso(3, [0.0, 1.0, 0.0], 102.0, 64.0, 8.0),
            ],
            vec![55.0, 70.0, 82.0, 91.0, 99.0, 105.0],
            30,
        ),
        GalleryCase::height(
            "interior-maxima",
            "7. Several interior maxima",
            vec![
                iso(1, [0.25, 0.42, 0.33], 105.0, 72.0, 20.0),
                iso(2, [0.56, 0.22, 0.22], 103.0, 75.0, 24.0),
                iso(3, [0.22, 0.26, 0.52], 101.0, 78.0, 28.0),
            ],
            vec![30.0, 48.0, 65.0, 78.0, 88.0, 96.0],
            30,
        ),
        GalleryCase::height(
            "mixed-topology",
            "8. Mixed topology",
            vec![
                iso(1, [1.0, 0.0, 0.0], 106.0, 70.0, 12.0),
                LiquidusFieldSpec::edge_bc(StablePhaseId(2), 0.44, 104.0, 92.0),
                iso(3, [0.30, 0.36, 0.34], 105.0, 115.0, 110.0),
                iso(4, [0.50, 0.28, 0.22], 103.0, 125.0, 160.0),
            ],
            vec![35.0, 52.0, 68.0, 80.0, 90.0, 98.0],
            30,
        ),
        GalleryCase::height(
            "narrow-phase-coarse",
            "9a. Narrow stable phase (coarse n=8)",
            vec![
                iso(1, [0.34, 0.33, 0.33], 100.0, 52.0, 4.0),
                iso(2, [0.34, 0.33, 0.33], 100.8, 900.0, 1_200.0),
                iso(3, [0.0, 0.0, 1.0], 99.0, 60.0, 8.0),
            ],
            vec![96.0, 98.0, 99.5, 100.2],
            8,
        ),
        GalleryCase::height(
            "narrow-phase-refined",
            "9b. Narrow stable phase (refined n=32)",
            vec![
                iso(1, [0.34, 0.33, 0.33], 100.0, 52.0, 4.0),
                iso(2, [0.34, 0.33, 0.33], 100.8, 900.0, 1_200.0),
                iso(3, [0.0, 0.0, 1.0], 99.0, 60.0, 8.0),
            ],
            vec![96.0, 98.0, 99.5, 100.2],
            32,
        ),
        GalleryCase::height(
            "metastable-pair",
            "10. Metastable A-B equality suppressed by C",
            vec![
                iso(1, [1.0, 0.0, 0.0], 100.0, 46.0, 4.0),
                iso(2, [0.0, 1.0, 0.0], 100.0, 46.0, 4.0),
                iso(3, [0.34, 0.33, 0.33], 108.0, 34.0, 4.0),
            ],
            vec![70.0, 82.0, 92.0, 99.0],
            24,
        ),
        GalleryCase::height(
            "asymmetric-fields",
            "11. Strongly asymmetric fields",
            vec![
                LiquidusFieldSpec::new(
                    StablePhaseId(1),
                    TernaryCoordinate::new(0.58, 0.22, 0.20),
                    105.0,
                    [[180.0, 55.0], [55.0, 48.0]],
                    40.0,
                ),
                LiquidusFieldSpec::new(
                    StablePhaseId(2),
                    TernaryCoordinate::new(0.22, 0.56, 0.22),
                    104.0,
                    [[48.0, -34.0], [-34.0, 190.0]],
                    42.0,
                ),
                LiquidusFieldSpec::new(
                    StablePhaseId(3),
                    TernaryCoordinate::new(0.20, 0.22, 0.58),
                    103.0,
                    [[112.0, -42.0], [-42.0, 72.0]],
                    60.0,
                ),
            ],
            vec![40.0, 58.0, 72.0, 84.0, 94.0, 100.0],
            30,
        ),
        GalleryCase::secondary(
            "secondary-scalar",
            "12. Height-gated secondary scalar",
            vec![
                iso(1, [1.0, 0.0, 0.0], 104.0, 58.0, 8.0),
                iso(2, [0.0, 1.0, 0.0], 103.0, 62.0, 8.0),
                iso(3, [0.34, 0.33, 0.33], 105.0, 105.0, 80.0),
            ],
            vec![0.20, 0.35, 0.50, 0.65, 0.80],
            28,
        ),
    ]
}

fn options(subdivisions: usize) -> StableUmbrellaOptions {
    StableUmbrellaOptions {
        subdivisions,
        value_tolerance: 1.0e-9,
        stability_tolerance: 1.0e-9,
        geometry_tolerance: 1.0e-9,
        parameter_tolerance: 1.0e-12,
        ..StableUmbrellaOptions::default()
    }
}
fn phase_color(phase: StablePhaseId) -> ShapeStyle {
    PALETTE[(phase.0.saturating_sub(1) as usize) % PALETTE.len()].stroke_width(1)
}
fn secondary_value(phase: StablePhaseId, [a, b, c]: [f64; 3]) -> f64 {
    match phase.0 {
        1 => 0.05 + 0.85 * b + 0.10 * c,
        2 => 0.08 + 0.20 * a + 0.72 * c,
        _ => 0.12 + 0.62 * a + 0.28 * b,
    }
}

fn render_case<DB>(root: &DrawingArea<DB, Shift>, case: &GalleryCase) -> Result<(), Box<dyn Error>>
where
    DB: DrawingBackend,
    DB::ErrorType: 'static,
{
    root.fill(&WHITE)?;
    let fields: Vec<RegularTernaryScalarField> = case
        .specs
        .iter()
        .map(|spec| spec.sample(case.subdivisions))
        .collect::<Result<_, _>>()?;
    let secondary_fields: Vec<RegularTernaryScalarField> = if case.secondary {
        case.specs
            .iter()
            .map(|spec| {
                let phase = spec.phase;
                RegularTernaryScalarField::from_fn(case.subdivisions, move |composition| {
                    secondary_value(phase, composition)
                })
            })
            .collect::<Result<_, _>>()?
    } else {
        Vec::new()
    };
    let phases = case
        .specs
        .iter()
        .zip(&fields)
        .enumerate()
        .map(|(index, (spec, field))| {
            let height = StablePhaseSource::new(
                spec.phase,
                StableScalarSource::regular(field, FieldInterpolation::Linear),
            );
            if case.secondary {
                height.with_secondary(StableScalarSource::regular(
                    &secondary_fields[index],
                    FieldInterpolation::Linear,
                ))
            } else {
                height
            }
        })
        .collect::<Vec<_>>();
    let prepared =
        PreparedStablePhaseEnsemble::new(phases, case.quantity, options(case.subdivisions))?;
    let contours = prepared.contours(&case.levels)?;
    let path_count: usize = contours.levels.iter().map(|level| level.paths.len()).sum();
    println!(
        "{}: n={} levels={} paths={} junctions={} interior-only={}",
        case.name,
        case.subdivisions,
        contours.levels.len(),
        path_count,
        contours
            .levels
            .iter()
            .map(|level| level.junctions.len())
            .sum::<usize>(),
        prepared
            .diagnostics()
            .interior_stable_polygons_without_vertex_winner
    );
    let mut chart = TernaryChartBuilder::on(root)
        .caption(
            format!("{}  [n={}]", case.title, case.subdivisions),
            ("sans-serif", 16, FontStyle::Bold, &BLACK),
        )
        .margin(24)
        .build()?;
    chart.configure_mesh().draw()?;
    chart.draw_series(
        TernaryStableContourSeries::new(&contours)
            .style_by_phase(phase_color)
            .legend(case.specs.len() <= 4)
            .phase_formatter(|phase| format!("Phase {}", phase.0)),
    )?;
    if case.specs.len() <= 4 {
        chart
            .configure_series_labels()
            .background_style(WHITE.mix(0.82))
            .border_style(BLACK)
            .draw()?;
    }
    Ok(())
}
fn render_individual(case: &GalleryCase, path: &Path) -> Result<(), Box<dyn Error>> {
    let root = SVGBackend::new(path, PANEL_SIZE).into_drawing_area();
    render_case(&root, case)?;
    root.present()?;
    Ok(())
}
fn render_gallery(cases: &[GalleryCase], path: &Path) -> Result<(), Box<dyn Error>> {
    let root = SVGBackend::new(path, GALLERY_SIZE).into_drawing_area();
    root.fill(&WHITE)?;
    let panels = root.split_evenly((4, 4));
    for (panel, case) in panels.iter().zip(cases) {
        render_case(panel, case)?;
    }
    root.present()?;
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let output_dir = PathBuf::from("docs/images/stable-isotherms");
    fs::create_dir_all(&output_dir)?;
    let cases = cases();
    for case in &cases {
        render_individual(case, &output_dir.join(format!("{}.svg", case.name)))?;
    }
    render_gallery(&cases, Path::new("docs/images/stable-isotherm-gallery.svg"))?;
    println!(
        "Wrote {} individual panels and docs/images/stable-isotherm-gallery.svg",
        cases.len()
    );
    Ok(())
}
