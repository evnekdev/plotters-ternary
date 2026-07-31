use crate::coord::{
    CartesianSegment, Normalization, TernaryCartesian, TernaryGeometry, TernaryPoint,
    TernaryViewport, Tolerance, clip_segment,
};

use super::{InvalidPointPolicy, MarkerClipMode, SeriesError};

/// A point prepared for marker rendering while retaining its source identity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PreparedPoint {
    pub index: usize,
    /// Validated and normalized semantic composition in A/B/C order.
    pub composition: TernaryPoint,
    pub cartesian: TernaryCartesian,
}

/// Project and clip a ternary polyline into directed visible logical subpaths.
///
/// Invalid points either return [`SeriesError::InvalidPoint`] or terminate the
/// current source run according to `invalid_policy`. Distinct runs and
/// materially invisible source segments are never joined.
pub fn prepare_polyline<I, P>(
    geometry: TernaryGeometry,
    viewport: TernaryViewport,
    points: I,
    normalization: Normalization,
    tolerance: Tolerance,
    invalid_policy: InvalidPointPolicy,
) -> Result<Vec<Vec<TernaryCartesian>>, SeriesError>
where
    I: IntoIterator<Item = P>,
    P: Into<TernaryPoint>,
{
    let mut visible = Vec::new();
    let mut run = Vec::new();

    for (index, point) in points.into_iter().enumerate() {
        match geometry.project(point.into(), normalization, tolerance) {
            Ok(projected) => run.push(projected),
            Err(source) => match invalid_policy {
                InvalidPointPolicy::Error => {
                    return Err(SeriesError::InvalidPoint { index, source });
                }
                InvalidPointPolicy::Break => {
                    prepare_projected_run(&run, viewport, tolerance, &mut visible)
                        .map_err(|source| SeriesError::InvalidPoint { index, source })?;
                    run.clear();
                }
            },
        }
    }

    prepare_projected_run(&run, viewport, tolerance, &mut visible).map_err(|source| {
        SeriesError::InvalidPoint {
            index: run.len(),
            source,
        }
    })?;
    Ok(visible)
}

/// Project a point series and apply centre-based or unrestricted marker policy.
///
/// This public convenience projection discards source identity. Internally,
/// The crate internal preparation path retains source identity for per-point marker styles.
pub fn prepare_points<I, P>(
    geometry: TernaryGeometry,
    viewport: TernaryViewport,
    points: I,
    normalization: Normalization,
    tolerance: Tolerance,
    invalid_policy: InvalidPointPolicy,
    clip_mode: MarkerClipMode,
) -> Result<Vec<TernaryCartesian>, SeriesError>
where
    I: IntoIterator<Item = P>,
    P: Into<TernaryPoint>,
{
    Ok(prepare_points_with_source(
        geometry,
        viewport,
        points,
        normalization,
        tolerance,
        invalid_policy,
        clip_mode,
    )?
    .into_iter()
    .map(|point| point.cartesian)
    .collect())
}

/// Project marker points while retaining original index and normalized semantic
/// composition for a per-point style provider.
pub(crate) fn prepare_points_with_source<I, P>(
    geometry: TernaryGeometry,
    viewport: TernaryViewport,
    points: I,
    normalization: Normalization,
    tolerance: Tolerance,
    invalid_policy: InvalidPointPolicy,
    clip_mode: MarkerClipMode,
) -> Result<Vec<PreparedPoint>, SeriesError>
where
    I: IntoIterator<Item = P>,
    P: Into<TernaryPoint>,
{
    let mut prepared = Vec::new();
    for (index, source) in points.into_iter().enumerate() {
        let validated = match source.into().validate(normalization, tolerance) {
            Ok(point) => point,
            Err(source) => match invalid_policy {
                InvalidPointPolicy::Error => {
                    return Err(SeriesError::InvalidPoint { index, source });
                }
                InvalidPointPolicy::Break => continue,
            },
        };
        let sum = validated.sum();
        let [a, b, c] = validated.as_array();
        let composition = TernaryPoint::new(a / sum, b / sum, c / sum);
        let cartesian = geometry
            .project(composition, Normalization::RequireUnitSum, tolerance)
            .map_err(|source| SeriesError::InvalidPoint { index, source })?;
        if clip_mode == MarkerClipMode::None
            || viewport
                .contains(cartesian, tolerance)
                .map_err(|source| SeriesError::InvalidPoint { index, source })?
        {
            prepared.push(PreparedPoint {
                index,
                composition,
                cartesian,
            });
        }
    }
    Ok(prepared)
}

