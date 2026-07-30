use super::{Error, TernaryCartesian, TernaryViewport, Tolerance};

/// A directed segment in the logical ternary Cartesian plane.
///
/// Zero-length segments are valid. Clipping retains one when its point lies
/// inside or on the viewport and rejects it otherwise.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CartesianSegment {
    pub start: TernaryCartesian,
    pub end: TernaryCartesian,
}

impl CartesianSegment {
    pub const fn new(start: TernaryCartesian, end: TernaryCartesian) -> Self {
        Self { start, end }
    }

    /// Evaluate the directed segment at a parameter, normally in `[0, 1]`.
    pub fn point_at(self, parameter: f64) -> TernaryCartesian {
        TernaryCartesian::new(
            self.start.x + (self.end.x - self.start.x) * parameter,
            self.start.y + (self.end.y - self.start.y) * parameter,
        )
    }
}

/// A clipped segment together with its parameter range on the source segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClippedSegment {
    pub segment: CartesianSegment,
    pub parameter_start: f64,
    pub parameter_end: f64,
}

/// Clip a directed segment against a logical viewport using Liang-Barsky.
///
/// The returned direction always matches the source direction. Parallel
/// segments within tolerance of a viewport side are treated as coincident;
/// materially external parallel segments return `None`.
pub fn clip_segment(
    segment: CartesianSegment,
    viewport: TernaryViewport,
    tolerance: Tolerance,
) -> Result<Option<CartesianSegment>, Error> {
    Ok(clip_segment_with_parameters(segment, viewport, tolerance)?.map(|clipped| clipped.segment))
}

/// Clip a segment and retain its original directed parameter interval.
pub fn clip_segment_with_parameters(
    segment: CartesianSegment,
    viewport: TernaryViewport,
    tolerance: Tolerance,
) -> Result<Option<ClippedSegment>, Error> {
    tolerance.validate()?;
    validate_segment(segment)?;

    let start_visible = viewport.contains(segment.start, tolerance)?;
    let end_visible = viewport.contains(segment.end, tolerance)?;
    if start_visible && end_visible {
        return Ok(Some(ClippedSegment {
            segment,
            parameter_start: 0.0,
            parameter_end: 1.0,
        }));
    }

    if segment.start == segment.end {
        return Ok(None);
    }

    let delta_x = segment.end.x - segment.start.x;
    let delta_y = segment.end.y - segment.start.y;
    let constraints = [
        (-delta_x, segment.start.x - viewport.x_min()),
        (delta_x, viewport.x_max() - segment.start.x),
        (-delta_y, segment.start.y - viewport.y_min()),
        (delta_y, viewport.y_max() - segment.start.y),
    ];

    let mut parameter_start: f64 = 0.0;
    let mut parameter_end: f64 = 1.0;
    for (direction, distance) in constraints {
        if direction == 0.0 {
            if distance < 0.0 && !tolerance.is_close(distance, 0.0) {
                return Ok(None);
            }
            continue;
        }

        let parameter = distance / direction;
        if direction < 0.0 {
            parameter_start = parameter_start.max(parameter);
        } else {
            parameter_end = parameter_end.min(parameter);
        }

        if parameter_start > parameter_end {
            if tolerance.is_close(parameter_start, parameter_end) {
                let tangent = (parameter_start + parameter_end) / 2.0;
                parameter_start = tangent;
                parameter_end = tangent;
            } else {
                return Ok(None);
            }
        }
    }

    parameter_start = parameter_start.clamp(0.0, 1.0);
    parameter_end = parameter_end.clamp(0.0, 1.0);
    Ok(Some(ClippedSegment {
        segment: CartesianSegment::new(
            segment.point_at(parameter_start),
            segment.point_at(parameter_end),
        ),
        parameter_start,
        parameter_end,
    }))
}

