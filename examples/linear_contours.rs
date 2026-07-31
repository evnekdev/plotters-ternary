mod contour_common;
mod legend_support;
mod output_support;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    contour_common::write_outputs(contour_common::ContourExample::Linear)
}
