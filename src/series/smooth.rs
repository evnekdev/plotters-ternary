use plotters::style::ShapeStyle;
use spline1d::Spline;

use crate::coord::{
    Component, Normalization, TernaryCartesian, TernaryGeometry, TernaryPoint, TernaryViewport,
    Tolerance,
};

use super::{InvalidPointPolicy, SeriesError, prepare_polyline};

const DEFAULT_SAMPLES_PER_INTERVAL: u32 = 24;
const MAX_SAMPLES_PER_INTERVAL: u32 = 4_096;
const MAX_TOTAL_SMOOTH_SAMPLES: usize = 100_000;

/// Explicit composition-space interpolation methods backed by `spline1d`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TernaryInterpolation {
    /// Shape-preserving piecewise cubic Hermite interpolation.
    #[default]
    Pchip,
    /// Akima's local cubic interpolation.
    Akima,
    /// Modified Akima interpolation.
    Makima,
    /// Steffen's monotone cubic interpolation.
    Steffen,
}

/// An explicitly interpolated ternary curve.
///
/// Unlike [`super::TernaryLineSeries`], this series samples a `spline1d`
/// interpolant in semantic composition space before projection and clipping.
pub struct TernarySmoothSeries<I> {
    points: I,
    style: ShapeStyle,
    interpolation: TernaryInterpolation,
    samples_per_interval: u32,
    normalization: Normalization,
    tolerance: Tolerance,
    invalid_policy: InvalidPointPolicy,
}

impl<I> TernarySmoothSeries<I> {
    /// Construct an explicit smooth series. PCHIP is the default method.
    pub fn new<S: Into<ShapeStyle>>(points: I, style: S) -> Self {
        Self {
            points,
            style: style.into(),
            interpolation: TernaryInterpolation::default(),
            samples_per_interval: DEFAULT_SAMPLES_PER_INTERVAL,
            normalization: Normalization::RequireUnitSum,
            tolerance: Tolerance::default(),
            invalid_policy: InvalidPointPolicy::Error,
        }
    }

    pub const fn interpolation(mut self, interpolation: TernaryInterpolation) -> Self {
        self.interpolation = interpolation;
        self
    }

    /// Select a bounded fixed number of samples for each source interval.
    pub const fn samples_per_interval(mut self, samples: u32) -> Self {
        self.samples_per_interval = samples;
        self
    }

    pub const fn normalization(mut self, normalization: Normalization) -> Self {
        self.normalization = normalization;
        self
    }

    pub const fn tolerance(mut self, tolerance: Tolerance) -> Self {
        self.tolerance = tolerance;
        self
    }