fn validate_segment(segment: CartesianSegment) -> Result<(), Error> {
    if !segment.start.x.is_finite()
        || !segment.start.y.is_finite()
        || !segment.end.x.is_finite()
        || !segment.end.y.is_finite()
    {
        return Err(Error::NonFiniteSegment {
            start_x: segment.start.x,
            start_y: segment.start.y,
            end_x: segment.end.x,
            end_y: segment.end.y,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const TOLERANCE: Tolerance = Tolerance {
        absolute: 1.0e-9,
        relative: 1.0e-9,
    };
    const EPSILON: f64 = 1.0e-9;

    fn viewport() -> TernaryViewport {
        TernaryViewport::new(0.0, 1.0, 0.0, 1.0).unwrap()
    }

    fn point(x: f64, y: f64) -> TernaryCartesian {
        TernaryCartesian::new(x, y)
    }

    fn segment(start: (f64, f64), end: (f64, f64)) -> CartesianSegment {
        CartesianSegment::new(point(start.0, start.1), point(end.0, end.1))
    }

    fn assert_segment_close(actual: CartesianSegment, expected: CartesianSegment) {
        for (actual, expected) in [
            (actual.start.x, expected.start.x),
            (actual.start.y, expected.start.y),
            (actual.end.x, expected.end.x),
            (actual.end.y, expected.end.y),
        ] {
            assert!(
                (actual - expected).abs() < EPSILON,
                "{actual} != {expected}"
            );
        }
    }

    #[test]
    fn fully_inside_and_outside_segments_are_distinguished() {
        let inside = segment((0.2, 0.3), (0.8, 0.7));
        assert_eq!(
            clip_segment(inside, viewport(), TOLERANCE).unwrap(),
            Some(inside)
        );

        for outside in [
            segment((-2.0, 0.2), (-1.0, 0.8)),
            segment((2.0, 0.2), (3.0, 0.8)),
            segment((0.2, 2.0), (0.8, 3.0)),
            segment((0.2, -2.0), (0.8, -1.0)),
            segment((-2.0, 2.0), (-1.0, 3.0)),
        ] {
            assert_eq!(clip_segment(outside, viewport(), TOLERANCE).unwrap(), None);
        }
    }

    #[test]
    fn crossings_cover_every_side_opposites_adjacent_and_both_endpoints_outside() {
        let cases = [
            (
                segment((-1.0, 0.5), (0.5, 0.5)),
                segment((0.0, 0.5), (0.5, 0.5)),
            ),
            (
                segment((0.5, 0.5), (2.0, 0.5)),
                segment((0.5, 0.5), (1.0, 0.5)),
            ),
            (
                segment((0.5, -1.0), (0.5, 0.5)),
                segment((0.5, 0.0), (0.5, 0.5)),
            ),
            (
                segment((0.5, 0.5), (0.5, 2.0)),
                segment((0.5, 0.5), (0.5, 1.0)),
            ),
            (
                segment((-1.0, 0.5), (2.0, 0.5)),
                segment((0.0, 0.5), (1.0, 0.5)),
            ),
            (
                segment((0.5, -1.0), (0.5, 2.0)),
                segment((0.5, 0.0), (0.5, 1.0)),
            ),
            (
                segment((-1.0, 0.2), (0.8, 1.2)),
                segment((0.0, 0.755_555_555_555_555_5), (0.44, 1.0)),
            ),
            (
                segment((-1.0, -1.0), (2.0, 2.0)),
                segment((0.0, 0.0), (1.0, 1.0)),
            ),
        ];
        for (source, expected) in cases {
            assert_segment_close(
                clip_segment(source, viewport(), TOLERANCE)
                    .unwrap()
                    .unwrap(),
                expected,
            );
        }
    }

    #[test]
    fn reversed_direction_horizontal_vertical_and_diagonal_are_preserved() {
        for source in [
            segment((-1.0, 0.5), (2.0, 0.5)),
            segment((0.5, -1.0), (0.5, 2.0)),
            segment((-1.0, -1.0), (2.0, 2.0)),
        ] {
            let forward = clip_segment(source, viewport(), TOLERANCE)
                .unwrap()
                .unwrap();
            let reversed_source = CartesianSegment::new(source.end, source.start);
            let reversed = clip_segment(reversed_source, viewport(), TOLERANCE)
                .unwrap()
                .unwrap();
            assert_segment_close(reversed, CartesianSegment::new(forward.end, forward.start));
        }
    }

    #[test]
    fn degenerate_boundary_coincident_and_corner_tangent_segments_are_explicit() {
        let zero_inside = segment((0.5, 0.5), (0.5, 0.5));
        assert_eq!(
            clip_segment(zero_inside, viewport(), TOLERANCE).unwrap(),
            Some(zero_inside)
        );
        assert_eq!(
            clip_segment(segment((2.0, 2.0), (2.0, 2.0)), viewport(), TOLERANCE).unwrap(),
            None
        );

        let boundary = segment((0.0, -1.0), (0.0, 2.0));
        assert_segment_close(
            clip_segment(boundary, viewport(), TOLERANCE)
                .unwrap()
                .unwrap(),
            segment((0.0, 0.0), (0.0, 1.0)),
        );
        let tangent = clip_segment(segment((-1.0, 0.0), (0.0, 1.0)), viewport(), TOLERANCE)
            .unwrap()
            .unwrap();
        assert_segment_close(tangent, segment((0.0, 1.0), (0.0, 1.0)));
    }

    #[test]
    fn non_finite_endpoints_are_rejected() {
        for source in [
            segment((f64::NAN, 0.0), (1.0, 1.0)),
            segment((0.0, f64::INFINITY), (1.0, 1.0)),
            segment((0.0, 0.0), (f64::NEG_INFINITY, 1.0)),
        ] {
            assert!(matches!(
                clip_segment(source, viewport(), TOLERANCE),
                Err(Error::NonFiniteSegment { .. })
            ));
        }
    }

    proptest! {
        #[test]
        fn clipped_endpoints_are_visible_and_lie_on_the_source(
            x0 in -5.0_f64..5.0,
            y0 in -5.0_f64..5.0,
            x1 in -5.0_f64..5.0,
            y1 in -5.0_f64..5.0,
        ) {
            let source = segment((x0, y0), (x1, y1));
            if let Some(clipped) = clip_segment_with_parameters(source, viewport(), TOLERANCE).unwrap() {
                prop_assert!(viewport().contains(clipped.segment.start, TOLERANCE).unwrap());
                prop_assert!(viewport().contains(clipped.segment.end, TOLERANCE).unwrap());
                let expected_start = source.point_at(clipped.parameter_start);
                let expected_end = source.point_at(clipped.parameter_end);
                prop_assert!((expected_start.x - clipped.segment.start.x).abs() < EPSILON);
                prop_assert!((expected_start.y - clipped.segment.start.y).abs() < EPSILON);
                prop_assert!((expected_end.x - clipped.segment.end.x).abs() < EPSILON);
                prop_assert!((expected_end.y - clipped.segment.end.y).abs() < EPSILON);
                prop_assert!(clipped.parameter_start >= 0.0);
                prop_assert!(clipped.parameter_end <= 1.0);
                prop_assert!(clipped.parameter_start <= clipped.parameter_end + EPSILON);
            }
        }

        #[test]
        fn reversing_a_source_reverses_its_clipped_result(
            x0 in -5.0_f64..5.0,
            y0 in -5.0_f64..5.0,
            x1 in -5.0_f64..5.0,
            y1 in -5.0_f64..5.0,
        ) {
            let source = segment((x0, y0), (x1, y1));
            let reverse = CartesianSegment::new(source.end, source.start);
            let forward_result = clip_segment(source, viewport(), TOLERANCE).unwrap();
            let reverse_result = clip_segment(reverse, viewport(), TOLERANCE).unwrap();
            prop_assert_eq!(forward_result.is_some(), reverse_result.is_some());
            if let (Some(forward), Some(reversed)) = (forward_result, reverse_result) {
                prop_assert!((forward.start.x - reversed.end.x).abs() < EPSILON);
                prop_assert!((forward.start.y - reversed.end.y).abs() < EPSILON);
                prop_assert!((forward.end.x - reversed.start.x).abs() < EPSILON);
                prop_assert!((forward.end.y - reversed.start.y).abs() < EPSILON);
            }
        }

        #[test]
        fn fully_contained_segments_are_unchanged(
            x0 in 0.0_f64..1.0,
            y0 in 0.0_f64..1.0,
            x1 in 0.0_f64..1.0,
            y1 in 0.0_f64..1.0,
        ) {
            let source = segment((x0, y0), (x1, y1));
            prop_assert_eq!(clip_segment(source, viewport(), TOLERANCE).unwrap(), Some(source));
        }
    }
}
