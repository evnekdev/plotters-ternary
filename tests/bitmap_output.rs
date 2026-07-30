#![allow(dead_code)]

#[path = "../examples/output_support/mod.rs"]
mod output_support;

use image::GenericImageView;
use output_support::{BitmapQuality, BitmapRenderError, BitmapRenderOptions, render_png, scaled};
use plotters::prelude::*;

#[test]
fn supersampling_dimensions_and_style_scale_are_explicit() {
    let options = BitmapRenderOptions::new((1_000, 800), BitmapQuality::Supersampled { factor: 3 });
    assert_eq!(options.scale().unwrap(), 3);
    assert_eq!(options.render_size().unwrap(), (3_000, 2_400));
    assert_eq!(options.buffer_len().unwrap(), 3_000 * 2_400 * 3);
    assert_eq!(scaled(7, options.scale().unwrap()), 21);
    assert_eq!(
        BitmapRenderOptions::new((1_000, 800), BitmapQuality::Native)
            .render_size()
            .unwrap(),
        (1_000, 800)
    );
}

#[test]
fn invalid_and_overflowing_supersampling_is_rejected() {
    assert_eq!(
        BitmapRenderOptions::new((100, 100), BitmapQuality::Supersampled { factor: 0 },)
            .render_size(),
        Err(BitmapRenderError::InvalidSupersampling { factor: 0 })
    );
    assert_eq!(
        BitmapRenderOptions::new((100, 100), BitmapQuality::Supersampled { factor: 5 },)
            .render_size(),
        Err(BitmapRenderError::InvalidSupersampling { factor: 5 })
    );
    assert_eq!(
        BitmapRenderOptions::new((u32::MAX, 1), BitmapQuality::Supersampled { factor: 4 },)
            .render_size(),
        Err(BitmapRenderError::DimensionOverflow)
    );
}

#[test]
fn supersampled_output_is_generated_at_the_requested_final_dimensions() {
    let path = std::env::temp_dir().join(format!(
        "plotters-ternary-supersampling-{}.png",
        std::process::id()
    ));
    let options = BitmapRenderOptions::new((64, 48), BitmapQuality::Supersampled { factor: 3 });
    render_png(
        &path,
        options,
        |root, scale| {
            assert_eq!(root.dim_in_pixel(), (192, 144));
            assert_eq!(scale, 3);
            root.fill(&WHITE)?;
            root.draw(&PathElement::new(
                [(0, 0), (191, 143)],
                BLACK.stroke_width(scale),
            ))?;
            root.present()?;
            Ok(())
        },
        |root| {
            assert_eq!(root.dim_in_pixel(), (64, 48));
            root.draw(&Text::new(
                "final-size text",
                (32, 24),
                ("sans-serif", 12).into_font(),
            ))?;
            root.present()?;
            Ok(())
        },
    )
    .unwrap();

    let image = image::ImageReader::open(&path).unwrap().decode().unwrap();
    assert_eq!(image.dimensions(), (64, 48));
    assert!(std::fs::metadata(&path).unwrap().len() > 0);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn native_mode_runs_both_layers_at_final_resolution_without_rescaling_text() {
    let path = std::env::temp_dir().join(format!(
        "plotters-ternary-native-layers-{}.png",
        std::process::id()
    ));
    let final_font_size = 12_u32;
    render_png(
        &path,
        BitmapRenderOptions::new((40, 30), BitmapQuality::Native),
        |root, scale| {
            assert_eq!(root.dim_in_pixel(), (40, 30));
            assert_eq!(scale, 1);
            root.fill(&WHITE)?;
            root.present()?;
            Ok(())
        },
        |root| {
            assert_eq!(root.dim_in_pixel(), (40, 30));
            root.draw(&Text::new(
                "caption and labels",
                (20, 15),
                ("sans-serif", final_font_size).into_font(),
            ))?;
            root.present()?;
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(final_font_size, 12);
    assert_eq!(
        image::ImageReader::open(&path)
            .unwrap()
            .decode()
            .unwrap()
            .dimensions(),
        (40, 30)
    );
    std::fs::remove_file(path).unwrap();
}

#[test]
fn text_layer_metrics_and_anchor_are_independent_of_supersampling_factor() {
    const OUTPUT_SIZE: (u32, u32) = (120, 90);
    const FONT_SIZE: u32 = 18;
    const TEXT_ANCHOR: (i32, i32) = (60, 24);

    for factor in [2, 3, 4] {
        let path = std::env::temp_dir().join(format!(
            "plotters-ternary-text-layer-{}-{factor}.png",
            std::process::id()
        ));
        let mut geometry_dimensions = None;
        let mut text_dimensions = None;
        let mut observed_font_size = None;
        let mut observed_anchor = None;
        render_png(
            &path,
            BitmapRenderOptions::new(OUTPUT_SIZE, BitmapQuality::Supersampled { factor }),
            |root, scale| {
                geometry_dimensions = Some(root.dim_in_pixel());
                assert_eq!(scale, factor);
                root.fill(&WHITE)?;
                root.present()?;
                Ok(())
            },
            |root| {
                text_dimensions = Some(root.dim_in_pixel());
                observed_font_size = Some(FONT_SIZE);
                observed_anchor = Some(TEXT_ANCHOR);
                root.draw(&Text::new(
                    "T",
                    TEXT_ANCHOR,
                    ("sans-serif", FONT_SIZE).into_font().color(&RED),
                ))?;
                root.present()?;
                Ok(())
            },
        )
        .unwrap();

        assert_eq!(
            geometry_dimensions,
            Some((OUTPUT_SIZE.0 * factor, OUTPUT_SIZE.1 * factor))
        );
        assert_eq!(text_dimensions, Some(OUTPUT_SIZE));
        assert_eq!(observed_font_size, Some(FONT_SIZE));
        assert_eq!(observed_anchor, Some(TEXT_ANCHOR));
        let image = image::ImageReader::open(&path)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgb8();
        assert!(
            image
                .pixels()
                .any(|pixel| u16::from(pixel[0]) > u16::from(pixel[1]) + 20)
        );
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn lanczos_supersampling_adds_fractional_edge_coverage_to_diagonal_geometry() {
    fn render_diagonal(path: &std::path::Path, quality: BitmapQuality) {
        render_png(
            path,
            BitmapRenderOptions::new((80, 60), quality),
            |root, scale| {
                root.fill(&WHITE)?;
                let dimensions = root.dim_in_pixel();
                root.draw(&PathElement::new(
                    [(3, dimensions.1 as i32 - 4), (dimensions.0 as i32 - 4, 3)],
                    BLACK.stroke_width(scale),
                ))?;
                root.present()?;
                Ok(())
            },
            |root| {
                root.present()?;
                Ok(())
            },
        )
        .unwrap();
    }

    fn fractional_pixels(path: &std::path::Path) -> usize {
        image::ImageReader::open(path)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgb8()
            .pixels()
            .filter(|pixel| pixel[0] > 0 && pixel[0] < 255)
            .count()
    }

    let temporary = std::env::temp_dir();
    let native = temporary.join(format!(
        "plotters-ternary-native-edge-{}.png",
        std::process::id()
    ));
    let supersampled = temporary.join(format!(
        "plotters-ternary-supersampled-edge-{}.png",
        std::process::id()
    ));
    render_diagonal(&native, BitmapQuality::Native);
    render_diagonal(&supersampled, BitmapQuality::Supersampled { factor: 3 });
    assert!(fractional_pixels(&supersampled) > fractional_pixels(&native));
    std::fs::remove_file(native).unwrap();
    std::fs::remove_file(supersampled).unwrap();
}
