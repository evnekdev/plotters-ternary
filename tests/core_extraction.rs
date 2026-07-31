use plotters_ternary::{
    FieldInterpolation, GridEvaluationError, GridVertexId, InterpolatedTernaryField,
    LatticeCoordinate, PointBoundaryLocation, RegularTernaryGrid, RegularTernaryScalarField,
};
use ternary_contours::{RegularTernaryScalarField as CoreField, interpolation::AlphaInterval};

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
        assert_eq!(chart.composition(id).unwrap(), [a, b, c]);
    }
    assert_eq!(
        chart
            .vertex_id(LatticeCoordinate { i: 1, j: 1, k: 0 })
            .unwrap(),
        GridVertexId(4)
    );
}

#[test]
fn compatibility_grid_and_evaluation_error_are_direct_core_types() {
    fn core_accepts_grid(value: &ternary_contours::RegularTernaryGrid) -> usize {
        value.vertex_count()
    }

    let grid = RegularTernaryGrid::new(3).unwrap();
    assert_eq!(core_accepts_grid(&grid), 10);
    assert_eq!(
        grid.indexed_compositions().next(),
        Some((GridVertexId(0), [0.0, 0.0, 1.0]))
    );

    let error = RegularTernaryScalarField::try_from_fn(2, |composition| {
        if composition == [0.0, 0.5, 0.5] {
            Err("missing")
        } else {
            Ok(composition[0])
        }
    })
    .unwrap_err();
    assert_eq!(
        error,
        GridEvaluationError::Evaluation {
            index: GridVertexId(1),
            composition: [0.0, 0.5, 0.5],
            source: "missing",
        }
    );
}

#[test]
fn pointwise_interpolation_types_are_direct_core_reexports() {
    let field = RegularTernaryScalarField::from_fn(4, |[a, b, c]| 2.0 * a - b + 3.0 * c).unwrap();
    let grid_location = field.grid().locate([0.25, 0.25, 0.5]).unwrap();
    assert_eq!(grid_location.boundary, PointBoundaryLocation::Vertex);
    let evaluator = InterpolatedTernaryField::new(&field, FieldInterpolation::Linear).unwrap();
    let sample = evaluator.evaluate_at_location(&grid_location).unwrap();
    assert_eq!(sample.location, grid_location);
    assert!((sample.value - 1.75).abs() < 1.0e-12);
    let core: ternary_contours::InterpolatedTernaryField<'_> =
        ternary_contours::InterpolatedTernaryField::new(
            &field,
            ternary_contours::FieldInterpolation::Linear,
        )
        .unwrap();
    assert!((core.value([0.25, 0.25, 0.5]).unwrap() - sample.value).abs() < 1.0e-12);
}

#[test]
fn advanced_plotters_reexports_are_the_core_types() {
    let interval: ternary_contours::interpolation::AlphaInterval =
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
#[test]
fn public_contour_types_are_direct_core_reexports() {
    fn chart_accepts(value: &plotters_ternary::ContourSet) -> &plotters_ternary::ContourSet {
        value
    }
    let field = CoreField::new(1, vec![0.0, 1.0, 2.0]).unwrap();
    let core = ternary_contours::ContourSet::compute(
        &field,
        &[0.5],
        ternary_contours::ContourOptions::linear(),
    )
    .unwrap();
    assert!(std::ptr::eq(chart_accepts(&core), &core));
}
