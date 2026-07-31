use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::Path;

use image::{RgbImage, imageops::FilterType};
use plotters::backend::DrawingBackend;
use plotters::coord::Shift;
use plotters::drawing::{DrawingArea, DrawingAreaErrorKind, IntoDrawingArea};
use plotters::prelude::{BitMapBackend, FontStyle, IntoTextStyle, SVGBackend};
use plotters_ternary::chart::{
    capture_rotated_text, capture_svg_rotated_text, draw_prepared_rotated_text,
    svg_rotated_text_elements,
};

pub const DEFAULT_BITMAP_SUPERSAMPLING: u32 = 3;
pub const MAX_BITMAP_SUPERSAMPLING: u32 = 4;
/// Development guardrail for the in-memory high-resolution RGB buffer.
pub const MAX_BITMAP_BUFFER_BYTES: usize = 512 * 1024 * 1024;

/// Reserve the same caption strip as the final-resolution text pass.
///
/// Plotters' font measurement is backend-specific and is not always perfectly
/// proportional when the bitmap geometry pass is supersampled. Measure the
/// caption with its final font size, then scale the resulting strip in pixels.
/// The geometry pass can omit caption glyphs while retaining the exact final
/// layout used by the text pass.
pub fn reserve_final_caption_space<DB>(
    root: &DrawingArea<DB, Shift>,
    caption: &str,
    final_font_size: u32,
    scale: u32,
) -> Result<DrawingArea<DB, Shift>, DrawingAreaErrorKind<DB::ErrorType>>
where
    DB: DrawingBackend,
{
    let style = ("sans-serif", final_font_size, FontStyle::Bold).into_text_style(root);
    let (_, text_height) = root.estimate_text_size(caption, &style)?;
    let vertical_padding = (text_height / 2).min(5);
    let strip_height = text_height.saturating_add(vertical_padding.saturating_mul(2));
    Ok(root.margin(strip_height.saturating_mul(scale), 0, 0, 0))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(dead_code)]
pub enum BitmapQuality {
    Native,
    Supersampled { factor: u32 },
}

impl Default for BitmapQuality {
    fn default() -> Self {
        Self::Supersampled {
            factor: DEFAULT_BITMAP_SUPERSAMPLING,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BitmapRenderOptions {
    pub output_size: (u32, u32),
    pub quality: BitmapQuality,
}

impl BitmapRenderOptions {
    pub const fn new(output_size: (u32, u32), quality: BitmapQuality) -> Self {
        Self {
            output_size,
            quality,
        }
    }

    pub fn scale(self) -> Result<u32, BitmapRenderError> {
        match self.quality {
            BitmapQuality::Native => Ok(1),
            BitmapQuality::Supersampled { factor: 0 } => {
                Err(BitmapRenderError::InvalidSupersampling { factor: 0 })
            }
            BitmapQuality::Supersampled { factor } if factor > MAX_BITMAP_SUPERSAMPLING => {
                Err(BitmapRenderError::InvalidSupersampling { factor })
            }
            BitmapQuality::Supersampled { factor } => Ok(factor),
        }
    }

    pub fn render_size(self) -> Result<(u32, u32), BitmapRenderError> {
        let scale = self.scale()?;
        let width = self
            .output_size
            .0
            .checked_mul(scale)
            .ok_or(BitmapRenderError::DimensionOverflow)?;
        let height = self
            .output_size
            .1
            .checked_mul(scale)
            .ok_or(BitmapRenderError::DimensionOverflow)?;
        if width == 0 || height == 0 {
            return Err(BitmapRenderError::ZeroOutputDimension);
        }
        Ok((width, height))
    }

    pub fn buffer_len(self) -> Result<usize, BitmapRenderError> {
        let (width, height) = self.render_size()?;
        let pixels = u64::from(width)
            .checked_mul(u64::from(height))
            .and_then(|value| value.checked_mul(3))
            .ok_or(BitmapRenderError::BufferSizeOverflow)?;
        let bytes = usize::try_from(pixels).map_err(|_| BitmapRenderError::BufferSizeOverflow)?;
        if bytes > MAX_BITMAP_BUFFER_BYTES {
            return Err(BitmapRenderError::BufferTooLarge {
                bytes,
                maximum: MAX_BITMAP_BUFFER_BYTES,
            });
        }
        Ok(bytes)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BitmapRenderError {
    InvalidSupersampling { factor: u32 },
    ZeroOutputDimension,
    DimensionOverflow,
    BufferSizeOverflow,
    BufferTooLarge { bytes: usize, maximum: usize },
    InvalidRgbBuffer,
}

impl Display for BitmapRenderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSupersampling { factor } => write!(
                formatter,
                "bitmap supersampling factor must be in 1..={MAX_BITMAP_SUPERSAMPLING}; received {factor}"
            ),
            Self::ZeroOutputDimension => {
                write!(formatter, "bitmap output dimensions must be nonzero")
            }
            Self::DimensionOverflow => write!(formatter, "scaled bitmap dimensions overflow u32"),
            Self::BufferSizeOverflow => write!(formatter, "scaled RGB buffer size overflows usize"),
            Self::BufferTooLarge { bytes, maximum } => write!(
                formatter,
                "scaled RGB buffer requires {bytes} bytes; configured limit is {maximum}"
            ),
            Self::InvalidRgbBuffer => {
                write!(formatter, "RGB buffer length does not match dimensions")
            }
        }
    }
}

impl Error for BitmapRenderError {}

pub fn scaled(value: u32, scale: u32) -> u32 {
    value.saturating_mul(scale)
}

pub fn render_svg<Geometry, Text>(
    path: impl AsRef<Path>,
    output_size: (u32, u32),
    render_geometry: Geometry,
    render_text: Text,
) -> Result<(), Box<dyn Error>>
where
    Geometry:
        for<'buffer> FnOnce(DrawingArea<SVGBackend<'buffer>, Shift>) -> Result<(), Box<dyn Error>>,
    Text:
        for<'buffer> FnOnce(DrawingArea<SVGBackend<'buffer>, Shift>) -> Result<(), Box<dyn Error>>,
{
    std::fs::write(
        path,
        render_svg_string(output_size, render_geometry, render_text)?,
    )?;
    Ok(())
}

