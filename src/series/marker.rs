//! Portable scientific marker geometry and Plotters drawing elements.
//!
//! Marker geometry is normalized around `(0, 0)` before a concrete
//! [`MarkerElement`] maps it to a Plotters backend anchor.

use std::fmt;

use plotters::backend::DrawingBackend;
use plotters::element::{Drawable, PointCollection};
use plotters::style::{Color, RGBAColor, ShapeStyle};
use plotters_backend::{BackendCoord, DrawingErrorKind};

const MIN_SIDES: u8 = 3;
const MAX_SIDES: u8 = 16;
const CIRCLE_SEGMENTS: usize = 32;
const EPS: f64 = 1.0e-10;

/// Policy governing whether a point marker centre must be in the logical
/// ternary viewport. `Centre` is intentionally not marker-bounds clipping.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum MarkerClipMode {
    /// Draw only when the marker centre is inside or on the viewport.
    #[default]
    Centre,
    /// Draw regardless of viewport containment; Plotters may truncate it.
    None,
}

/// Direction used by radial marker partitions.
///
/// `0 degrees` points right and positive angles turn counter-clockwise in ordinary
/// mathematical coordinates (visually toward the top of backend pixel space).
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum SweepDirection {
    /// Advance toward increasing angles.
    #[default]
    CounterClockwise,
    /// Advance toward decreasing angles.
    Clockwise,
}

/// A built-in scientific marker outline.
///
/// `Plus` is the orthogonal `+` symbol; `Cross` is the diagonal `x` symbol.
/// `Triangle` is retained as the legacy spelling for [`MarkerShape::TriangleUp`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MarkerShape {
    /// A circle, approximated by 32 shared logical vertices.
    #[default]
    Circle,
    /// An ellipse with positive horizontal-to-vertical aspect ratio.
    Ellipse { aspect_ratio: f64 },
    /// An axis-aligned square.
    Square,
    /// A rectangle with positive horizontal-to-vertical aspect ratio.
    Rectangle { aspect_ratio: f64 },
    /// A square with a finite corner ratio in `0.0..=0.5`.
    RoundedSquare { corner_ratio: f64 },
    /// A 45-degree square.
    Diamond,
    /// Historical alias for [`MarkerShape::TriangleUp`].
    Triangle,
    /// An upward-pointing triangle.
    TriangleUp,
    /// A downward-pointing triangle.
    TriangleDown,
    /// A left-pointing triangle.
    TriangleLeft,
    /// A right-pointing triangle.
    TriangleRight,
    /// A regular polygon with intrinsic rotation.
    RegularPolygon { sides: u8, rotation_deg: f64 },
    /// A star with alternating outer and inner vertices.
    Star {
        points: u8,
        inner_ratio: f64,
        rotation_deg: f64,
    },
    /// Stroke-only orthogonal `+`.
    Plus,
    /// Stroke-only diagonal `x`.
    Cross,
    /// Stroke-only multi-arm symbol.
    Asterisk { arms: u8 },
}

impl MarkerShape {
    /// A pentagon with no intrinsic rotation.
    pub const fn pentagon() -> Self {
        Self::RegularPolygon {
            sides: 5,
            rotation_deg: 0.0,
        }
    }
    /// A hexagon with no intrinsic rotation.
    pub const fn hexagon() -> Self {
        Self::RegularPolygon {
            sides: 6,
            rotation_deg: 0.0,
        }
    }
    /// An octagon with no intrinsic rotation.
    pub const fn octagon() -> Self {
        Self::RegularPolygon {
            sides: 8,
            rotation_deg: 0.0,
        }
    }
    /// A four-point star with the conventional inner ratio.
    pub const fn star4() -> Self {
        Self::Star {
            points: 4,
            inner_ratio: 0.45,
            rotation_deg: 0.0,
        }
    }
    /// A five-point star with the conventional inner ratio.
    pub const fn star5() -> Self {
        Self::Star {
            points: 5,
            inner_ratio: 0.45,
            rotation_deg: 0.0,
        }
    }
    /// A six-point star with the conventional inner ratio.
    pub const fn star6() -> Self {
        Self::Star {
            points: 6,
            inner_ratio: 0.45,
            rotation_deg: 0.0,
        }
    }
    /// An eight-point star with the conventional inner ratio.
    pub const fn star8() -> Self {
        Self::Star {
            points: 8,
            inner_ratio: 0.45,
            rotation_deg: 0.0,
        }
    }
    /// Whether the shape can carry a fill or partition.
    pub const fn is_fillable(self) -> bool {
        !matches!(self, Self::Plus | Self::Cross | Self::Asterisk { .. })
    }
    fn canonical(self) -> Self {
        if matches!(self, Self::Triangle) {
            Self::TriangleUp
        } else {
            self
        }
    }
}

/// A validated shape plus an additional rotation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarkerGeometry {
    shape: MarkerShape,
    rotation_deg: f64,
}

impl MarkerGeometry {
    /// Construct validated geometry.
    pub fn new(shape: MarkerShape) -> Result<Self, MarkerError> {
        let geometry = Self {
            shape: shape.canonical(),
            rotation_deg: 0.0,
        };
        geometry.validate()?;
        Ok(geometry)
    }
    /// Add a finite counter-clockwise rotation in degrees.
    pub fn rotated(mut self, rotation_deg: f64) -> Result<Self, MarkerError> {
        self.rotation_deg = rotation_deg;
        self.validate()?;
        Ok(self)
    }
    /// Return the canonical shape.
    pub const fn shape(self) -> MarkerShape {
        self.shape
    }
    /// Return the additional rotation.
    pub const fn rotation_deg(self) -> f64 {
        self.rotation_deg
    }
    /// Whether this geometry supports fills.
    pub const fn is_fillable(self) -> bool {
        self.shape.is_fillable()
    }

    /// Return a normalized, centred outer outline.
    pub fn outline(self) -> Result<Vec<LocalPoint>, MarkerError> {
        self.validate()?;
        let mut points = match self.shape {
            MarkerShape::Circle => ellipse(1.0),
            MarkerShape::Ellipse { aspect_ratio } => ellipse(aspect_ratio),
            MarkerShape::Square => rectangle(1.0),
            MarkerShape::Rectangle { aspect_ratio } => rectangle(aspect_ratio),
            MarkerShape::RoundedSquare { corner_ratio } => rounded_square(corner_ratio),
            MarkerShape::Diamond => vec![p(0.0, -1.0), p(1.0, 0.0), p(0.0, 1.0), p(-1.0, 0.0)],
            MarkerShape::TriangleUp => triangle(0.0),
            MarkerShape::TriangleRight => triangle(270.0),
            MarkerShape::TriangleDown => triangle(180.0),
            MarkerShape::TriangleLeft => triangle(90.0),
            MarkerShape::RegularPolygon {
                sides,
                rotation_deg,
            } => regular(sides, rotation_deg),
            MarkerShape::Star {
                points,
                inner_ratio,
                rotation_deg,
            } => star(points, inner_ratio, rotation_deg),
            MarkerShape::Plus | MarkerShape::Cross | MarkerShape::Asterisk { .. } => Vec::new(),
            MarkerShape::Triangle => unreachable!("legacy triangle is canonicalized"),
        };
        for point in &mut points {
            *point = rotate(*point, self.rotation_deg);
        }
        Ok(normalize(points))
    }

