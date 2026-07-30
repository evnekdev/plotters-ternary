use plotters::backend::DrawingBackend;
use plotters::element::{PathElement, Text};
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters::style::{BLACK, Color, IntoTextStyle, RGBColor, ShapeStyle, TextStyle};

use crate::coord::{Component, TernaryCartesian, TernaryGeometry, Tolerance};

use super::{TernaryChart, TernaryChartError};

const DEFAULT_MAJOR_STEP: f64 = 0.1;
const MAX_MAJOR_INTERVALS: usize = 10_000;

/// Plotters-like configuration for the first ternary boundary and major mesh.
pub struct TernaryMeshConfig<'chart, 'series, 'font, DB: DrawingBackend> {
    chart: &'chart mut TernaryChart<'series, DB>,
    major_step: f64,
    boundary_style: ShapeStyle,
    major_grid_style: ShapeStyle,
    axis_names: [Option<String>; 3],
    corner_names: [Option<String>; 3],
    text_style: TextStyle<'font>,
    draw_axis_names: bool,
    draw_corner_names: bool,
    draw_grid: bool,
    draw_boundary: bool,
}

impl<'chart, 'series, DB: DrawingBackend> TernaryMeshConfig<'chart, 'series, 'static, DB> {
    pub(crate) fn new(chart: &'chart mut TernaryChart<'series, DB>) -> Self {
        let text_style = ("sans-serif", 18)
            .into_text_style(chart.plotting_area())
            .color(&BLACK);
        Self {
            chart,
            major_step: DEFAULT_MAJOR_STEP,
            boundary_style: BLACK.stroke_width(2),
            major_grid_style: RGBColor(185, 190, 198).stroke_width(1),
            axis_names: [None, None, None],
            corner_names: [None, None, None],
            text_style,
            draw_axis_names: true,
            draw_corner_names: true,
            draw_grid: true,
            draw_boundary: true,
        }
    }
}

impl<'chart, 'series, 'font, DB: DrawingBackend> TernaryMeshConfig<'chart, 'series, 'font, DB> {
    /// Set the common major-grid step in unit composition space.
    pub fn major_step(mut self, step: f64) -> Self {
        self.major_step = step;
        self
    }

