mod builder;
mod context;
mod contour_display;
mod error;
mod mesh;
mod rotated_text;

pub use builder::TernaryChartBuilder;
pub use context::{CartesianChartContext, CartesianPlottingArea, TernaryChart};
pub use contour_display::{
    ContourColorBar, ContourColorBarOrientation, ContourColorBarPosition, ContourDisplayError,
    ContourLabelAnchor, ContourLabelConfig, ContourLabelMode, ContourLabelPlacement,
    ContourLabelStyle,
};
pub use error::TernaryChartError;
pub use mesh::{
    AxisLabelFormat, AxisNamePosition, AxisTextStyle, CornerLabelVisibility, CroppedAxisPolicy,
    EndpointLabelPolicy, TernaryAxis, TernaryAxisConfig, TernaryMeshConfig, TickDirection,
    TickRangeMode, TickSpec, TickStyle,
};

pub(crate) use rotated_text::RotatedText;
pub use rotated_text::{
    PreparedRotatedText, capture_rotated_text, capture_svg_rotated_text,
    draw_prepared_rotated_text, svg_rotated_text_elements,
};
