/// Built-in Plotters marker shapes available to [`crate::TernaryPointSeries`].
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum MarkerShape {
    /// A circular marker; fill and stroke follow its `ShapeStyle`.
    #[default]
    Circle,
    /// Plotters' diagonal cross marker.
    Cross,
    /// Plotters' filled triangular marker.
    Triangle,
}

/// Visibility policy for a marker centre relative to the logical viewport.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum MarkerClipMode {
    /// Draw only when the projected centre is inside or on the viewport.
    #[default]
    Centre,
    /// Submit every valid projected centre to Plotters.
    ///
    /// This is not bounds clipping. An outside coordinate may be affected by
    /// Plotters' plotting-area coordinate truncation.
    None,
}