    /// Set the style of visible original triangle-edge fragments.
    pub fn boundary_style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.boundary_style = style.into();
        self
    }

    /// Set the style shared by all three major component-grid families.
    pub fn major_grid_style<S: Into<ShapeStyle>>(mut self, style: S) -> Self {
        self.major_grid_style = style.into();
        self
    }

    /// Replace the shared basic axis/corner text style.
    pub fn text_style<'next, S>(self, style: S) -> TernaryMeshConfig<'chart, 'series, 'next, DB>
    where
        S: IntoTextStyle<'next>,
    {
        let text_style = style.into_text_style(self.chart.plotting_area());
        TernaryMeshConfig {
            chart: self.chart,
            major_step: self.major_step,
            boundary_style: self.boundary_style,
            major_grid_style: self.major_grid_style,
            axis_names: self.axis_names,
            corner_names: self.corner_names,
            text_style,
            draw_axis_names: self.draw_axis_names,
            draw_corner_names: self.draw_corner_names,
            draw_grid: self.draw_grid,
            draw_boundary: self.draw_boundary,
        }
    }

    pub fn axis_a_name<S: Into<String>>(mut self, name: S) -> Self {
        self.axis_names[Component::A.index()] = Some(name.into());
        self
    }

    pub fn axis_b_name<S: Into<String>>(mut self, name: S) -> Self {
        self.axis_names[Component::B.index()] = Some(name.into());
        self
    }

    pub fn axis_c_name<S: Into<String>>(mut self, name: S) -> Self {
        self.axis_names[Component::C.index()] = Some(name.into());
        self
    }

    pub fn corner_a_name<S: Into<String>>(mut self, name: S) -> Self {
        self.corner_names[Component::A.index()] = Some(name.into());
        self
    }

    pub fn corner_b_name<S: Into<String>>(mut self, name: S) -> Self {
        self.corner_names[Component::B.index()] = Some(name.into());
        self
    }

    pub fn corner_c_name<S: Into<String>>(mut self, name: S) -> Self {
        self.corner_names[Component::C.index()] = Some(name.into());
        self
    }

    pub fn hide_axis_names(mut self) -> Self {
        self.draw_axis_names = false;
        self
    }

    pub fn hide_corner_names(mut self) -> Self {
        self.draw_corner_names = false;
        self
    }

    pub fn hide_grid_lines(mut self) -> Self {
        self.draw_grid = false;
        self
    }

    pub fn hide_triangle_boundary(mut self) -> Self {
        self.draw_boundary = false;
        self
    }

    /// Draw clipped grid lines, visible triangle edges, and eligible names.
    pub fn draw(self) -> Result<(), TernaryChartError<DB::ErrorType>> {
        let values = major_grid_values(self.major_step)?;
        let geometry = self.chart.geometry;
        let viewport = self.chart.viewport;
        let tolerance = self.chart.tolerance;

        if self.draw_grid {
            for component in Component::ALL {
                for &value in &values {
                    if let Some(segment) =
                        geometry.visible_component_isoline(component, value, viewport, tolerance)?
                    {
                        let path = PathElement::new(
                            [
                                (segment.start.x, segment.start.y),
                                (segment.end.x, segment.end.y),
                            ],
                            self.major_grid_style,
                        );
                        self.chart.plotting_area().draw(&path)?;
                    }
                }
            }
        }

        if self.draw_boundary {
            for visible in geometry.visible_edges(viewport, tolerance)? {
                let segment = visible.segment;
                let path = PathElement::new(
                    [
                        (segment.start.x, segment.start.y),
                        (segment.end.x, segment.end.y),
                    ],
                    self.boundary_style,
                );
                self.chart.plotting_area().draw(&path)?;
            }
        }

        if self.draw_axis_names {
            draw_names(
                self.chart,
                &self.axis_names,
                &self.text_style,
                axis_name_anchor,
                axis_text_anchor,
            )?;
        }
        if self.draw_corner_names {
            draw_names(
                self.chart,
                &self.corner_names,
                &self.text_style,
                |geometry, component| geometry.vertex(component),
                corner_text_anchor,
            )?;
        }

        Ok(())
    }
}

fn draw_names<'series, 'font, DB, Anchor, Position>(
    chart: &TernaryChart<'series, DB>,
    names: &[Option<String>; 3],
    style: &TextStyle<'font>,
    anchor: Anchor,
    position: Position,
) -> Result<(), TernaryChartError<DB::ErrorType>>
where
    DB: DrawingBackend,
    Anchor: Fn(TernaryGeometry, Component) -> TernaryCartesian,
    Position: Fn(TernaryGeometry, Component) -> Pos,
{
    for component in Component::ALL {
        let Some(name) = &names[component.index()] else {
            continue;
        };
        let point = anchor(chart.geometry, component);
        if chart.viewport.contains(point, chart.tolerance)? {
            let text = Text::new(
                name.clone(),
                (point.x, point.y),
                style.clone().pos(position(chart.geometry, component)),
            );
            chart.plotting_area().draw(&text)?;
        }
    }
    Ok(())
}

pub(crate) fn major_grid_values<E: std::error::Error + Send + Sync>(
    step: f64,
) -> Result<Vec<f64>, TernaryChartError<E>> {
    if !step.is_finite() || step <= 0.0 || step > 1.0 {
        return Err(TernaryChartError::InvalidMajorStep { value: step });
    }
    let intervals = (1.0 / step).ceil() as usize;
    if intervals > MAX_MAJOR_INTERVALS {
        return Err(TernaryChartError::InvalidMajorStep { value: step });
    }
    Ok((1..intervals)
        .map(|index| index as f64 * step)
        .filter(|value| *value < 1.0)
        .collect())
}

pub(crate) fn axis_name_anchor(
    geometry: TernaryGeometry,
    component: Component,
) -> TernaryCartesian {
    let [first, second] = component.others();
    midpoint(geometry.vertex(first), geometry.vertex(second))
}