    /// Return stroke-only local segments.
    pub fn stroke_segments(self) -> Result<Vec<LocalSegment>, MarkerError> {
        self.validate()?;
        let segments = match self.shape {
            MarkerShape::Plus => vec![
                seg(p(-1.0, 0.0), p(1.0, 0.0)),
                seg(p(0.0, -1.0), p(0.0, 1.0)),
            ],
            MarkerShape::Cross => vec![
                seg(p(-1.0, -1.0), p(1.0, 1.0)),
                seg(p(-1.0, 1.0), p(1.0, -1.0)),
            ],
            MarkerShape::Asterisk { arms } => (0..arms)
                .map(|i| {
                    let angle = f64::from(i) * 180.0 / f64::from(arms);
                    seg(rotate(p(-1.0, 0.0), angle), rotate(p(1.0, 0.0), angle))
                })
                .collect(),
            _ => Vec::new(),
        };
        Ok(segments
            .into_iter()
            .map(|s| {
                seg(
                    rotate(s.start, self.rotation_deg),
                    rotate(s.end, self.rotation_deg),
                )
            })
            .collect())
    }

    fn validate(self) -> Result<(), MarkerError> {
        finite_rotation(self.rotation_deg)?;
        match self.shape {
            MarkerShape::Ellipse { aspect_ratio } | MarkerShape::Rectangle { aspect_ratio } => {
                if !aspect_ratio.is_finite() || aspect_ratio <= 0.0 {
                    return Err(MarkerError::InvalidAspectRatio {
                        value: aspect_ratio,
                    });
                }
            }
            MarkerShape::RoundedSquare { corner_ratio } => {
                if !corner_ratio.is_finite() || !(0.0..=0.5).contains(&corner_ratio) {
                    return Err(MarkerError::InvalidCornerRatio {
                        value: corner_ratio,
                    });
                }
            }
            MarkerShape::RegularPolygon {
                sides,
                rotation_deg,
            } => {
                count(sides, MarkerError::InvalidPolygonSides { sides })?;
                finite_rotation(rotation_deg)?;
            }
            MarkerShape::Star {
                points,
                inner_ratio,
                rotation_deg,
            } => {
                count(points, MarkerError::InvalidStarPoints { points })?;
                if !inner_ratio.is_finite() || !(0.0..1.0).contains(&inner_ratio) {
                    return Err(MarkerError::InvalidStarInnerRatio { value: inner_ratio });
                }
                finite_rotation(rotation_deg)?;
            }
            MarkerShape::Asterisk { arms } => {
                count(arms, MarkerError::InvalidAsteriskArms { arms })?
            }
            _ => {}
        }
        Ok(())
    }
}

fn count(value: u8, error: MarkerError) -> Result<(), MarkerError> {
    if (MIN_SIDES..=MAX_SIDES).contains(&value) {
        Ok(())
    } else {
        Err(error)
    }
}
fn finite_rotation(value: f64) -> Result<(), MarkerError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(MarkerError::NonFiniteRotation { value })
    }
}

/// A colored, weighted marker slice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MarkerSlice {
    /// Positive finite relative weight.
    pub weight: f64,
    /// Fill colour.
    pub color: RGBAColor,
}

impl MarkerSlice {
    /// Construct from a normal Plotters colour.
    pub fn new<C: Color>(weight: f64, color: C) -> Self {
        Self {
            weight,
            color: color.to_rgba(),
        }
    }
}

/// Interior partition geometry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarkerPartition {
    /// Sectors begin at `start_angle_deg` and follow `direction`.
    Radial {
        start_angle_deg: f64,
        direction: SweepDirection,
    },
    /// Slice 0 upper, slice 1 lower.
    Horizontal,
    /// Slice 0 left, slice 1 right.
    Vertical,
    /// Slice 0 upper/right of visual `/`, slice 1 opposite.
    DiagonalForward,
    /// Slice 0 upper/left of visual `\\`, slice 1 opposite.
    DiagonalBackward,
    /// Upper-right, lower-right, lower-left, upper-left, rotated counter-clockwise.
    Quadrants { rotation_deg: f64 },
}

/// Interior marker fill treatment.
#[derive(Clone, Debug, PartialEq)]
pub enum MarkerFill {
    /// No fill; a visible edge is required.
    Empty,
    /// One solid fill.
    Solid { color: RGBAColor },
    /// Independently coloured regions and an optional divider style.
    Partitioned {
        partition: MarkerPartition,
        slices: Vec<MarkerSlice>,
        divider: Option<ShapeStyle>,
    },
}

/// Complete validated marker appearance.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkerStyle {
    /// Outline and extra rotation.
    pub geometry: MarkerGeometry,
    /// Interior treatment.
    pub fill: MarkerFill,
    /// Common outer edge, drawn only after fill and dividers.
    pub edge: Option<ShapeStyle>,
}

