mod common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    common::write_outputs(common::ExampleView::CroppedRight)
}
