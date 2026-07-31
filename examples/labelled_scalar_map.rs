mod bands_common;
mod output_support;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    bands_common::write_outputs(bands_common::Example::LabelledMap)
}
