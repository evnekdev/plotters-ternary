use std::fmt;

use plotters::style::{Color, FontTransform};
use plotters_backend::text_anchor::{HPos, Pos, VPos};

use crate::chart::AxisTextStyle;
use crate::coord::{Normalization, TernaryPoint, Tolerance};

/// Owned text attributes for [`TernaryText`].
///
/// This is an alias of [`AxisTextStyle`] so annotation text uses the same
/// final-output-pixel font, weight, and RGBA colour model as mesh labels.
pub type AnnotationTextStyle = AxisTextStyle;

/// Horizontal placement of an annotation relative to its ternary anchor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HorizontalAnchor {
    Left,
    #[default]
    Center,
    Right,
}

/// Vertical placement of an annotation relative to its ternary anchor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VerticalAnchor {
    Top,
    #[default]
    Center,
    Bottom,
    /// Baseline is currently mapped to Plotters' closest portable `Bottom`
    /// anchor. Exact baseline metrics are backend-specific.
    Baseline,
}

/// A portable two-dimensional text anchor.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TextAnchor {
    horizontal: HorizontalAnchor,
    vertical: VerticalAnchor,
}

impl TextAnchor {
    pub const fn new(horizontal: HorizontalAnchor, vertical: VerticalAnchor) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }
    pub const fn horizontal(self) -> HorizontalAnchor {
        self.horizontal
    }
    pub const fn vertical(self) -> VerticalAnchor {
        self.vertical
    }
    pub const fn center() -> Self {
        Self::new(HorizontalAnchor::Center, VerticalAnchor::Center)
    }
    pub(crate) fn plotters_pos(self) -> Pos {
        let horizontal = match self.horizontal {
            HorizontalAnchor::Left => HPos::Left,
            HorizontalAnchor::Center => HPos::Center,
            HorizontalAnchor::Right => HPos::Right,
        };
        let vertical = match self.vertical {
            VerticalAnchor::Top => VPos::Top,
            VerticalAnchor::Center => VPos::Center,
            VerticalAnchor::Bottom | VerticalAnchor::Baseline => VPos::Bottom,
        };
        Pos::new(horizontal, vertical)
    }
}

/// Explicit annotation visibility policy for the invisible logical viewport.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AnnotationClipMode {
    /// Emit text only if its logical ternary anchor is in or on the viewport.
    #[default]
    Anchor,
    /// Submit text even when the logical anchor lies outside the viewport.
    None,
}

/// Plotters-native, portable quarter-turn annotation rotation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TextRotation {
    #[default]
    None,
    Rotate90,
    Rotate180,
    Rotate270,
}

impl TextRotation {
    pub(crate) const fn plotters_transform(self) -> FontTransform {
        match self {
            Self::None => FontTransform::None,
            Self::Rotate90 => FontTransform::Rotate90,
            Self::Rotate180 => FontTransform::Rotate180,
            Self::Rotate270 => FontTransform::Rotate270,
        }
    }
}

/// A text annotation anchored in ternary composition space.
///
/// Content and style are owned. Pixel offsets are applied after projection and
/// are final-output pixels, so geometry supersampling never changes their
/// final placement.
pub struct TernaryText {
    point: TernaryPoint,
    text: String,
    style: AnnotationTextStyle,
    anchor: TextAnchor,
    offset: (i32, i32),
    clip_mode: AnnotationClipMode,
    rotation: TextRotation,
    normalization: Normalization,
    tolerance: Tolerance,
}

impl TernaryText {
    /// Create a final-resolution, centred, anchor-clipped annotation.
    pub fn new<S: Into<String>>(point: TernaryPoint, text: S) -> Self {
        Self {
            point,
            text: text.into(),
            style: AxisTextStyle::sans_serif(
                22,
                plotters::style::FontStyle::Normal,
                plotters::style::BLACK.to_rgba(),
            ),
            anchor: TextAnchor::default(),
            offset: (0, 0),
            clip_mode: AnnotationClipMode::default(),
            rotation: TextRotation::default(),
            normalization: Normalization::RequireUnitSum,
            tolerance: Tolerance::default(),
        }
    }

    /// Set the owned text attributes.
    pub fn style(mut self, style: AnnotationTextStyle) -> Self {
        self.style = style;
        self
    }
    /// Set the local horizontal/vertical anchor.
    pub const fn anchor(mut self, anchor: TextAnchor) -> Self {
        self.anchor = anchor;
        self
    }
    /// Set a final-output-pixel offset from the projected ternary anchor.
    pub const fn offset(mut self, offset: (i32, i32)) -> Self {
        self.offset = offset;
        self
    }
    /// Select anchor clipping or unrestricted submission.
    pub const fn clip_mode(mut self, clip_mode: AnnotationClipMode) -> Self {
        self.clip_mode = clip_mode;
        self
    }
    /// Use only a Plotters-native quarter-turn text transform.
    pub const fn rotation(mut self, rotation: TextRotation) -> Self {
        self.rotation = rotation;
        self
    }
    /// Select explicit validation or normalisation.
    pub const fn normalization(mut self, normalization: Normalization) -> Self {
        self.normalization = normalization;
        self
    }
    /// Select the tolerance for validation and anchor clipping.
    pub const fn tolerance(mut self, tolerance: Tolerance) -> Self {
        self.tolerance = tolerance;
        self
    }

    pub const fn point(&self) -> TernaryPoint {
        self.point
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub const fn offset_value(&self) -> (i32, i32) {
        self.offset
    }
    pub const fn text_anchor(&self) -> TextAnchor {
        self.anchor
    }
    pub const fn annotation_clip_mode(&self) -> AnnotationClipMode {
        self.clip_mode
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        TernaryPoint,
        String,
        AnnotationTextStyle,
        TextAnchor,
        (i32, i32),
        AnnotationClipMode,
        TextRotation,
        Normalization,
        Tolerance,
    ) {
        (
            self.point,
            self.text,
            self.style,
            self.anchor,
            self.offset,
            self.clip_mode,
            self.rotation,
            self.normalization,
            self.tolerance,
        )
    }
}

/// Annotation preparation errors.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum AnnotationError {
    /// The composition anchor could not be validated or projected.
    InvalidAnchor { source: crate::coord::Error },
}

impl fmt::Display for AnnotationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAnchor { source } => write!(f, "invalid ternary text anchor: {source}"),
        }
    }
}

impl std::error::Error for AnnotationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidAnchor { source } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_defaults_and_final_pixel_offsets_are_stable() {
        let annotation = TernaryText::new(TernaryPoint::new(0.4, 0.3, 0.3), "Liquid + α")
            .anchor(TextAnchor::new(
                HorizontalAnchor::Right,
                VerticalAnchor::Bottom,
            ))
            .offset((8, -10));
        assert_eq!(annotation.text(), "Liquid + α");
        assert_eq!(annotation.offset_value(), (8, -10));
        assert_eq!(
            annotation.text_anchor().horizontal(),
            HorizontalAnchor::Right
        );
        assert_eq!(annotation.text_anchor().vertical(), VerticalAnchor::Bottom);
    }
}