fn prepare_projected_run(
    run: &[TernaryCartesian],
    viewport: TernaryViewport,
    tolerance: Tolerance,
    visible: &mut Vec<Vec<TernaryCartesian>>,
) -> Result<(), crate::coord::Error> {
    if let [point] = run {
        if viewport.contains(*point, tolerance)? {
            visible.push(vec![*point]);
        }
        return Ok(());
    }

    let mut current = Vec::new();
    for pair in run.windows(2) {
        let source = CartesianSegment::new(pair[0], pair[1]);
        let Some(clipped) = clip_segment(source, viewport, tolerance)? else {
            finish_subpath(&mut current, visible);
            continue;
        };

        if current.is_empty() {
            push_distinct(&mut current, clipped.start, tolerance);
        } else if !points_close(
            *current.last().expect("non-empty"),
            clipped.start,
            tolerance,
        ) {
            finish_subpath(&mut current, visible);
            push_distinct(&mut current, clipped.start, tolerance);
        }
        push_distinct(&mut current, clipped.end, tolerance);

        if !points_close(clipped.end, source.end, tolerance) {
            finish_subpath(&mut current, visible);
        }
    }
    finish_subpath(&mut current, visible);
    Ok(())
}

fn push_distinct(path: &mut Vec<TernaryCartesian>, point: TernaryCartesian, tolerance: Tolerance) {
    if path
        .last()
        .is_none_or(|last| !points_close(*last, point, tolerance))
    {
        path.push(point);
    }
}

fn finish_subpath(current: &mut Vec<TernaryCartesian>, visible: &mut Vec<Vec<TernaryCartesian>>) {
    if !current.is_empty() {
        visible.push(std::mem::take(current));
    }
}