impl MarkerStyle {
    /// Build an empty contour marker.
    pub fn empty<S: Into<ShapeStyle>>(shape: MarkerShape, edge: S) -> Result<Self, MarkerError> {
        Self::new(
            MarkerGeometry::new(shape)?,
            MarkerFill::Empty,
            Some(edge.into()),
        )
    }
    /// Build an independently filled and edged marker.
    pub fn solid<C: Color, S: Into<ShapeStyle>>(
        shape: MarkerShape,
        color: C,
        edge: S,
    ) -> Result<Self, MarkerError> {
        Self::new(
            MarkerGeometry::new(shape)?,
            MarkerFill::Solid {
                color: color.to_rgba(),
            },
            Some(edge.into()),
        )
    }
    /// Build a fully filled FactSage-style marker with the same edge and fill colour.
    pub fn fact_sage<C: Color>(shape: MarkerShape, color: C) -> Result<Self, MarkerError> {
        let color = color.to_rgba();
        Self::new(
            MarkerGeometry::new(shape)?,
            MarkerFill::Solid { color },
            Some(ShapeStyle {
                color,
                filled: false,
                stroke_width: 1,
            }),
        )
    }
    /// Build a partitioned marker through the common validated path.
    pub fn partitioned(
        shape: MarkerShape,
        partition: MarkerPartition,
        slices: Vec<MarkerSlice>,
        divider: Option<ShapeStyle>,
        edge: Option<ShapeStyle>,
    ) -> Result<Self, MarkerError> {
        Self::new(
            MarkerGeometry::new(shape)?,
            MarkerFill::Partitioned {
                partition,
                slices,
                divider,
            },
            edge,
        )
    }
    /// Build two to four equal radial sectors, beginning at 90 degrees and sweeping clockwise.
    pub fn equal_radial<C: Color, I: IntoIterator<Item = C>>(
        shape: MarkerShape,
        colors: I,
        edge: Option<ShapeStyle>,
    ) -> Result<Self, MarkerError> {
        Self::partitioned(
            shape,
            MarkerPartition::Radial {
                start_angle_deg: 90.0,
                direction: SweepDirection::Clockwise,
            },
            colors
                .into_iter()
                .map(|color| MarkerSlice::new(1.0, color))
                .collect(),
            None,
            edge,
        )
    }
    /// Build weighted radial sectors.
    pub fn weighted_radial(
        shape: MarkerShape,
        start_angle_deg: f64,
        direction: SweepDirection,
        slices: Vec<MarkerSlice>,
        divider: Option<ShapeStyle>,
        edge: Option<ShapeStyle>,
    ) -> Result<Self, MarkerError> {
        Self::partitioned(
            shape,
            MarkerPartition::Radial {
                start_angle_deg,
                direction,
            },
            slices,
            divider,
            edge,
        )
    }
    /// Build a horizontal two-colour marker (upper first).
    pub fn horizontal<C0: Color, C1: Color>(
        shape: MarkerShape,
        first: C0,
        second: C1,
        edge: Option<ShapeStyle>,
    ) -> Result<Self, MarkerError> {
        Self::two(shape, MarkerPartition::Horizontal, first, second, edge)
    }
    /// Build a vertical two-colour marker (left first).
    pub fn vertical<C0: Color, C1: Color>(
        shape: MarkerShape,
        first: C0,
        second: C1,
        edge: Option<ShapeStyle>,
    ) -> Result<Self, MarkerError> {
        Self::two(shape, MarkerPartition::Vertical, first, second, edge)
    }
    /// Build a visual `/` two-colour marker.
    pub fn diagonal_forward<C0: Color, C1: Color>(
        shape: MarkerShape,
        first: C0,
        second: C1,
        edge: Option<ShapeStyle>,
    ) -> Result<Self, MarkerError> {
        Self::two(shape, MarkerPartition::DiagonalForward, first, second, edge)
    }
    /// Build a visual `\\` two-colour marker.
    pub fn diagonal_backward<C0: Color, C1: Color>(
        shape: MarkerShape,
        first: C0,
        second: C1,
        edge: Option<ShapeStyle>,
    ) -> Result<Self, MarkerError> {
        Self::two(
            shape,
            MarkerPartition::DiagonalBackward,
            first,
            second,
            edge,
        )
    }
    /// Build four equal quadrants in upper-right, lower-right, lower-left, upper-left order.
    pub fn quadrants<C: Color, I: IntoIterator<Item = C>>(
        shape: MarkerShape,
        rotation_deg: f64,
        colors: I,
        edge: Option<ShapeStyle>,
    ) -> Result<Self, MarkerError> {
        Self::partitioned(
            shape,
            MarkerPartition::Quadrants { rotation_deg },
            colors
                .into_iter()
                .map(|color| MarkerSlice::new(1.0, color))
                .collect(),
            None,
            edge,
        )
    }
    /// Construct a fully explicit marker style.
    pub fn new(
        geometry: MarkerGeometry,
        fill: MarkerFill,
        edge: Option<ShapeStyle>,
    ) -> Result<Self, MarkerError> {
        let style = Self {
            geometry,
            fill,
            edge,
        };
        style.validate()?;
        Ok(style)
    }
    /// Validate this style without creating a drawable element.
    pub fn validate(&self) -> Result<(), MarkerError> {
        self.geometry.validate()?;
        if !self.geometry.is_fillable() {
            if !matches!(self.fill, MarkerFill::Empty) {
                return Err(MarkerError::FillUnsupportedForStrokeOnly {
                    shape: self.geometry.shape(),
                });
            }
            return self
                .edge
                .is_some()
                .then_some(())
                .ok_or(MarkerError::EmptyMarkerWithoutEdge);
        }
        match &self.fill {
            MarkerFill::Empty => self
                .edge
                .is_some()
                .then_some(())
                .ok_or(MarkerError::EmptyMarkerWithoutEdge),
            MarkerFill::Solid { .. } => Ok(()),
            MarkerFill::Partitioned {
                partition, slices, ..
            } => validate_partition(*partition, slices),
        }
    }
    /// Prepare the shared backend-neutral draw plan.
    pub fn drawing(&self) -> Result<MarkerDrawing, MarkerError> {
        self.validate()?;
        if !self.geometry.is_fillable() {
            return Ok(MarkerDrawing {
                outline: Vec::new(),
                fills: Vec::new(),
                dividers: Vec::new(),
                divider_style: None,
                edge: self.edge,
                strokes: self.geometry.stroke_segments()?,
            });
        }
        let outline = self.geometry.outline()?;
        let (fills, dividers, divider_style) = match &self.fill {
            MarkerFill::Empty => (Vec::new(), Vec::new(), None),
            MarkerFill::Solid { color } => (
                vec![MarkerFillPolygon {
                    points: outline.clone(),
                    color: *color,
                }],
                Vec::new(),
                None,
            ),
            MarkerFill::Partitioned {
                partition,
                slices,
                divider,
            } => (
                partition_fills(&outline, *partition, slices),
                divider.as_ref().map_or_else(Vec::new, |_| {
                    partition_dividers(&outline, *partition, slices)
                }),
                *divider,
            ),
        };
        Ok(MarkerDrawing {
            outline,
            fills,
            dividers,
            divider_style,
            edge: self.edge,
            strokes: Vec::new(),
        })
    }
    /// Scale only edge and divider widths for a high-resolution bitmap pass.
    pub fn scaled(&self, factor: u32) -> Self {
        let mut result = self.clone();
        if let Some(edge) = result.edge.as_mut() {
            edge.stroke_width = edge.stroke_width.saturating_mul(factor);
        }
        if let MarkerFill::Partitioned { divider, .. } = &mut result.fill
            && let Some(divider) = divider.as_mut()
        {
            divider.stroke_width = divider.stroke_width.saturating_mul(factor);
        }
        result
    }
    pub(crate) fn from_legacy(shape: MarkerShape, style: ShapeStyle) -> Result<Self, MarkerError> {
        let geometry = MarkerGeometry::new(shape)?;
        if geometry.is_fillable() {
            let fill = if style.filled {
                MarkerFill::Solid { color: style.color }
            } else {
                MarkerFill::Empty
            };
            Self::new(geometry, fill, (!style.filled).then_some(style))
        } else {
            Self::new(geometry, MarkerFill::Empty, Some(style))
        }
    }
    fn two<C0: Color, C1: Color>(
        shape: MarkerShape,
        partition: MarkerPartition,
        first: C0,
        second: C1,
        edge: Option<ShapeStyle>,
    ) -> Result<Self, MarkerError> {
        Self::partitioned(
            shape,
            partition,
            vec![MarkerSlice::new(1.0, first), MarkerSlice::new(1.0, second)],
            None,
            edge,
        )
    }
}