fn midpoint(first: TernaryCartesian, second: TernaryCartesian) -> TernaryCartesian {
    TernaryCartesian::new((first.x + second.x) / 2.0, (first.y + second.y) / 2.0)
}

fn corner_text_anchor(geometry: TernaryGeometry, component: Component) -> Pos {
    let order = geometry.vertex_order();
    if component == order.left() {
        Pos::new(HPos::Left, VPos::Top)
    } else if component == order.right() {
        Pos::new(HPos::Right, VPos::Top)
    } else {
        match geometry.orientation() {
            crate::coord::TriangleOrientation::Up => Pos::new(HPos::Center, VPos::Bottom),
            crate::coord::TriangleOrientation::Down => Pos::new(HPos::Center, VPos::Top),
        }
    }
}

fn axis_text_anchor(geometry: TernaryGeometry, component: Component) -> Pos {
    let anchor = axis_name_anchor(geometry, component);
    if anchor.y.abs() <= Tolerance::default().absolute {
        match geometry.orientation() {
            crate::coord::TriangleOrientation::Up => Pos::new(HPos::Center, VPos::Top),
            crate::coord::TriangleOrientation::Down => Pos::new(HPos::Center, VPos::Bottom),
        }
    } else if anchor.x < 0.5 {
        Pos::new(HPos::Right, VPos::Center)
    } else {
        Pos::new(HPos::Left, VPos::Center)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EQUILATERAL_TRIANGLE_HEIGHT, TernaryViewport, TriangleEdge, TriangleOrientation,
        VertexOrder,
    };

    #[test]
    fn major_values_are_integer_indexed_and_exclude_boundaries() {
        assert_eq!(
            major_grid_values::<std::io::Error>(0.25).unwrap(),
            vec![0.25, 0.5, 0.75]
        );
        assert_eq!(
            major_grid_values::<std::io::Error>(0.3).unwrap(),
            vec![0.3, 0.6, 0.899_999_999_999_999_9]
        );
        assert!(major_grid_values::<std::io::Error>(1.0).unwrap().is_empty());
    }

    #[test]
    fn invalid_major_steps_are_rejected() {
        for step in [0.0, -0.1, 1.1, 1.0e-6, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                major_grid_values::<std::io::Error>(step),
                Err(TernaryChartError::InvalidMajorStep { .. })
            ));
        }
    }

    #[test]
    fn boundary_preparation_covers_full_crop_and_interior_views() {
        let geometry = TernaryGeometry::default();
        let tolerance = Tolerance::default();
        let full = geometry
            .visible_edges(TernaryViewport::full(geometry), tolerance)
            .unwrap();
        assert_eq!(full.len(), 3);

        let crop =
            TernaryViewport::new(0.55, 1.02, -0.03, EQUILATERAL_TRIANGLE_HEIGHT * 0.75).unwrap();
        let crop_edges: Vec<_> = geometry
            .visible_edges(crop, tolerance)
            .unwrap()
            .into_iter()
            .map(|edge| edge.edge)
            .collect();
        assert_eq!(
            crop_edges,
            vec![TriangleEdge::LeftRight, TriangleEdge::RightApex]
        );

        let interior = TernaryViewport::new(0.3, 0.7, 0.15, 0.35).unwrap();
        assert!(
            geometry
                .visible_edges(interior, tolerance)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn component_name_anchors_follow_semantics_under_custom_order() {
        let order = VertexOrder::new(Component::C, Component::A, Component::B).unwrap();
        let geometry = TernaryGeometry::new(TriangleOrientation::Up, order);
        assert_eq!(
            geometry.vertex(Component::A),
            TernaryCartesian::new(1.0, 0.0)
        );
        assert_eq!(
            axis_name_anchor(geometry, Component::A),
            TernaryCartesian::new(0.25, EQUILATERAL_TRIANGLE_HEIGHT / 2.0)
        );
        assert_eq!(
            axis_name_anchor(geometry, Component::B),
            TernaryCartesian::new(0.5, 0.0)
        );
    }
}