    pub const fn invalid_point_policy(mut self, policy: InvalidPointPolicy) -> Self {
        self.invalid_policy = policy;
        self
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        I,
        ShapeStyle,
        TernaryInterpolation,
        u32,
        Normalization,
        Tolerance,
        InvalidPointPolicy,
    ) {
        (
            self.points,
            self.style,
            self.interpolation,
            self.samples_per_interval,
            self.normalization,
            self.tolerance,
            self.invalid_policy,
        )
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct SmoothPreparation {
    pub(crate) interpolation: TernaryInterpolation,
    pub(crate) samples_per_interval: u32,
    pub(crate) normalization: Normalization,
    pub(crate) tolerance: Tolerance,
    pub(crate) invalid_policy: InvalidPointPolicy,
}
pub(crate) fn prepare_smooth_polyline<I, P>(
    geometry: TernaryGeometry,
    viewport: TernaryViewport,
    points: I,
    preparation: SmoothPreparation,
) -> Result<Vec<Vec<TernaryCartesian>>, SeriesError>
where
    I: IntoIterator<Item = P>,
    P: Into<TernaryPoint>,
{
    let SmoothPreparation {
        interpolation,
        samples_per_interval,
        normalization,
        tolerance,
        invalid_policy,
    } = preparation;
    validate_sampling(samples_per_interval)?;
    tolerance
        .validate()
        .map_err(|source| SeriesError::InvalidPoint { index: 0, source })?;

    let mut source_run = Vec::new();
    let mut sampled_runs = Vec::new();
    for (index, point) in points.into_iter().enumerate() {
        match unit_composition(point.into(), normalization, tolerance) {
            Ok(point) => source_run.push(point),
            Err(source) => match invalid_policy {
                InvalidPointPolicy::Error => {
                    return Err(SeriesError::InvalidPoint { index, source });
                }
                InvalidPointPolicy::Break => {
                    sample_source_run(
                        &source_run,
                        interpolation,
                        samples_per_interval,
                        tolerance,
                        invalid_policy,
                        &mut sampled_runs,
                    )?;
                    source_run.clear();
                }
            },
        }
    }
    sample_source_run(
        &source_run,
        interpolation,
        samples_per_interval,
        tolerance,
        invalid_policy,
        &mut sampled_runs,
    )?;

    let mut visible = Vec::new();
    for sampled in sampled_runs {
        visible.extend(prepare_polyline(
            geometry,
            viewport,
            sampled,
            Normalization::RequireUnitSum,
            tolerance,
            InvalidPointPolicy::Error,
        )?);
    }
    Ok(visible)
}

fn validate_sampling(samples_per_interval: u32) -> Result<(), SeriesError> {
    if samples_per_interval == 0 || samples_per_interval > MAX_SAMPLES_PER_INTERVAL {
        return Err(SeriesError::InvalidSmoothSampling {
            samples_per_interval,
            maximum: MAX_SAMPLES_PER_INTERVAL,
        });
    }
    Ok(())
}

fn unit_composition(
    point: TernaryPoint,
    normalization: Normalization,
    tolerance: Tolerance,
) -> Result<TernaryPoint, crate::coord::Error> {
    let validated = point.validate(normalization, tolerance)?;
    let sum = validated.sum();
    let [a, b, c] = validated.as_array();
    TernaryPoint::new(a / sum, b / sum, c / sum).validate(Normalization::Normalize, tolerance)
}

fn sample_source_run(
    source: &[TernaryPoint],
    interpolation: TernaryInterpolation,
    samples_per_interval: u32,
    tolerance: Tolerance,
    invalid_policy: InvalidPointPolicy,
    sampled_runs: &mut Vec<Vec<TernaryPoint>>,
) -> Result<(), SeriesError> {
    if source.is_empty() {
        return Ok(());
    }
    if source.len() == 1 {
        sampled_runs.push(source.to_vec());
        return Ok(());
    }

    let interval_count = source.len() - 1;
    let requested = interval_count
        .checked_mul(samples_per_interval as usize)
        .and_then(|value| value.checked_add(1))
        .ok_or(SeriesError::TooManySmoothSamples {
            requested: usize::MAX,
            maximum: MAX_TOTAL_SMOOTH_SAMPLES,
        })?;
    if requested > MAX_TOTAL_SMOOTH_SAMPLES {
        return Err(SeriesError::TooManySmoothSamples {
            requested,
            maximum: MAX_TOTAL_SMOOTH_SAMPLES,
        });
    }

    let parameters: Vec<_> = (0..source.len()).map(|index| index as f64).collect();
    let component_a: Vec<_> = source
        .iter()
        .map(|point| point.component(Component::A))
        .collect();
    let component_b: Vec<_> = source
        .iter()
        .map(|point| point.component(Component::B))
        .collect();
    let spline_a = make_spline(interpolation, &parameters, &component_a);
    let spline_b = make_spline(interpolation, &parameters, &component_b);

    let mut current = Vec::new();
    let mut sample_index = 0;
    for interval in 0..interval_count {
        for subdivision in 0..samples_per_interval {
            let parameter =
                interval as f64 + f64::from(subdivision) / f64::from(samples_per_interval);
            append_sample(
                &spline_a,
                &spline_b,
                parameter,
                sample_index,
                tolerance,
                invalid_policy,
                &mut current,
                sampled_runs,
            )?;
            sample_index += 1;
        }
    }
    append_sample(
        &spline_a,
        &spline_b,
        interval_count as f64,
        sample_index,
        tolerance,
        invalid_policy,
        &mut current,
        sampled_runs,
    )?;
    finish_sampled_run(&mut current, sampled_runs);
    Ok(())
}

fn make_spline(
    interpolation: TernaryInterpolation,
    parameters: &[f64],
    values: &[f64],
) -> Spline<f64> {
    match interpolation {
        TernaryInterpolation::Pchip => Spline::pchip(parameters, values),
        TernaryInterpolation::Akima => Spline::akima(parameters, values),
        TernaryInterpolation::Makima => Spline::makima(parameters, values),
        TernaryInterpolation::Steffen => Spline::steffen(parameters, values),
    }
}

#[allow(clippy::too_many_arguments)]
fn append_sample(
    spline_a: &Spline<f64>,
    spline_b: &Spline<f64>,
    parameter: f64,
    sample_index: usize,
    tolerance: Tolerance,
    invalid_policy: InvalidPointPolicy,
    current: &mut Vec<TernaryPoint>,
    sampled_runs: &mut Vec<Vec<TernaryPoint>>,
) -> Result<(), SeriesError> {
    let a = spline_a
        .interpolate(&parameter)
        .ok_or(SeriesError::SmoothInterpolationFailed {
            sample: sample_index,
            parameter,
        })?;
    let b = spline_b
        .interpolate(&parameter)
        .ok_or(SeriesError::SmoothInterpolationFailed {
            sample: sample_index,
            parameter,
        })?;
    match validate_generated_sample(TernaryPoint::new(a, b, 1.0 - a - b), tolerance) {
        Ok(point) => current.push(point),
        Err(source) => match invalid_policy {
            InvalidPointPolicy::Error => {
                return Err(SeriesError::InvalidInterpolatedPoint {
                    sample: sample_index,
                    source,
                });
            }
            InvalidPointPolicy::Break => finish_sampled_run(current, sampled_runs),
        },
    }
    Ok(())
}

fn validate_generated_sample(
    point: TernaryPoint,
    tolerance: Tolerance,
) -> Result<TernaryPoint, crate::coord::Error> {
    // Generated samples alone receive this explicit numerical cleanup: tiny
    // negatives are zeroed by validation and the result is renormalized.
    point.validate(Normalization::Normalize, tolerance)
}

fn finish_sampled_run(current: &mut Vec<TernaryPoint>, sampled_runs: &mut Vec<Vec<TernaryPoint>>) {
    if !current.is_empty() {
        sampled_runs.push(std::mem::take(current));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coord::{TriangleOrientation, VertexOrder};

    const TOLERANCE: Tolerance = Tolerance {
        absolute: 1.0e-9,
        relative: 1.0e-9,
    };

    fn sample(
        points: &[TernaryPoint],
        interpolation: TernaryInterpolation,
    ) -> Result<Vec<Vec<TernaryPoint>>, SeriesError> {
        let mut sampled = Vec::new();
        sample_source_run(
            points,
            interpolation,
            12,
            TOLERANCE,
            InvalidPointPolicy::Error,
            &mut sampled,
        )?;
        Ok(sampled)
    }

    #[test]
    fn smooth_samples_pass_through_observations_without_mutating_them() {
        let original = vec![
            TernaryPoint::new(0.7, 0.2, 0.1),
            TernaryPoint::new(0.4, 0.35, 0.25),
            TernaryPoint::new(0.15, 0.45, 0.4),
        ];
        let before = original.clone();
        let sampled = sample(&original, TernaryInterpolation::Pchip).unwrap();
        assert_eq!(original, before);
        assert_eq!(sampled.len(), 1);
        assert_eq!(sampled[0].len(), 25);
        for (source_index, source) in original.iter().enumerate() {
            let actual = sampled[0][source_index * 12];
            for component in Component::ALL {
                assert!((actual.component(component) - source.component(component)).abs() < 1.0e-9);
            }
        }
    }

    #[test]
    fn boundary_samples_remain_unit_and_non_negative() {
        let points = [
            TernaryPoint::new(1.0, 0.0, 0.0),
            TernaryPoint::new(0.5, 0.5, 0.0),
            TernaryPoint::new(0.0, 1.0, 0.0),
        ];
        for interpolation in [TernaryInterpolation::Pchip, TernaryInterpolation::Steffen] {
            let sampled = sample(&points, interpolation).unwrap();
            for point in &sampled[0] {
                assert!((point.sum() - 1.0).abs() < 1.0e-12);
                assert!(point.as_array().into_iter().all(|value| value >= 0.0));
                assert!((0.0..=1.0).contains(&point.component(Component::A)));
            }
            for pair in sampled[0].windows(2) {
                assert!(
                    pair[0].component(Component::A) + TOLERANCE.absolute
                        >= pair[1].component(Component::A)
                );
                assert!(
                    pair[0].component(Component::B)
                        <= pair[1].component(Component::B) + TOLERANCE.absolute
                );
            }
        }
    }

    #[test]
    fn materially_invalid_generated_compositions_are_detected() {
        assert!(matches!(
            validate_generated_sample(TernaryPoint::new(0.8, 0.3, -0.1), TOLERANCE),
            Err(crate::coord::Error::NegativeComponent {
                component: Component::C,
                ..
            })
        ));
        let cleaned = validate_generated_sample(
            TernaryPoint::new(0.6, 0.400_000_000_1, -1.0e-10),
            Tolerance::new(1.0e-9, 1.0e-9).unwrap(),
        )
        .unwrap();
        assert!((cleaned.sum() - 1.0).abs() < 1.0e-12);
        assert_eq!(cleaned.component(Component::C), 0.0);
    }

    #[test]
    fn semantic_curve_is_independent_of_vertex_order_and_orientation() {
        let points = [
            TernaryPoint::new(0.65, 0.25, 0.10),
            TernaryPoint::new(0.40, 0.30, 0.30),
            TernaryPoint::new(0.20, 0.45, 0.35),
        ];
        let expected = sample(&points, TernaryInterpolation::Pchip).unwrap();
        let geometries = [
            TernaryGeometry::default(),
            TernaryGeometry::new(TriangleOrientation::Down, VertexOrder::default()),
            TernaryGeometry::new(
                TriangleOrientation::Up,
                VertexOrder::new(Component::C, Component::A, Component::B).unwrap(),
            ),
        ];
        for geometry in geometries {
            let viewport = TernaryViewport::full(geometry);
            let paths = prepare_smooth_polyline(
                geometry,
                viewport,
                points,
                SmoothPreparation {
                    interpolation: TernaryInterpolation::Pchip,
                    samples_per_interval: 12,
                    normalization: Normalization::RequireUnitSum,
                    tolerance: TOLERANCE,
                    invalid_policy: InvalidPointPolicy::Error,
                },
            )
            .unwrap();
            assert_eq!(paths.len(), 1);
            for (projected, expected) in paths[0].iter().zip(&expected[0]) {
                let actual = geometry.unproject(*projected, TOLERANCE).unwrap();
                for component in Component::ALL {
                    assert!(
                        (actual.component(component) - expected.component(component)).abs()
                            < 1.0e-8
                    );
                }
            }
        }
    }

    #[test]
    fn smooth_curve_exit_and_reentry_remain_separate_visible_fragments() {
        let geometry = TernaryGeometry::default();
        let viewport = TernaryViewport::new(0.4, 0.6, 0.2, 0.4).unwrap();
        let points: Vec<_> = [(0.45, 0.3), (0.5, 0.6), (0.55, 0.3)]
            .into_iter()
            .map(|(x, y)| {
                geometry
                    .unproject(TernaryCartesian::new(x, y), TOLERANCE)
                    .unwrap()
            })
            .collect();
        let paths = prepare_smooth_polyline(
            geometry,
            viewport,
            points,
            SmoothPreparation {
                interpolation: TernaryInterpolation::Pchip,
                samples_per_interval: 24,
                normalization: Normalization::RequireUnitSum,
                tolerance: TOLERANCE,
                invalid_policy: InvalidPointPolicy::Error,
            },
        )
        .unwrap();
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn every_exposed_spline1d_mode_produces_finite_unit_samples() {
        let points = [
            TernaryPoint::new(0.65, 0.25, 0.10),
            TernaryPoint::new(0.45, 0.30, 0.25),
            TernaryPoint::new(0.25, 0.40, 0.35),
            TernaryPoint::new(0.10, 0.50, 0.40),
        ];
        for interpolation in [
            TernaryInterpolation::Pchip,
            TernaryInterpolation::Akima,
            TernaryInterpolation::Makima,
            TernaryInterpolation::Steffen,
        ] {
            let sampled = sample(&points, interpolation).unwrap();
            assert_eq!(sampled.len(), 1);
            assert!(sampled[0].iter().all(|point| {
                point.as_array().into_iter().all(f64::is_finite)
                    && (point.sum() - 1.0).abs() < 1.0e-12
            }));
        }
    }
    #[test]
    fn invalid_sampling_is_bounded() {
        for samples_per_interval in [0, MAX_SAMPLES_PER_INTERVAL + 1] {
            assert!(matches!(
                validate_sampling(samples_per_interval),
                Err(SeriesError::InvalidSmoothSampling { .. })
            ));
        }
    }
}