fn validate_partition(
    partition: MarkerPartition,
    slices: &[MarkerSlice],
) -> Result<(), MarkerError> {
    let required = match partition {
        MarkerPartition::Radial {
            start_angle_deg, ..
        } => {
            finite_rotation(start_angle_deg)?;
            None
        }
        MarkerPartition::Quadrants { rotation_deg } => {
            finite_rotation(rotation_deg)?;
            Some(4)
        }
        _ => Some(2),
    };
    if slices.is_empty() {
        return Err(MarkerError::EmptyPartition);
    }
    if slices.len() > 4 {
        return Err(MarkerError::TooManySlices {
            count: slices.len(),
            maximum: 4,
        });
    }
    if let Some(expected) = required
        && slices.len() != expected
    {
        return Err(MarkerError::InvalidSliceCount {
            expected,
            actual: slices.len(),
        });
    }
    for (index, slice) in slices.iter().enumerate() {
        if !slice.weight.is_finite() || slice.weight <= 0.0 {
            return Err(MarkerError::InvalidSliceWeight {
                index,
                weight: slice.weight,
            });
        }
    }
    Ok(())
}

/// A normalized backend-neutral marker point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalPoint {
    pub x: f64,
    pub y: f64,
}
/// A normalized local line segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalSegment {
    pub start: LocalPoint,
    pub end: LocalPoint,
}
/// A coloured local polygon.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkerFillPolygon {
    pub points: Vec<LocalPoint>,
    pub color: RGBAColor,
}
/// Prepared local marker drawing order: fills, dividers, one outer edge.
#[derive(Clone, Debug, PartialEq)]
pub struct MarkerDrawing {
    pub outline: Vec<LocalPoint>,
    pub fills: Vec<MarkerFillPolygon>,
    pub dividers: Vec<LocalSegment>,
    pub divider_style: Option<ShapeStyle>,
    pub edge: Option<ShapeStyle>,
    pub strokes: Vec<LocalSegment>,
}

/// A concrete owned Plotters element for one scientific marker.
///
/// The anchor is always the visual centre, whether it is a logical ternary
/// point or a backend-pixel legend coordinate.
pub struct MarkerElement<Coord> {
    anchor: Coord,
    size: u32,
    drawing: MarkerDrawing,
}
impl<Coord> MarkerElement<Coord> {
    /// Validate and build an element.
    pub fn new(anchor: Coord, size: u32, style: MarkerStyle) -> Result<Self, MarkerError> {
        if size == 0 {
            return Err(MarkerError::InvalidSize { size });
        }
        Ok(Self {
            anchor,
            size,
            drawing: style.drawing()?,
        })
    }
    /// Return backend-pixel half-size.
    pub const fn size(&self) -> u32 {
        self.size
    }
    /// Inspect local geometry.
    pub const fn drawing(&self) -> &MarkerDrawing {
        &self.drawing
    }
}
impl<'a, Coord> PointCollection<'a, Coord> for &'a MarkerElement<Coord> {
    type Point = &'a Coord;
    type IntoIter = std::iter::Once<&'a Coord>;
    fn point_iter(self) -> Self::IntoIter {
        std::iter::once(&self.anchor)
    }
}
impl<Coord, DB: DrawingBackend> Drawable<DB> for MarkerElement<Coord> {
    fn draw<I: Iterator<Item = BackendCoord>>(
        &self,
        mut points: I,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        let Some(anchor) = points.next() else {
            return Ok(());
        };
        let map = |point: LocalPoint| backend_point(anchor, point, self.size);
        for fill in &self.drawing.fills {
            backend.fill_polygon(
                fill.points.iter().copied().map(map),
                &fill.color.to_backend_color(),
            )?;
        }
        if let Some(style) = self.drawing.divider_style {
            for divider in &self.drawing.dividers {
                backend.draw_path([map(divider.start), map(divider.end)], &style)?;
            }
        }
        if let Some(style) = self.drawing.edge {
            if let Some(first) = self.drawing.outline.first().copied() {
                let mut path: Vec<_> = self.drawing.outline.iter().copied().map(map).collect();
                path.push(map(first));
                backend.draw_path(path, &style)?;
            }
            for stroke in &self.drawing.strokes {
                backend.draw_path([map(stroke.start), map(stroke.end)], &style)?;
            }
        }
        Ok(())
    }
}
fn backend_point(anchor: BackendCoord, point: LocalPoint, size: u32) -> BackendCoord {
    (
        round(f64::from(anchor.0) + point.x * f64::from(size)),
        round(f64::from(anchor.1) + point.y * f64::from(size)),
    )
}
fn round(value: f64) -> i32 {
    value
        .round()
        .clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32
}

