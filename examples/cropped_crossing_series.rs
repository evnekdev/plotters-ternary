mod output_support;
mod series_common;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    series_common::write_outputs(series_common::SeriesExample::CroppedCrossing)
}
