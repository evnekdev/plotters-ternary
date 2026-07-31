#[path = "advanced_contour_common/mod.rs"]
mod advanced_contour_common;
#[path = "output_support/mod.rs"]
mod output_support;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    advanced_contour_common::run(advanced_contour_common::AdvancedContourExample::RepeatedLabels)
}