/// Marker validation error.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum MarkerError {
    InvalidSize { size: u32 },
    InvalidAspectRatio { value: f64 },
    InvalidCornerRatio { value: f64 },
    InvalidPolygonSides { sides: u8 },
    InvalidStarPoints { points: u8 },
    InvalidStarInnerRatio { value: f64 },
    InvalidAsteriskArms { arms: u8 },
    NonFiniteRotation { value: f64 },
    FillUnsupportedForStrokeOnly { shape: MarkerShape },
    EmptyMarkerWithoutEdge,
    EmptyPartition,
    TooManySlices { count: usize, maximum: usize },
    InvalidSliceCount { expected: usize, actual: usize },
    InvalidSliceWeight { index: usize, weight: f64 },
}
impl fmt::Display for MarkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSize { size } => {
                write!(f, "marker size must be greater than zero: {size}")
            }
            Self::InvalidAspectRatio { value } => write!(
                f,
                "marker aspect ratio must be finite and positive: {value:?}"
            ),
            Self::InvalidCornerRatio { value } => write!(
                f,
                "rounded-square corner ratio must be in 0.0..=0.5: {value:?}"
            ),
            Self::InvalidPolygonSides { sides } => write!(
                f,
                "regular polygon sides must be in {MIN_SIDES}..={MAX_SIDES}: {sides}"
            ),
            Self::InvalidStarPoints { points } => write!(
                f,
                "star points must be in {MIN_SIDES}..={MAX_SIDES}: {points}"
            ),
            Self::InvalidStarInnerRatio { value } => {
                write!(f, "star inner ratio must be in (0, 1): {value:?}")
            }
            Self::InvalidAsteriskArms { arms } => write!(
                f,
                "asterisk arms must be in {MIN_SIDES}..={MAX_SIDES}: {arms}"
            ),
            Self::NonFiniteRotation { value } => {
                write!(f, "marker rotation must be finite: {value:?}")
            }
            Self::FillUnsupportedForStrokeOnly { shape } => {
                write!(f, "{shape:?} is stroke-only and cannot be filled")
            }
            Self::EmptyMarkerWithoutEdge => {
                write!(f, "empty or stroke-only marker requires an edge")
            }
            Self::EmptyPartition => write!(f, "marker partition requires slices"),
            Self::TooManySlices { count, maximum } => write!(
                f,
                "marker partition supports at most {maximum} slices: {count}"
            ),
            Self::InvalidSliceCount { expected, actual } => write!(
                f,
                "marker partition requires exactly {expected} slices: {actual}"
            ),
            Self::InvalidSliceWeight { index, weight } => write!(
                f,
                "marker slice {index} weight must be finite and positive: {weight:?}"
            ),
        }
    }
}
impl std::error::Error for MarkerError {}

fn p(x: f64, y: f64) -> LocalPoint {
    LocalPoint { x, y }
}
fn seg(start: LocalPoint, end: LocalPoint) -> LocalSegment {
    LocalSegment { start, end }
}
fn polar(angle: f64) -> LocalPoint {
    let radians = angle.to_radians();
    p(radians.cos(), -radians.sin())
}
fn rotate(point: LocalPoint, angle: f64) -> LocalPoint {
    let radians = angle.to_radians();
    let (sin, cos) = radians.sin_cos();
    p(
        point.x * cos + point.y * sin,
        -point.x * sin + point.y * cos,
    )
}
fn ellipse(aspect: f64) -> Vec<LocalPoint> {
    let (x, y) = if aspect >= 1.0 {
        (1.0, aspect.recip())
    } else {
        (aspect, 1.0)
    };
    (0..CIRCLE_SEGMENTS)
        .map(|index| {
            let q = polar(360.0 * index as f64 / CIRCLE_SEGMENTS as f64);
            p(q.x * x, q.y * y)
        })
        .collect()
}
fn rectangle(aspect: f64) -> Vec<LocalPoint> {
    let (x, y) = if aspect >= 1.0 {
        (1.0, aspect.recip())
    } else {
        (aspect, 1.0)
    };
    vec![p(-x, -y), p(x, -y), p(x, y), p(-x, y)]
}
fn rounded_square(ratio: f64) -> Vec<LocalPoint> {
    if ratio <= EPS {
        return rectangle(1.0);
    }
    let c = 1.0 - ratio;
    [(c, -c, -90.0), (c, c, 0.0), (-c, c, 90.0), (-c, -c, 180.0)]
        .into_iter()
        .flat_map(|(x, y, start)| {
            (0..5).map(move |index| {
                let q = polar(start + 90.0 * index as f64 / 4.0);
                p(x + ratio * q.x, y + ratio * q.y)
            })
        })
        .collect()
}
fn triangle(rotation: f64) -> Vec<LocalPoint> {
    [p(0.0, -1.0), p(1.0, 1.0), p(-1.0, 1.0)]
        .into_iter()
        .map(|point| rotate(point, rotation))
        .collect()
}
fn regular(sides: u8, rotation: f64) -> Vec<LocalPoint> {
    (0..sides)
        .map(|index| polar(-90.0 + rotation + 360.0 * f64::from(index) / f64::from(sides)))
        .collect()
}
fn star(points: u8, inner: f64, rotation: f64) -> Vec<LocalPoint> {
    (0..usize::from(points) * 2)
        .map(|index| {
            let q = polar(-90.0 + rotation + 180.0 * index as f64 / f64::from(points));
            let radius = if index % 2 == 0 { 1.0 } else { inner };
            p(q.x * radius, q.y * radius)
        })
        .collect()
}
fn normalize(mut points: Vec<LocalPoint>) -> Vec<LocalPoint> {
    if points.is_empty() {
        return points;
    }
    let min_x = points.iter().map(|q| q.x).fold(f64::INFINITY, f64::min);
    let max_x = points.iter().map(|q| q.x).fold(f64::NEG_INFINITY, f64::max);
    let min_y = points.iter().map(|q| q.y).fold(f64::INFINITY, f64::min);
    let max_y = points.iter().map(|q| q.y).fold(f64::NEG_INFINITY, f64::max);
    let cx = (min_x + max_x) / 2.0;
    let cy = (min_y + max_y) / 2.0;
    let extent = ((max_x - min_x) / 2.0).max((max_y - min_y) / 2.0);
    for q in &mut points {
        q.x = (q.x - cx) / extent;
        q.y = (q.y - cy) / extent;
    }
    points
}

