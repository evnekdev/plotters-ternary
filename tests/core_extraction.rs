use plotters_ternary::{GridVertexId, LatticeCoordinate, RegularTernaryScalarField};
use ternary_contours::{AlphaInterval, RegularTernaryScalarField as CoreField};

#[test]
fn compatibility_field_delegates_indexing_values_and_compositions_to_the_core() {
    let values = vec![0.5, 1.5, 2.5, 3.5, 4.5, 5.5];
    let chart = RegularTernaryScalarField::new(2, values.clone()).unwrap();
    let core = CoreField::new(2, values).unwrap();
    assert_eq!(chart.vertex_count(), core.vertex_count());
    assert_eq!(
        chart.triangle_count().unwrap(),
        core.triangle_count().unwrap()
    );
    assert_eq!(chart.edge_count().unwrap(), core.edge_count().unwrap());
    for index in 0..chart.vertex_count() {
        let id = GridVertexId(index);
        assert_eq!(chart.value(id).unwrap(), core.value(id).unwrap());
        assert_eq!(
            chart.coordinate_of(index).unwrap(),
            core.coordinate_of(index).unwrap()
        );
        let [a, b, c] = core.composition(id).unwrap();
        assert_eq!(chart.composition(id).unwrap().as_array(), [a, b, c]);
    }
    assert_eq!(
        chart
            .vertex_id(LatticeCoordinate { i: 1, j: 1, k: 0 })
            .unwrap(),
        GridVertexId(4)
    );
}

#[test]
fn advanced_plotters_reexports_are_the_core_types() {
    let interval: ternary_contours::AlphaInterval =
        plotters_ternary::interpolation::AlphaInterval::new(2.5, -4.0);
    assert_eq!(interval, AlphaInterval::new(2.5, -4.0));
    assert_eq!(interval.reversed(), AlphaInterval::new(-1.5, 4.0));
}

#[cfg(feature = "cubic-alpha")]
#[test]
fn plotters_contour_options_select_the_core_muggianu_model() {
    use plotters_ternary::interpolation::evaluate_pair;
    use plotters_ternary::{BinaryExtrapolation, ContourOptions, CubicAlphaOptions};
    let options = CubicAlphaOptions::default();
    assert_eq!(options.extrapolation, BinaryExtrapolation::Muggianu);
    assert_eq!(
        ContourOptions::cubic_alpha(options).interpolation,
        plotters_ternary::ContourInterpolation::CubicAlpha(options)
    );
    let pair = evaluate_pair(
        0.2,
        0.3,
        0.5,
        AlphaInterval::new(1.7, -0.9),
        options.extrapolation,
    );
    assert!((pair.parameter - 0.55).abs() < 1.0e-14);
}