fn points_close(left: TernaryCartesian, right: TernaryCartesian, tolerance: Tolerance) -> bool {
    tolerance.is_close(left.x, right.x) && tolerance.is_close(left.y, right.y)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    const TOLERANCE: Tolerance = Tolerance {
        absolute: 1.0e-9,
        relative: 1.0e-9,
    };

    fn geometry() -> TernaryGeometry {
        TernaryGeometry::default()
    }

    fn viewport() -> TernaryViewport {
        TernaryViewport::new(0.4, 0.6, 0.2, 0.4).unwrap()
    }

    fn composition(x: f64, y: f64) -> TernaryPoint {
        geometry()
            .unproject(TernaryCartesian::new(x, y), TOLERANCE)
            .unwrap()
    }

    fn prepare(points: Vec<TernaryPoint>) -> Vec<Vec<TernaryCartesian>> {
        prepare_polyline(
            geometry(),
            viewport(),
            points,
            Normalization::RequireUnitSum,
            TOLERANCE,
            InvalidPointPolicy::Error,
        )
        .unwrap()
    }

    fn assert_point(actual: TernaryCartesian, expected: (f64, f64)) {
        assert!((actual.x - expected.0).abs() < 1.0e-9, "{actual:?}");
        assert!((actual.y - expected.1).abs() < 1.0e-9, "{actual:?}");
    }

    #[test]
    fn fully_inside_and_fully_outside_lines_are_distinguished() {
        let inside = prepare(vec![composition(0.45, 0.25), composition(0.55, 0.35)]);
        assert_eq!(inside.len(), 1);
        assert_point(inside[0][0], (0.45, 0.25));
        assert_point(inside[0][1], (0.55, 0.35));

        let outside = prepare(vec![composition(0.35, 0.5), composition(0.65, 0.5)]);
        assert!(outside.is_empty());
    }

    #[test]
    fn both_outside_endpoints_still_produce_a_crossing() {
        let paths = prepare(vec![composition(0.2, 0.3), composition(0.8, 0.3)]);
        assert_eq!(paths.len(), 1);
        assert_point(paths[0][0], (0.4, 0.3));
        assert_point(paths[0][1], (0.6, 0.3));
    }

    #[test]
    fn entering_leaving_and_direction_are_preserved() {
        let forward = prepare(vec![composition(0.2, 0.3), composition(0.5, 0.3)]);
        assert_point(forward[0][0], (0.4, 0.3));
        assert_point(forward[0][1], (0.5, 0.3));

        let reversed = prepare(vec![composition(0.5, 0.3), composition(0.2, 0.3)]);
        assert_point(reversed[0][0], (0.5, 0.3));
        assert_point(reversed[0][1], (0.4, 0.3));

        let leaving = prepare(vec![composition(0.5, 0.3), composition(0.8, 0.3)]);
        assert_point(leaving[0][0], (0.5, 0.3));
        assert_point(leaving[0][1], (0.6, 0.3));
    }

    #[test]
    fn an_outside_excursion_splits_exit_and_reentry() {
        let paths = prepare(vec![
            composition(0.45, 0.3),
            composition(0.5, 0.6),
            composition(0.55, 0.3),
        ]);
        assert_eq!(paths.len(), 2);
        assert_point(paths[0][0], (0.45, 0.3));
        assert_point(*paths[0].last().unwrap(), (0.466_666_666_666_666_7, 0.4));
        assert_point(paths[1][0], (0.533_333_333_333_333_3, 0.4));
        assert_point(*paths[1].last().unwrap(), (0.55, 0.3));
    }

    #[test]
    fn corner_tangent_and_boundary_coincident_segments_survive() {
        let tangent = prepare(vec![composition(0.3, 0.3), composition(0.4, 0.2)]);
        assert_eq!(tangent, vec![vec![TernaryCartesian::new(0.4, 0.2)]]);

        let boundary = prepare(vec![composition(0.4, 0.25), composition(0.4, 0.35)]);
        assert_eq!(boundary.len(), 1);
        assert_point(boundary[0][0], (0.4, 0.25));
        assert_point(boundary[0][1], (0.4, 0.35));
    }

    #[test]
    fn repeated_and_single_points_do_not_create_adjacent_duplicates() {
        let repeated = prepare(vec![
            composition(0.5, 0.3),
            composition(0.5, 0.3),
            composition(0.55, 0.3),
        ]);
        assert_eq!(repeated.len(), 1);
        assert_eq!(repeated[0].len(), 2);

        let single = prepare(vec![composition(0.5, 0.3)]);
        assert_eq!(single.len(), 1);
        assert_eq!(single[0].len(), 1);
        assert_point(single[0][0], (0.5, 0.3));
    }

    #[test]
    fn invalid_points_report_index_or_break_runs_without_joining() {
        let invalid = TernaryPoint::new(f64::NAN, 0.0, 1.0);
        let points = vec![composition(0.45, 0.3), invalid, composition(0.55, 0.3)];
        assert!(matches!(
            prepare_polyline(
                geometry(),
                viewport(),
                points.clone(),
                Normalization::RequireUnitSum,
                TOLERANCE,
                InvalidPointPolicy::Error,
            ),
            Err(SeriesError::InvalidPoint { index: 1, .. })
        ));
        let broken = prepare_polyline(
            geometry(),
            viewport(),
            points,
            Normalization::RequireUnitSum,
            TOLERANCE,
            InvalidPointPolicy::Break,
        )
        .unwrap();
        assert_eq!(broken.len(), 2);
        assert_eq!(broken[0].len(), 1);
        assert_eq!(broken[1].len(), 1);
    }

    #[test]
    fn centre_and_none_marker_clipping_are_distinct() {
        let points = vec![composition(0.5, 0.3), composition(0.7, 0.3)];
        let centre = prepare_points(
            geometry(),
            viewport(),
            points.clone(),
            Normalization::RequireUnitSum,
            TOLERANCE,
            InvalidPointPolicy::Error,
            MarkerClipMode::Centre,
        )
        .unwrap();
        let none = prepare_points(
            geometry(),
            viewport(),
            points,
            Normalization::RequireUnitSum,
            TOLERANCE,
            InvalidPointPolicy::Error,
            MarkerClipMode::None,
        )
        .unwrap();
        assert_eq!(centre.len(), 1);
        assert_eq!(none.len(), 2);
    }

    #[test]
    fn point_preparation_reports_or_breaks_at_invalid_values() {
        let points = vec![
            composition(0.5, 0.3),
            TernaryPoint::new(f64::INFINITY, 0.0, 0.0),
            composition(0.55, 0.3),
        ];
        assert!(matches!(
            prepare_points(
                geometry(),
                viewport(),
                points.clone(),
                Normalization::RequireUnitSum,
                TOLERANCE,
                InvalidPointPolicy::Error,
                MarkerClipMode::Centre,
            ),
            Err(SeriesError::InvalidPoint { index: 1, .. })
        ));
        let broken = prepare_points(
            geometry(),
            viewport(),
            points,
            Normalization::RequireUnitSum,
            TOLERANCE,
            InvalidPointPolicy::Break,
            MarkerClipMode::Centre,
        )
        .unwrap();
        assert_eq!(broken.len(), 2);
    }

    proptest! {        #[test]
        fn every_prepared_endpoint_is_contained_and_adjacent_points_are_distinct(
            triples in prop::collection::vec((0.0_f64..1.0, 0.0_f64..1.0, 0.0_f64..1.0), 1..30)
        ) {
            let points: Vec<_> = triples
                .into_iter()
                .filter(|(a, b, c)| a + b + c > 1.0e-6)
                .map(|(a, b, c)| TernaryPoint::new(a, b, c))
                .collect();
            let paths = prepare_polyline(
                geometry(),
                viewport(),
                points,
                Normalization::Normalize,
                TOLERANCE,
                InvalidPointPolicy::Error,
            ).unwrap();
            for path in paths {
                for point in &path {
                    prop_assert!(viewport().contains(*point, TOLERANCE).unwrap());
                }
                for pair in path.windows(2) {
                    prop_assert!(!points_close(pair[0], pair[1], TOLERANCE));
                }
            }
        }
    }
}