pub fn render_svg_string<Geometry, Text>(
    output_size: (u32, u32),
    render_geometry: Geometry,
    render_text: Text,
) -> Result<String, Box<dyn Error>>
where
    Geometry:
        for<'buffer> FnOnce(DrawingArea<SVGBackend<'buffer>, Shift>) -> Result<(), Box<dyn Error>>,
    Text:
        for<'buffer> FnOnce(DrawingArea<SVGBackend<'buffer>, Shift>) -> Result<(), Box<dyn Error>>,
{
    let mut geometry_document = String::new();
    {
        let root = SVGBackend::with_string(&mut geometry_document, output_size).into_drawing_area();
        render_geometry(root)?;
    }
    let mut text_document = String::new();
    let (text_result, rotated_text) = capture_svg_rotated_text(|| {
        let root = SVGBackend::with_string(&mut text_document, output_size).into_drawing_area();
        render_text(root)
    });
    text_result?;

    let (header, geometry) = svg_parts(&geometry_document)?;
    let (_, text) = svg_parts(&text_document)?;
    let rotated_text = svg_rotated_text_elements(&rotated_text);
    Ok(format!(
        "{header}\n<g id=\"ternary-geometry\" shape-rendering=\"geometricPrecision\" stroke-linecap=\"round\" stroke-linejoin=\"round\">\n{geometry}</g>\n<g id=\"ternary-text\">\n{text}{rotated_text}</g>\n</svg>\n"
    ))
}

fn svg_parts(document: &str) -> Result<(&str, &str), Box<dyn Error>> {
    let open_end = document.find('>').ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "SVG root start is missing")
    })? + 1;
    let close_start = document.rfind("</svg>").ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "SVG root end is missing")
    })?;
    if close_start < open_end {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "SVG root bounds are reversed",
        )
        .into());
    }
    Ok((&document[..open_end], &document[open_end..close_start]))
}
pub fn render_png<Geometry, Text>(
    path: impl AsRef<Path>,
    options: BitmapRenderOptions,
    render_geometry: Geometry,
    render_text: Text,
) -> Result<(), Box<dyn Error>>
where
    Geometry: for<'buffer> FnOnce(
        DrawingArea<BitMapBackend<'buffer>, Shift>,
        u32,
    ) -> Result<(), Box<dyn Error>>,
    Text: for<'buffer> FnOnce(
        DrawingArea<BitMapBackend<'buffer>, Shift>,
    ) -> Result<(), Box<dyn Error>>,
{
    let scale = options.scale()?;
    let render_size = options.render_size()?;
    let mut geometry_buffer = vec![255; options.buffer_len()?];
    {
        let root =
            BitMapBackend::with_buffer(&mut geometry_buffer, render_size).into_drawing_area();
        render_geometry(root, scale)?;
    }

    let geometry = RgbImage::from_raw(render_size.0, render_size.1, geometry_buffer)
        .ok_or(BitmapRenderError::InvalidRgbBuffer)?;
    let mut final_image = match options.quality {
        BitmapQuality::Native => geometry,
        BitmapQuality::Supersampled { .. } => image::imageops::resize(
            &geometry,
            options.output_size.0,
            options.output_size.1,
            FilterType::Lanczos3,
        ),
    };

    let (text_result, rotated_text) = capture_rotated_text(|| {
        let root = BitMapBackend::with_buffer(final_image.as_mut(), options.output_size)
            .into_drawing_area();
        render_text(root)
    });
    text_result?;
    {
        let root = BitMapBackend::with_buffer(final_image.as_mut(), options.output_size)
            .into_drawing_area();
        draw_prepared_rotated_text(&root, &rotated_text)?;
        root.present()?;
    }

    final_image.save(path)?;
    Ok(())
}
