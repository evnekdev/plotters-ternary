use std::collections::BTreeSet;

use plotters::backend::DrawingBackend;
use plotters::chart::SeriesAnno;
use plotters::element::PathElement;
use plotters::style::{BLACK, Color, ShapeStyle};

use crate::chart::{TernaryChart, TernaryChartError};
use crate::coord::{Normalization, TernaryPoint};

use super::{InvalidPointPolicy, TernarySeries, prepare_polyline};

/// Plotters adapter for phase-labelled stable contours.
///
/// The numerical [`ternary_contours::StableContourSet`] is immutable input:
/// this adapter only projects, clips, styles, and optionally labels its paths.
/// Paths are drawn independently, so a stable-phase boundary remains a visible
/// ownership break and paths from different phase IDs are never joined.
pub struct TernaryStableContourSeries<'a> {
    contours: &'a ternary_contours::StableContourSet,
    styles: Box<dyn Fn(ternary_contours::StablePhaseId) -> ShapeStyle + 'a>,
    show_legend: bool,
    formatter: Box<dyn Fn(ternary_contours::StablePhaseId) -> String + 'a>,
}

impl<'a> TernaryStableContourSeries<'a> {
    /// Create a series with black two-pixel paths and no legend entries.
    pub fn new(contours: &'a ternary_contours::StableContourSet) -> Self {
        Self {
            contours,
            styles: Box::new(|_| BLACK.stroke_width(2)),
            show_legend: false,
            formatter: Box::new(|phase| format!("Phase {}", phase.0)),
        }
    }

    /// Select a deterministic style from each stable phase identifier.
    pub fn style_by_phase<F>(mut self, provider: F) -> Self
    where
        F: Fn(ternary_contours::StablePhaseId) -> ShapeStyle + 'a,
    {
        self.styles = Box::new(provider);
        self
    }

    /// Register one native Plotters legend entry per phase.
    pub const fn legend(mut self, enabled: bool) -> Self {
        self.show_legend = enabled;
        self
    }

    /// Format native legend labels from phase identifiers.
    pub fn phase_formatter<F, S>(mut self, formatter: F) -> Self
    where
        F: Fn(ternary_contours::StablePhaseId) -> S + 'a,
        S: Into<String>,
    {
        self.formatter = Box::new(move |phase| formatter(phase).into());
        self
    }
}

impl<'contour, DB> TernarySeries<DB> for TernaryStableContourSeries<'contour>
where
    DB: DrawingBackend,
{
    fn draw<'chart, 'series>(
        self,
        chart: &'chart mut TernaryChart<'series, DB>,
    ) -> Result<&'chart mut SeriesAnno<'series, DB>, TernaryChartError<DB::ErrorType>>
    where
        DB: 'series,
    {
        let mut labelled = BTreeSet::new();
        for level in &self.contours.levels {
            for path in &level.paths {
                let source = path
                    .points
                    .iter()
                    .copied()
                    .map(|point| TernaryPoint::from(point.as_array()))
                    .collect::<Vec<_>>();
                let mut source = source;
                if path.closed && !source.is_empty() {
                    source.push(source[0]);
                }
                let visible = prepare_polyline(
                    chart.geometry,
                    chart.viewport,
                    source,
                    Normalization::RequireUnitSum,
                    chart.tolerance,
                    InvalidPointPolicy::Error,
                )?;
                if visible.is_empty() {
                    continue;
                }
                let phase = path.phase;
                let style = (self.styles)(phase);
                let elements = visible.into_iter().map(|line| {
                    PathElement::new(
                        line.into_iter()
                            .map(|point| (point.x, point.y))
                            .collect::<Vec<_>>(),
                        style,
                    )
                });
                let annotation = chart.context.draw_series(elements)?;
                if self.show_legend && labelled.insert(phase) {
                    let label = (self.formatter)(phase);
                    annotation
                        .label(label)
                        .legend(move |(x, y)| PathElement::new([(x, y), (x + 24, y)], style));
                }
            }
        }
        chart
            .context
            .draw_series(std::iter::empty::<PathElement<(f64, f64)>>())
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_phase_stable_and_legend_is_opt_in() {
        let series = TernaryStableContourSeries::new;
        let _ = series;
    }
}
