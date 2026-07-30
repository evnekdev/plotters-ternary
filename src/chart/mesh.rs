use std::iter::{Once, once};

use plotters::backend::DrawingBackend;
use plotters::element::{Drawable, EmptyElement, PathElement, PointCollection, Text};
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters::style::{BLACK, Color, FontStyle, IntoTextStyle, RGBColor, ShapeStyle, TextStyle};

use crate::coord::{Component, TernaryCartesian, TernaryGeometry};

use super::{TernaryChart, TernaryChartError};

const DEFAULT_MAJOR_STEP: f64 = 0.1;
const MAX_MAJOR_INTERVALS: usize = 10_000;

/// Plotters-like configuration for the first ternary boundary and major mesh.
pub struct TernaryMeshConfig<'chart, 'series, 'axis, 'corner, DB: DrawingBackend> {
    chart: &'chart mut TernaryChart<'series, DB>,
    major_step: f64,
    boundary_style: ShapeStyle,
    major_grid_style: ShapeStyle,
    axis_names: [Option<String>; 3],
    corner_names: [Option<String>; 3],
    axis_name_style: TextStyle<'axis>,
    corner_label_style: TextStyle<'corner>,
    axis_label_offset: u32,
    corner_label_offset: u32,
    draw_axis_names: bool,
    draw_corner_names: bool,
    draw_grid: bool,
    draw_boundary: bool,
}

impl<'chart, 'series, DB: DrawingBackend> TernaryMeshConfig<'chart, 'series, 'static, 'static, DB> {
    pub(crate) fn new(chart: &'chart mut TernaryChart<'series, DB>) -> Self {
        let axis_name_style = ("sans-serif", 26, FontStyle::Bold)
            .into_text_style(chart.plotting_area())
            .color(&BLACK);
        let corner_label_style = ("sans-serif", 28, FontStyle::Bold)
            .into_text_style(chart.plotting_area())
            .color(&BLACK);
        Self {
            chart,
            major_step: DEFAULT_MAJOR_STEP,
            boundary_style: BLACK.stroke_width(2),
            major_grid_style: RGBColor(185, 190, 198).stroke_width(1),
            axis_names: [None, None, None],
            corner_names: [None, None, None],
            axis_name_style,
            corner_label_style,
            axis_label_offset: 24,
            corner_label_offset: 16,
            draw_axis_names: true,
            draw_corner_names: true,
            draw_grid: true,
            draw_boundary: true,
        }
    }
}

impl<'chart, 'series, 'axis, 'corner, DB: DrawingBackend>
    TernaryMeshConfig<'chart, 'series, 'axis, 'corner, DB>
{
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

    /// Replace both axis and corner text styles for backward compatibility.
    pub fn text_style<'next, S>(
        self,
        style: S,
    ) -> TernaryMeshConfig<'chart, 'series, 'next, 'next, DB>
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
            axis_name_style: text_style.clone(),
            corner_label_style: text_style,
            axis_label_offset: self.axis_label_offset,
            corner_label_offset: self.corner_label_offset,
            draw_axis_names: self.draw_axis_names,
            draw_corner_names: self.draw_corner_names,
            draw_grid: self.draw_grid,
            draw_boundary: self.draw_boundary,
        }
    }

    /// Set the component-axis name style independently from corner labels.
    pub fn axis_name_style<'next, S>(
        self,
        style: S,
    ) -> TernaryMeshConfig<'chart, 'series, 'next, 'corner, DB>
    where
        S: IntoTextStyle<'next>,
    {
        let axis_name_style = style.into_text_style(self.chart.plotting_area());
        TernaryMeshConfig {
            chart: self.chart,
            major_step: self.major_step,
            boundary_style: self.boundary_style,
            major_grid_style: self.major_grid_style,
            axis_names: self.axis_names,
            corner_names: self.corner_names,
            axis_name_style,
            corner_label_style: self.corner_label_style,
            axis_label_offset: self.axis_label_offset,
            corner_label_offset: self.corner_label_offset,
            draw_axis_names: self.draw_axis_names,
            draw_corner_names: self.draw_corner_names,
            draw_grid: self.draw_grid,
            draw_boundary: self.draw_boundary,
        }
    }

    /// Set the pure-component corner-label style independently from axis names.
    pub fn corner_label_style<'next, S>(
        self,
        style: S,
    ) -> TernaryMeshConfig<'chart, 'series, 'axis, 'next, DB>
    where
        S: IntoTextStyle<'next>,
    {
        let corner_label_style = style.into_text_style(self.chart.plotting_area());
        TernaryMeshConfig {
            chart: self.chart,
            major_step: self.major_step,
            boundary_style: self.boundary_style,
            major_grid_style: self.major_grid_style,
            axis_names: self.axis_names,
            corner_names: self.corner_names,
            axis_name_style: self.axis_name_style,
            corner_label_style,
            axis_label_offset: self.axis_label_offset,
            corner_label_offset: self.corner_label_offset,
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

    /// Set the outward axis-name offset in backend pixels.
    pub const fn axis_label_offset(mut self, offset: u32) -> Self {
        self.axis_label_offset = offset;
        self
    }

    /// Set the outward corner-name offset in backend pixels.
    pub const fn corner_label_offset(mut self, offset: u32) -> Self {
        self.corner_label_offset = offset;
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
            draw_axis_names(
                self.chart,
                &self.axis_names,
                &self.axis_name_style,
                self.axis_label_offset,
            )?;
        }
        if self.draw_corner_names {
            draw_corner_names(
                self.chart,
                &self.corner_names,
                &self.corner_label_style,
                self.corner_label_offset,
            )?;
        }

        Ok(())
    }
}

