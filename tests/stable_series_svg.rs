use plotters::prelude::*;
use plotters_ternary::{
    FieldInterpolation, PreparedStablePhaseEnsemble, StableContourQuantity, StablePhaseId,
    StablePhaseSource, StableScalarSource, StableUmbrellaOptions, TernaryChartBuilder,
    TernaryStableContourSeries,
};
use ternary_contours::{LiquidusFieldSpec, TernaryCoordinate};

#[test]
fn stable_series_projects_phase_paths_and_registers_phase_legends() {
    let specs = [
        LiquidusFieldSpec::corner_a(StablePhaseId(1), 100.0, 80.0),
        LiquidusFieldSpec::corner_b(StablePhaseId(2), 100.0, 80.0),
        LiquidusFieldSpec::isotropic(
            StablePhaseId(3),
            TernaryCoordinate::new(0.34, 0.33, 0.33),
            101.0,
            120.0,
            50.0,
        ),
    ];
    let fields: Vec<_> = specs.iter().map(|spec| spec.sample(20).unwrap()).collect();
    let phases = specs
        .iter()
        .zip(&fields)
        .map(|(spec, field)| {
            StablePhaseSource::new(
                spec.phase,
                StableScalarSource::regular(field, FieldInterpolation::Linear),
            )
        })
        .collect::<Vec<_>>();
    let set = PreparedStablePhaseEnsemble::new(
        phases,
        StableContourQuantity::Height,
        StableUmbrellaOptions {
            subdivisions: 20,
            ..StableUmbrellaOptions::default()
        },
    )
    .unwrap()
    .contours(&[60.0, 80.0, 92.0])
    .unwrap();

    let mut svg = String::new();
    {
        let root = SVGBackend::with_string(&mut svg, (720, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let mut chart = TernaryChartBuilder::on(&root).margin(35).build().unwrap();
        chart
            .draw_series(
                TernaryStableContourSeries::new(&set)
                    .style_by_phase(|phase| match phase.0 {
                        1 => RED.stroke_width(2),
                        2 => BLUE.stroke_width(2),
                        _ => GREEN.stroke_width(2),
                    })
                    .legend(true)
                    .phase_formatter(|phase| format!("Stable phase {}", phase.0)),
            )
            .unwrap();
        chart.configure_series_labels().draw().unwrap();
        drop(chart);
        root.present().unwrap();
    }
    assert!(svg.contains("Stable phase 1"));
    assert!(svg.contains("Stable phase 2"));
    assert!(svg.contains("#FF0000"));
    assert!(svg.contains("#0000FF"));
    assert!(svg.contains("<polyline"));
    assert!(!svg.contains("<image"));
}