fn partition_fills(
    outline: &[LocalPoint],
    partition: MarkerPartition,
    slices: &[MarkerSlice],
) -> Vec<MarkerFillPolygon> {
    match partition {
        MarkerPartition::Radial {
            start_angle_deg,
            direction,
        } => radial_fills(outline, start_angle_deg, direction, slices),
        MarkerPartition::Horizontal => two_fills(outline, slices, |q| -q.y),
        MarkerPartition::Vertical => two_fills(outline, slices, |q| -q.x),
        MarkerPartition::DiagonalForward => two_fills(outline, slices, |q| -(q.x + q.y)),
        MarkerPartition::DiagonalBackward => two_fills(outline, slices, |q| q.x - q.y),
        MarkerPartition::Quadrants { rotation_deg } => {
            quadrant_fills(outline, rotation_deg, slices)
        }
    }
}
fn radial_fills(
    outline: &[LocalPoint],
    start: f64,
    direction: SweepDirection,
    slices: &[MarkerSlice],
) -> Vec<MarkerFillPolygon> {
    let total: f64 = slices.iter().map(|slice| slice.weight).sum();
    let sign = if direction == SweepDirection::CounterClockwise {
        1.0
    } else {
        -1.0
    };
    let mut current = start;
    let mut output = Vec::new();
    for slice in slices {
        let sweep = 360.0 * slice.weight / total;
        let next = current + sign * sweep;
        if sweep >= 360.0 - EPS {
            output.push(MarkerFillPolygon {
                points: outline.to_vec(),
                color: slice.color,
            });
        } else {
            let (first, last) = if direction == SweepDirection::CounterClockwise {
                (current, next)
            } else {
                (next, current)
            };
            output.extend(
                fan(outline)
                    .into_iter()
                    .flat_map(|triangle| sector_clip(&triangle, first, last))
                    .filter(|points| points.len() >= 3 && area(points).abs() > EPS)
                    .map(|points| MarkerFillPolygon {
                        points,
                        color: slice.color,
                    }),
            );
        }
        current = next;
    }
    output
}
fn two_fills<F: Fn(LocalPoint) -> f64 + Copy>(
    outline: &[LocalPoint],
    slices: &[MarkerSlice],
    distance: F,
) -> Vec<MarkerFillPolygon> {
    fan(outline)
        .into_iter()
        .flat_map(|triangle| {
            [(slices[0].color, 1.0), (slices[1].color, -1.0)]
                .into_iter()
                .filter_map(move |(color, sign)| {
                    let points = clip_half(&triangle, |q| sign * distance(q));
                    (points.len() >= 3 && area(&points).abs() > EPS)
                        .then_some(MarkerFillPolygon { points, color })
                })
        })
        .collect()
}
fn quadrant_fills(
    outline: &[LocalPoint],
    rotation: f64,
    slices: &[MarkerSlice],
) -> Vec<MarkerFillPolygon> {
    fan(outline)
        .into_iter()
        .flat_map(|triangle| {
            (0..4).filter_map(move |index| {
                let mut clipped = triangle.clone();
                let signs = match index {
                    0 => [(1.0, 0.0), (0.0, -1.0)],
                    1 => [(1.0, 0.0), (0.0, 1.0)],
                    2 => [(-1.0, 0.0), (0.0, 1.0)],
                    _ => [(-1.0, 0.0), (0.0, -1.0)],
                };
                for (x, y) in signs {
                    clipped = clip_half(&clipped, |q| {
                        let q = rotate(q, -rotation);
                        if x != 0.0 { x * q.x } else { y * q.y }
                    });
                }
                (clipped.len() >= 3 && area(&clipped).abs() > EPS).then_some(MarkerFillPolygon {
                    points: clipped,
                    color: slices[index].color,
                })
            })
        })
        .collect()
}
fn partition_dividers(
    outline: &[LocalPoint],
    partition: MarkerPartition,
    slices: &[MarkerSlice],
) -> Vec<LocalSegment> {
    match partition {
        MarkerPartition::Radial {
            start_angle_deg,
            direction,
        } => {
            let total: f64 = slices.iter().map(|slice| slice.weight).sum();
            let sign = if direction == SweepDirection::CounterClockwise {
                1.0
            } else {
                -1.0
            };
            let mut current = start_angle_deg;
            slices
                .iter()
                .take(slices.len().saturating_sub(1))
                .filter_map(|slice| {
                    current += sign * 360.0 * slice.weight / total;
                    ray(outline, current).map(|end| seg(p(0.0, 0.0), end))
                })
                .collect()
        }
        MarkerPartition::Horizontal => line_inside(outline, p(-1.0, 0.0), p(1.0, 0.0)),
        MarkerPartition::Vertical => line_inside(outline, p(0.0, -1.0), p(0.0, 1.0)),
        MarkerPartition::DiagonalForward => line_inside(outline, p(-1.0, 1.0), p(1.0, -1.0)),
        MarkerPartition::DiagonalBackward => line_inside(outline, p(-1.0, -1.0), p(1.0, 1.0)),
        MarkerPartition::Quadrants { rotation_deg } => {
            let h = seg(
                rotate(p(-1.0, 0.0), rotation_deg),
                rotate(p(1.0, 0.0), rotation_deg),
            );
            let v = seg(
                rotate(p(0.0, -1.0), rotation_deg),
                rotate(p(0.0, 1.0), rotation_deg),
            );
            line_inside(outline, h.start, h.end)
                .into_iter()
                .chain(line_inside(outline, v.start, v.end))
                .collect()
        }
    }
}
fn fan(outline: &[LocalPoint]) -> Vec<Vec<LocalPoint>> {
    outline
        .iter()
        .copied()
        .zip(outline.iter().copied().cycle().skip(1))
        .take(outline.len())
        .map(|(a, b)| vec![p(0.0, 0.0), a, b])
        .collect()
}
fn sector_clip(polygon: &[LocalPoint], start: f64, end: f64) -> Vec<Vec<LocalPoint>> {
    let sweep = end - start;
    if sweep <= EPS {
        return Vec::new();
    }
    let chunks = (sweep / 179.0).ceil() as usize;
    (0..chunks)
        .filter_map(|index| {
            let a = polar(start + sweep * index as f64 / chunks as f64);
            let b = polar(start + sweep * (index + 1) as f64 / chunks as f64);
            let first = clip_half(polygon, |q| cross_math(a, q));
            let second = clip_half(&first, |q| cross_math(q, b));
            (second.len() >= 3).then_some(second)
        })
        .collect()
}
fn clip_half<F: Fn(LocalPoint) -> f64>(polygon: &[LocalPoint], distance: F) -> Vec<LocalPoint> {
    let Some(mut before) = polygon.last().copied() else {
        return Vec::new();
    };
    let mut before_distance = distance(before);
    let mut output = Vec::new();
    for current in polygon.iter().copied() {
        let current_distance = distance(current);
        let inside_before = before_distance >= -EPS;
        let inside_current = current_distance >= -EPS;
        if inside_before != inside_current {
            let ratio = before_distance / (before_distance - current_distance);
            output.push(p(
                before.x + (current.x - before.x) * ratio,
                before.y + (current.y - before.y) * ratio,
            ));
        }
        if inside_current {
            output.push(current);
        }
        before = current;
        before_distance = current_distance;
    }
    output
}
fn area(points: &[LocalPoint]) -> f64 {
    points
        .iter()
        .copied()
        .zip(points.iter().copied().cycle().skip(1))
        .take(points.len())
        .map(|(a, b)| a.x * b.y - a.y * b.x)
        .sum::<f64>()
        / 2.0
}
fn cross_math(a: LocalPoint, b: LocalPoint) -> f64 {
    a.x * -b.y - -a.y * b.x
}
fn cross(a: LocalPoint, b: LocalPoint) -> f64 {
    a.x * b.y - a.y * b.x
}
fn ray(outline: &[LocalPoint], angle: f64) -> Option<LocalPoint> {
    let d = polar(angle);
    outline
        .iter()
        .copied()
        .zip(outline.iter().copied().cycle().skip(1))
        .take(outline.len())
        .filter_map(|(a, b)| {
            let edge = p(b.x - a.x, b.y - a.y);
            let denominator = cross(d, edge);
            if denominator.abs() <= EPS {
                return None;
            }
            let t = cross(a, edge) / denominator;
            let u = cross(a, d) / denominator;
            (t >= -EPS && (-EPS..=1.0 + EPS).contains(&u)).then_some(p(d.x * t, d.y * t))
        })
        .max_by(|a, b| {
            (a.x * a.x + a.y * a.y)
                .partial_cmp(&(b.x * b.x + b.y * b.y))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}
fn line_inside(outline: &[LocalPoint], start: LocalPoint, end: LocalPoint) -> Vec<LocalSegment> {
    let d = p(end.x - start.x, end.y - start.y);
    let mut values = vec![-2.0, 2.0];
    for (a, b) in outline
        .iter()
        .copied()
        .zip(outline.iter().copied().cycle().skip(1))
        .take(outline.len())
    {
        let edge = p(b.x - a.x, b.y - a.y);
        let denominator = cross(d, edge);
        if denominator.abs() <= EPS {
            continue;
        }
        let r = p(a.x - start.x, a.y - start.y);
        let t = cross(r, edge) / denominator;
        let u = cross(r, d) / denominator;
        if (-EPS..=1.0 + EPS).contains(&u) {
            values.push(t);
        }
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values.dedup_by(|a, b| (*a - *b).abs() <= EPS);
    values
        .windows(2)
        .filter_map(|pair| {
            let m = (pair[0] + pair[1]) / 2.0;
            let midpoint = p(start.x + m * d.x, start.y + m * d.y);
            inside(midpoint, outline).then_some(seg(
                p(start.x + pair[0] * d.x, start.y + pair[0] * d.y),
                p(start.x + pair[1] * d.x, start.y + pair[1] * d.y),
            ))
        })
        .collect()
}
fn inside(point: LocalPoint, polygon: &[LocalPoint]) -> bool {
    polygon
        .iter()
        .copied()
        .zip(polygon.iter().copied().cycle().skip(1))
        .take(polygon.len())
        .fold(false, |state, (a, b)| {
            let crosses = (a.y > point.y) != (b.y > point.y)
                && point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x;
            if crosses { !state } else { state }
        })
}

#[cfg(test)]
mod tests {
    use plotters::prelude::*;

    use super::*;

    const EDGE: ShapeStyle = ShapeStyle {
        color: RGBAColor(20, 30, 40, 1.0),
        filled: false,
        stroke_width: 2,
    };

    fn polygon_area(polygons: &[MarkerFillPolygon]) -> f64 {
        polygons
            .iter()
            .map(|polygon| area(&polygon.points).abs())
            .sum()
    }

    fn assert_centred(points: &[LocalPoint]) {
        let min_x = points
            .iter()
            .map(|point| point.x)
            .fold(f64::INFINITY, f64::min);
        let max_x = points
            .iter()
            .map(|point| point.x)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_y = points
            .iter()
            .map(|point| point.y)
            .fold(f64::INFINITY, f64::min);
        let max_y = points
            .iter()
            .map(|point| point.y)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((min_x + max_x).abs() < 1.0e-9);
        assert!((min_y + max_y).abs() < 1.0e-9);
        assert!(max_x - min_x <= 2.0 + 1.0e-9);
        assert!(max_y - min_y <= 2.0 + 1.0e-9);
    }

    #[test]
    fn fillable_outlines_are_centred_and_rotation_preserves_their_centre() {
        let shapes = [
            MarkerShape::Circle,
            MarkerShape::Ellipse { aspect_ratio: 1.8 },
            MarkerShape::Square,
            MarkerShape::Rectangle { aspect_ratio: 0.6 },
            MarkerShape::RoundedSquare { corner_ratio: 0.25 },
            MarkerShape::Diamond,
            MarkerShape::TriangleUp,
            MarkerShape::TriangleDown,
            MarkerShape::TriangleLeft,
            MarkerShape::TriangleRight,
            MarkerShape::hexagon(),
            MarkerShape::star5(),
        ];
        for shape in shapes {
            let outline = MarkerGeometry::new(shape).unwrap().outline().unwrap();
            assert_centred(&outline);
            let rotated = MarkerGeometry::new(shape)
                .unwrap()
                .rotated(37.0)
                .unwrap()
                .outline()
                .unwrap();
            assert_centred(&rotated);
        }
    }

    #[test]
    fn triangles_have_distinct_tip_directions() {
        let outline = |shape| MarkerGeometry::new(shape).unwrap().outline().unwrap();
        let up = outline(MarkerShape::TriangleUp);
        let down = outline(MarkerShape::TriangleDown);
        let left = outline(MarkerShape::TriangleLeft);
        let right = outline(MarkerShape::TriangleRight);
        assert_ne!(up, down);
        assert_ne!(left, right);
        assert_eq!(up.iter().filter(|point| point.y < -0.99).count(), 1);
        assert_eq!(down.iter().filter(|point| point.y > 0.99).count(), 1);
        assert_eq!(left.iter().filter(|point| point.x < -0.99).count(), 1);
        assert_eq!(right.iter().filter(|point| point.x > 0.99).count(), 1);
        assert_eq!(
            MarkerGeometry::new(MarkerShape::Triangle).unwrap().shape(),
            MarkerShape::TriangleUp
        );
    }

    #[test]
    fn regular_and_star_vertices_follow_the_requested_pattern() {
        assert_eq!(
            MarkerGeometry::new(MarkerShape::pentagon())
                .unwrap()
                .outline()
                .unwrap()
                .len(),
            5
        );
        let star = MarkerGeometry::new(MarkerShape::Star {
            points: 6,
            inner_ratio: 0.4,
            rotation_deg: 0.0,
        })
        .unwrap()
        .outline()
        .unwrap();
        assert_eq!(star.len(), 12);
        let radii: Vec<_> = star
            .iter()
            .map(|point| (point.x * point.x + point.y * point.y).sqrt())
            .collect();
        for pair in radii.windows(2) {
            assert!((pair[0] - pair[1]).abs() > 0.15);
        }
    }

    #[test]
    fn empty_solid_and_stroke_only_models_have_unambiguous_draw_plans() {
        let empty = MarkerStyle::empty(MarkerShape::Circle, EDGE)
            .unwrap()
            .drawing()
            .unwrap();
        assert!(empty.fills.is_empty());
        assert!(empty.edge.is_some());
        let solid = MarkerStyle::solid(MarkerShape::Square, RED, EDGE)
            .unwrap()
            .drawing()
            .unwrap();
        assert_eq!(solid.fills.len(), 1);
        assert_eq!(solid.fills[0].points.len(), 4);
        let stroke = MarkerStyle::empty(MarkerShape::Plus, EDGE)
            .unwrap()
            .drawing()
            .unwrap();
        assert!(stroke.outline.is_empty());
        assert_eq!(stroke.strokes.len(), 2);
        assert!(matches!(
            MarkerStyle::solid(MarkerShape::Cross, RED, EDGE),
            Err(MarkerError::FillUnsupportedForStrokeOnly { .. })
        ));
    }

    #[test]
    fn radial_weights_are_normalized_and_dividers_follow_their_angles() {
        let style = MarkerStyle::weighted_radial(
            MarkerShape::Circle,
            0.0,
            SweepDirection::CounterClockwise,
            vec![MarkerSlice::new(1.0, RED), MarkerSlice::new(3.0, BLUE)],
            Some(WHITE.stroke_width(1)),
            Some(EDGE),
        )
        .unwrap();
        let drawing = style.drawing().unwrap();
        assert_eq!(drawing.dividers.len(), 1);
        let end = drawing.dividers[0].end;
        let angle = (-end.y).atan2(end.x).to_degrees().rem_euclid(360.0);
        assert!((angle - 90.0).abs() < 1.0e-6, "{angle}");
        let outer = area(&drawing.outline).abs();
        assert!(
            (polygon_area(&drawing.fills) - outer).abs() < 1.0e-6,
            "{} {outer}",
            polygon_area(&drawing.fills)
        );
    }

    #[test]
    fn every_partition_mode_covers_its_outer_shape_without_outer_edge_duplication() {
        let modes = [
            MarkerStyle::horizontal(MarkerShape::Circle, RED, BLUE, Some(EDGE)).unwrap(),
            MarkerStyle::vertical(MarkerShape::Circle, RED, BLUE, Some(EDGE)).unwrap(),
            MarkerStyle::diagonal_forward(MarkerShape::Circle, RED, BLUE, Some(EDGE)).unwrap(),
            MarkerStyle::diagonal_backward(MarkerShape::Circle, RED, BLUE, Some(EDGE)).unwrap(),
            MarkerStyle::quadrants(
                MarkerShape::Circle,
                25.0,
                [RED, BLUE, GREEN, YELLOW],
                Some(EDGE),
            )
            .unwrap(),
        ];
        for style in modes {
            let drawing = style.drawing().unwrap();
            let outer = area(&drawing.outline).abs();
            assert!((polygon_area(&drawing.fills) - outer).abs() < 1.0e-6);
            assert_eq!(drawing.edge, Some(EDGE));
            for polygon in drawing.fills {
                for point in polygon.points {
                    assert!(point.x.abs() <= 1.0 + 1.0e-8 && point.y.abs() <= 1.0 + 1.0e-8);
                }
            }
        }
    }

    #[test]
    fn divider_is_optional_and_edge_and_fill_are_independent() {
        let no_divider = MarkerStyle::horizontal(MarkerShape::Diamond, RED, BLUE, Some(EDGE))
            .unwrap()
            .drawing()
            .unwrap();
        assert!(no_divider.dividers.is_empty());
        let with_divider = MarkerStyle::partitioned(
            MarkerShape::Diamond,
            MarkerPartition::Horizontal,
            vec![MarkerSlice::new(1.0, RED), MarkerSlice::new(1.0, BLUE)],
            Some(WHITE.stroke_width(2)),
            Some(EDGE),
        )
        .unwrap()
        .drawing()
        .unwrap();
        assert_eq!(with_divider.dividers.len(), 1);
        assert_eq!(with_divider.edge.unwrap().color, EDGE.color);
        assert_eq!(with_divider.fills[0].color, RED.to_rgba());
    }

    #[test]
    fn svg_draw_order_emits_one_outer_edge_after_partition_fills() {
        let edge = ShapeStyle {
            color: RGBAColor(1, 2, 3, 1.0),
            filled: false,
            stroke_width: 2,
        };
        let style = MarkerStyle::partitioned(
            MarkerShape::Circle,
            MarkerPartition::Vertical,
            vec![MarkerSlice::new(1.0, RED), MarkerSlice::new(1.0, BLUE)],
            Some(WHITE.stroke_width(1)),
            Some(edge),
        )
        .unwrap();
        let marker = MarkerElement::new((40, 40), 16, style).unwrap();
        let mut svg = String::new();
        {
            let root = SVGBackend::with_string(&mut svg, (80, 80)).into_drawing_area();
            root.fill(&WHITE).unwrap();
            root.draw(&marker).unwrap();
            root.present().unwrap();
        }
        assert!(!svg.contains("<image"));
        assert_eq!(svg.matches("stroke=\"#010203\"").count(), 1);
        let last_fill = svg.rfind("fill=\"#0000FF\"").unwrap();
        let edge_position = svg.rfind("stroke=\"#010203\"").unwrap();
        assert!(edge_position > last_fill);
    }

    #[test]
    fn invalid_marker_models_are_rejected() {
        assert!(matches!(
            MarkerGeometry::new(MarkerShape::RegularPolygon {
                sides: 2,
                rotation_deg: 0.0
            }),
            Err(MarkerError::InvalidPolygonSides { .. })
        ));
        assert!(matches!(
            MarkerGeometry::new(MarkerShape::Star {
                points: 17,
                inner_ratio: 0.5,
                rotation_deg: 0.0
            }),
            Err(MarkerError::InvalidStarPoints { .. })
        ));
        assert!(matches!(
            MarkerGeometry::new(MarkerShape::Star {
                points: 5,
                inner_ratio: 1.0,
                rotation_deg: 0.0
            }),
            Err(MarkerError::InvalidStarInnerRatio { .. })
        ));
        assert!(matches!(
            MarkerStyle::equal_radial(MarkerShape::Circle, Vec::<RGBColor>::new(), Some(EDGE)),
            Err(MarkerError::EmptyPartition)
        ));
        assert!(matches!(
            MarkerStyle::equal_radial(
                MarkerShape::Circle,
                [RED, BLUE, GREEN, YELLOW, BLACK],
                Some(EDGE)
            ),
            Err(MarkerError::TooManySlices { .. })
        ));
        assert!(matches!(
            MarkerStyle::weighted_radial(
                MarkerShape::Circle,
                0.0,
                SweepDirection::Clockwise,
                vec![MarkerSlice::new(0.0, RED)],
                None,
                Some(EDGE)
            ),
            Err(MarkerError::InvalidSliceWeight { .. })
        ));
        assert!(matches!(
            MarkerStyle::empty(MarkerShape::Circle, EDGE)
                .unwrap()
                .scaled(0)
                .validate(),
            Ok(())
        ));
    }
}