fn draw_corner_names<'series, 'font, DB: DrawingBackend>(
    chart: &TernaryChart<'series, DB>,
    names: &[Option<String>; 3],
    style: &TextStyle<'font>,
    offset: u32,
) -> Result<(), TernaryChartError<DB::ErrorType>> {
    for component in Component::ALL {
        let Some(name) = &names[component.index()] else {
            continue;
        };
        let vertex = chart.geometry.vertex(component);
        if chart.viewport.contains(vertex, chart.tolerance)? {
            let layout = corner_label_layout(chart, component, offset);
            let text = EmptyElement::<_, DB>::at((vertex.x, vertex.y))
                + Text::new(
                    name.clone(),
                    layout.offset,
                    style.clone().pos(layout.position),
                );
            chart.plotting_area().draw(&text)?;
        }
    }
    Ok(())
}

fn draw_axis_names<'series, 'font, DB: DrawingBackend>(
    chart: &TernaryChart<'series, DB>,
    names: &[Option<String>; 3],
    style: &TextStyle<'font>,
    offset: u32,
) -> Result<(), TernaryChartError<DB::ErrorType>> {
    for component in Component::ALL {
        let Some(name) = &names[component.index()] else {
            continue;
        };
        let anchor = axis_name_anchor(chart.geometry, component);
        if chart.viewport.contains(anchor, chart.tolerance)? {
            let layout = axis_label_layout(chart, component, offset);
            if layout.angle.abs() <= 1.0e-12 {
                let text = EmptyElement::<_, DB>::at((anchor.x, anchor.y))
                    + Text::new(
                        name.clone(),
                        layout.offset,
                        style.clone().pos(Pos::new(HPos::Center, VPos::Center)),
                    );
                chart.plotting_area().draw(&text)?;
            } else {
                chart.plotting_area().draw(&RotatedText::new(
                    (anchor.x, anchor.y),
                    name.clone(),
                    style.clone(),
                    layout.angle,
                    layout.offset,
                ))?;
            }
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

fn centroid(geometry: TernaryGeometry) -> TernaryCartesian {
    let [a, b, c] = geometry.vertices();
    TernaryCartesian::new((a.x + b.x + c.x) / 3.0, (a.y + b.y + c.y) / 3.0)
}

#[derive(Clone, Copy)]
struct PixelLabelLayout {
    offset: (i32, i32),
    angle: f64,
    position: Pos,
}

fn outward_offset(from: (i32, i32), toward: (i32, i32), distance: u32) -> (i32, i32) {
    let dx = f64::from(toward.0 - from.0);
    let dy = f64::from(toward.1 - from.1);
    let length = dx.hypot(dy);
    if length == 0.0 {
        return (0, 0);
    }
    (
        (dx / length * f64::from(distance)).round() as i32,
        (dy / length * f64::from(distance)).round() as i32,
    )
}

fn corner_label_layout<DB: DrawingBackend>(
    chart: &TernaryChart<'_, DB>,
    component: Component,
    distance: u32,
) -> PixelLabelLayout {
    let area = chart.plotting_area();
    let centre = area.map_coordinate(&(centroid(chart.geometry).x, centroid(chart.geometry).y));
    let vertex = chart.geometry.vertex(component);
    let vertex_pixel = area.map_coordinate(&(vertex.x, vertex.y));
    let offset = outward_offset(centre, vertex_pixel, distance);
    let position = Pos::new(
        if offset.0 < 0 {
            HPos::Right
        } else if offset.0 > 0 {
            HPos::Left
        } else {
            HPos::Center
        },
        if offset.1 < 0 {
            VPos::Bottom
        } else {
            VPos::Top
        },
    );
    PixelLabelLayout {
        offset,
        angle: 0.0,
        position,
    }
}

fn axis_label_layout<DB: DrawingBackend>(
    chart: &TernaryChart<'_, DB>,
    component: Component,
    distance: u32,
) -> PixelLabelLayout {
    let area = chart.plotting_area();
    let [first_component, second_component] = component.others();
    let first = chart.geometry.vertex(first_component);
    let second = chart.geometry.vertex(second_component);
    let first_pixel = area.map_coordinate(&(first.x, first.y));
    let second_pixel = area.map_coordinate(&(second.x, second.y));
    let anchor = axis_name_anchor(chart.geometry, component);
    let anchor_pixel = area.map_coordinate(&(anchor.x, anchor.y));
    let centre = centroid(chart.geometry);
    let centre_pixel = area.map_coordinate(&(centre.x, centre.y));

    let mut dx = f64::from(second_pixel.0 - first_pixel.0);
    let mut dy = f64::from(second_pixel.1 - first_pixel.1);
    if dx < 0.0 || (dx.abs() <= f64::EPSILON && dy < 0.0) {
        dx = -dx;
        dy = -dy;
    }
    PixelLabelLayout {
        offset: outward_offset(centre_pixel, anchor_pixel, distance),
        angle: dy.atan2(dx),
        position: Pos::new(HPos::Center, VPos::Center),
    }
}

struct RotatedText<'font> {
    anchor: (f64, f64),
    text: String,
    style: TextStyle<'font>,
    angle: f64,
    offset: (i32, i32),
}

impl<'font> RotatedText<'font> {
    fn new(
        anchor: (f64, f64),
        text: String,
        style: TextStyle<'font>,
        angle: f64,
        offset: (i32, i32),
    ) -> Self {
        Self {
            anchor,
            text,
            style,
            angle,
            offset,
        }
    }
}

impl<'element, 'font> PointCollection<'element, (f64, f64)> for &'element RotatedText<'font> {
    type Point = &'element (f64, f64);
    type IntoIter = Once<Self::Point>;

    fn point_iter(self) -> Self::IntoIter {
        once(&self.anchor)
    }
}

impl<'font, DB: DrawingBackend> Drawable<DB> for RotatedText<'font> {
    fn draw<I: Iterator<Item = (i32, i32)>>(
        &self,
        mut positions: I,
        backend: &mut DB,
        _parent_dim: (u32, u32),
    ) -> Result<(), plotters_backend::DrawingErrorKind<DB::ErrorType>> {
        let Some(anchor) = positions.next() else {
            return Ok(());
        };
        let ((min_x, min_y), (max_x, max_y)) = self
            .style
            .font
            .layout_box(&self.text)
            .map_err(|error| plotters_backend::DrawingErrorKind::FontError(Box::new(error)))?;
        let centre_x = f64::from(min_x + max_x) / 2.0;
        let centre_y = f64::from(min_y + max_y) / 2.0;
        let cosine = self.angle.cos();
        let sine = self.angle.sin();
        let base_color = self.style.color;
        let result = self.style.font.draw(&self.text, (0, 0), |x, y, alpha| {
            if alpha == 0.0 {
                return Ok(());
            }
            let local_x = f64::from(x) - centre_x;
            let local_y = f64::from(y) - centre_y;
            let rotated_x = local_x * cosine - local_y * sine;
            let rotated_y = local_x * sine + local_y * cosine;
            let mut color = base_color;
            color.alpha *= f64::from(alpha);
            backend.draw_pixel(
                (
                    anchor.0 + self.offset.0 + rotated_x.round() as i32,
                    anchor.1 + self.offset.1 + rotated_y.round() as i32,
                ),
                color,
            )
        });
        match result {
            Ok(drawing_result) => drawing_result,
            Err(font_error) => Err(plotters_backend::DrawingErrorKind::FontError(Box::new(
                font_error,
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use plotters::drawing::IntoDrawingArea;

    use super::*;
    use crate::{
        EQUILATERAL_TRIANGLE_HEIGHT, TernaryViewport, Tolerance, TriangleEdge, TriangleOrientation,
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

    fn assert_label_vectors_follow_geometry(geometry: TernaryGeometry) {
        let mut buffer = vec![0; 600 * 500 * 3];
        let root = plotters::prelude::BitMapBackend::with_buffer(&mut buffer, (600, 500))
            .into_drawing_area();
        let chart = crate::TernaryChartBuilder::on(&root)
            .geometry(geometry)
            .viewport(TernaryViewport::full(geometry))
            .margin(30)
            .build()
            .unwrap();
        let area = chart.plotting_area();
        let centre = centroid(geometry);
        let centre_pixel = area.map_coordinate(&(centre.x, centre.y));

        for component in Component::ALL {
            let vertex = geometry.vertex(component);
            let vertex_pixel = area.map_coordinate(&(vertex.x, vertex.y));
            let corner = corner_label_layout(&chart, component, 20);
            let radial = (
                vertex_pixel.0 - centre_pixel.0,
                vertex_pixel.1 - centre_pixel.1,
            );
            assert!(radial.0 * corner.offset.0 + radial.1 * corner.offset.1 > 0);

            let [first, second] = component.others();
            let first = geometry.vertex(first);
            let second = geometry.vertex(second);
            let first_pixel = area.map_coordinate(&(first.x, first.y));
            let second_pixel = area.map_coordinate(&(second.x, second.y));
            let edge = (
                f64::from(second_pixel.0 - first_pixel.0),
                f64::from(second_pixel.1 - first_pixel.1),
            );
            let axis = axis_label_layout(&chart, component, 24);
            let baseline = (axis.angle.cos(), axis.angle.sin());
            let cross = edge.0 * baseline.1 - edge.1 * baseline.0;
            assert!(cross.abs() <= edge.0.hypot(edge.1) * 0.01);

            let anchor = axis_name_anchor(geometry, component);
            let anchor_pixel = area.map_coordinate(&(anchor.x, anchor.y));
            let radial = (
                anchor_pixel.0 - centre_pixel.0,
                anchor_pixel.1 - centre_pixel.1,
            );
            assert!(radial.0 * axis.offset.0 + radial.1 * axis.offset.1 > 0);
        }
    }

    #[test]
    fn corner_and_axis_label_layouts_are_outward_and_edge_parallel() {
        assert_label_vectors_follow_geometry(TernaryGeometry::default());
        assert_label_vectors_follow_geometry(TernaryGeometry::new(
            TriangleOrientation::Down,
            VertexOrder::default(),
        ));
        assert_label_vectors_follow_geometry(TernaryGeometry::new(
            TriangleOrientation::Up,
            VertexOrder::new(Component::C, Component::A, Component::B).unwrap(),
        ));
    }

    #[test]
    fn default_corner_style_is_at_least_as_prominent_as_axis_style() {
        let mut buffer = vec![0; 400 * 320 * 3];
        let root = plotters::prelude::BitMapBackend::with_buffer(&mut buffer, (400, 320))
            .into_drawing_area();
        let mut chart = crate::TernaryChartBuilder::on(&root).build().unwrap();
        let mesh = TernaryMeshConfig::new(&mut chart);
        assert!(mesh.corner_label_style.font.get_size() >= mesh.axis_name_style.font.get_size());
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
