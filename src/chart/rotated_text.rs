//! Prepared arbitrary-angle axis text with raster and SVG adapters.
//!
//! Plotters' generic backend trait cannot express an SVG text transform.  The
//! regular renderer therefore creates an antialiased coverage mask for raster
//! backends, while the explicit SVG capture adapter records the same prepared
//! command and emits native, searchable SVG text in the example output helper.

use std::cell::RefCell;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::iter::{Once, once};

use plotters::backend::DrawingBackend;
use plotters::coord::Shift;
use plotters::drawing::{DrawingArea, DrawingAreaErrorKind};
use plotters::element::{Drawable, PointCollection};
use plotters::style::{FontStyle, IntoFont, TextStyle};
use plotters_backend::DrawingErrorKind;

use super::AxisTextStyle;

/// The default internal glyph-mask scale for non-quarter-turn raster text.
pub(crate) const DEFAULT_ROTATED_TEXT_MASK_SCALE: u32 = 4;
pub(crate) const MAX_ROTATED_TEXT_MASK_SCALE: u32 = 4;
const MAX_ROTATED_TEXT_MASK_BYTES: usize = 64 * 1024 * 1024;

/// Bounded raster quality for arbitrary-angle axis names.
///
/// This remains an implementation detail until non-Plotters output adapters
/// become part of the public chart API. A factor of one still uses a coverage
/// mask and inverse bilinear rotation; it is not nearest-neighbour rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RotatedTextQuality {
    Supersampled { factor: u32 },
}

impl Default for RotatedTextQuality {
    fn default() -> Self {
        Self::Supersampled {
            factor: DEFAULT_ROTATED_TEXT_MASK_SCALE,
        }
    }
}

impl RotatedTextQuality {
    fn scale(self) -> Result<u32, RotatedTextError> {
        match self {
            Self::Supersampled { factor: 0 } => Err(RotatedTextError::InvalidQuality { factor: 0 }),
            Self::Supersampled { factor } if factor > MAX_ROTATED_TEXT_MASK_SCALE => {
                Err(RotatedTextError::InvalidQuality { factor })
            }
            Self::Supersampled { factor } => Ok(factor),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RotatedTextError {
    InvalidQuality { factor: u32 },
    DimensionOverflow,
    MaskTooLarge { bytes: usize },
}

impl Display for RotatedTextError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidQuality { factor } => write!(
                formatter,
                "rotated-text mask scale must be in 1..={MAX_ROTATED_TEXT_MASK_SCALE}; received {factor}"
            ),
            Self::DimensionOverflow => write!(formatter, "rotated-text mask dimensions overflow"),
            Self::MaskTooLarge { bytes } => write!(
                formatter,
                "rotated-text mask requires {bytes} bytes; maximum is {MAX_ROTATED_TEXT_MASK_BYTES}"
            ),
        }
    }
}

impl std::error::Error for RotatedTextError {}

/// One backend-neutral axis-name command after logical placement is complete.
///
/// `anchor` is the exact Plotters-mapped anchor in final output pixels;
/// `offset` is deliberately retained rather than folded into it so both raster
/// and SVG adapters consume identical layout data.
#[doc(hidden)]
#[derive(Clone)]
pub struct PreparedRotatedText {
    text: String,
    anchor: (i32, i32),
    angle_rad: f64,
    offset: (i32, i32),
    style: AxisTextStyle,
}

impl PreparedRotatedText {
    pub const fn anchor(&self) -> (i32, i32) {
        self.anchor
    }
    pub const fn offset(&self) -> (i32, i32) {
        self.offset
    }
    pub const fn angle_rad(&self) -> f64 {
        self.angle_rad
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn style(&self) -> &AxisTextStyle {
        &self.style
    }
}

impl<'element> PointCollection<'element, (i32, i32)> for &'element PreparedRotatedText {
    type Point = &'element (i32, i32);
    type IntoIter = Once<Self::Point>;

    fn point_iter(self) -> Self::IntoIter {
        once(&self.anchor)
    }
}

impl<DB: DrawingBackend> Drawable<DB> for PreparedRotatedText {
    fn draw<I: Iterator<Item = (i32, i32)>>(
        &self,
        mut positions: I,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        let Some(anchor) = positions.next() else {
            return Ok(());
        };
        let style = (
            self.style.family(),
            self.style.size(),
            self.style.font_style(),
        )
            .into_font()
            .color(&self.style.color());
        draw_coverage_rotated_text(
            backend,
            anchor,
            &self.text,
            &style,
            self.angle_rad,
            self.offset,
            RotatedTextQuality::default(),
        )
    }
}

/// Draw captured arbitrary-angle text into a final-resolution raster text pass.
#[doc(hidden)]
pub fn draw_prepared_rotated_text<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    commands: &[PreparedRotatedText],
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
    for command in commands {
        area.draw(command)?;
    }
    Ok(())
}

thread_local! {
    static SVG_CAPTURE: RefCell<Option<Vec<PreparedRotatedText>>> = const { RefCell::new(None) };
}

/// Capture prepared arbitrary-angle text submitted during one output text pass.
///
/// The closure still draws ordinary Plotters text. Only non-quarter-turn axis
/// names are collected. Raster adapters redraw the commands through the
/// coverage-mask renderer and SVG adapters insert them as native SVG `<text>`.
/// This is an explicit output-adapter boundary, not a backend type-name check
/// or downcast.
#[doc(hidden)]
pub fn capture_rotated_text<T>(render: impl FnOnce() -> T) -> (T, Vec<PreparedRotatedText>) {
    SVG_CAPTURE.with(|capture| {
        assert!(
            capture.borrow().is_none(),
            "nested SVG rotated-text captures are not supported"
        );
        *capture.borrow_mut() = Some(Vec::new());
    });
    let result = render();
    let commands = SVG_CAPTURE.with(|capture| {
        capture
            .borrow_mut()
            .take()
            .expect("SVG rotated-text capture must remain active")
    });
    (result, commands)
}

/// SVG-named compatibility adapter for existing SVG output helpers.
#[doc(hidden)]
pub fn capture_svg_rotated_text<T>(render: impl FnOnce() -> T) -> (T, Vec<PreparedRotatedText>) {
    capture_rotated_text(render)
}

fn capture_svg_command(command: PreparedRotatedText) -> bool {
    SVG_CAPTURE.with(|capture| {
        let mut capture = capture.borrow_mut();
        let Some(commands) = capture.as_mut() else {
            return false;
        };
        commands.push(command);
        true
    })
}

/// Convert prepared arbitrary-angle text commands into native SVG text nodes.
///
/// The caller owns group placement.  All string fields and text content are XML
/// escaped, while Unicode is preserved directly as UTF-8.
#[doc(hidden)]
pub fn svg_rotated_text_elements(commands: &[PreparedRotatedText]) -> String {
    let mut svg = String::new();
    for command in commands {
        let x = command.anchor.0 + command.offset.0;
        let y = command.anchor.1 + command.offset.1;
        let angle = command.angle_rad.to_degrees();
        let color = command.style.color();
        let font_attributes = match command.style.font_style() {
            FontStyle::Bold => " font-weight=\"bold\"",
            FontStyle::Italic => " font-style=\"italic\"",
            FontStyle::Oblique => " font-style=\"oblique\"",
            FontStyle::Normal => "",
        };
        svg.push_str(&format!(
            "<text x=\"{x}\" y=\"{y}\" text-anchor=\"middle\" dominant-baseline=\"middle\" transform=\"rotate({angle:.6} {x} {y})\" font-family=\"{}\" font-size=\"{}\" fill=\"#{:02X}{:02X}{:02X}\" fill-opacity=\"{:.6}\"{}>{}</text>\n",
            escape_xml(command.style.family()),
            command.style.size(),
            color.0,
            color.1,
            color.2,
            color.3,
            font_attributes,
            escape_xml(&command.text),
        ));
    }
    svg
}

fn escape_xml(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '\"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

pub(crate) struct RotatedText {
    anchor: (f64, f64),
    text: String,
    style: AxisTextStyle,
    angle: f64,
    offset: (i32, i32),
    quality: RotatedTextQuality,
}

impl RotatedText {
    pub(crate) fn new(
        anchor: (f64, f64),
        text: String,
        style: AxisTextStyle,
        angle: f64,
        offset: (i32, i32),
    ) -> Self {
        Self {
            anchor,
            text,
            style,
            angle,
            offset,
            quality: RotatedTextQuality::default(),
        }
    }
}

impl<'element> PointCollection<'element, (f64, f64)> for &'element RotatedText {
    type Point = &'element (f64, f64);
    type IntoIter = Once<Self::Point>;

    fn point_iter(self) -> Self::IntoIter {
        once(&self.anchor)
    }
}

impl<DB: DrawingBackend> Drawable<DB> for RotatedText {
    fn draw<I: Iterator<Item = (i32, i32)>>(
        &self,
        mut positions: I,
        backend: &mut DB,
        _: (u32, u32),
    ) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
        let Some(anchor) = positions.next() else {
            return Ok(());
        };
        let command = PreparedRotatedText {
            text: self.text.clone(),
            anchor,
            angle_rad: self.angle,
            offset: self.offset,
            style: self.style.clone(),
        };
        if capture_svg_command(command) {
            return Ok(());
        }
        let style = (
            self.style.family(),
            self.style.size(),
            self.style.font_style(),
        )
            .into_font()
            .color(&self.style.color());
        draw_coverage_rotated_text(
            backend,
            anchor,
            &self.text,
            &style,
            self.angle,
            self.offset,
            self.quality,
        )
    }
}

#[derive(Clone, Debug)]
struct CoverageMask {
    width: usize,
    height: usize,
    centre: (f64, f64),
    coverage: Vec<f32>,
    scale: u32,
}

impl CoverageMask {
    fn new(
        width: usize,
        height: usize,
        centre: (f64, f64),
        scale: u32,
    ) -> Result<Self, RotatedTextError> {
        let bytes = width
            .checked_mul(height)
            .and_then(|value| value.checked_mul(std::mem::size_of::<f32>()))
            .ok_or(RotatedTextError::DimensionOverflow)?;
        if bytes > MAX_ROTATED_TEXT_MASK_BYTES {
            return Err(RotatedTextError::MaskTooLarge { bytes });
        }
        Ok(Self {
            width,
            height,
            centre,
            coverage: vec![0.0; width * height],
            scale,
        })
    }

    fn set_max(&mut self, x: i32, y: i32, value: f32) {
        let (Ok(x), Ok(y)) = (usize::try_from(x), usize::try_from(y)) else {
            return;
        };
        if x < self.width && y < self.height {
            let pixel = &mut self.coverage[y * self.width + x];
            *pixel = pixel.max(value.clamp(0.0, 1.0));
        }
    }

    fn bilinear(&self, x: f64, y: f64) -> f32 {
        let x0 = x.floor() as isize;
        let y0 = y.floor() as isize;
        let tx = (x - x0 as f64) as f32;
        let ty = (y - y0 as f64) as f32;
        let sample = |x: isize, y: isize| -> f32 {
            if x < 0 || y < 0 || x >= self.width as isize || y >= self.height as isize {
                0.0
            } else {
                self.coverage[y as usize * self.width + x as usize]
            }
        };
        let top = sample(x0, y0) * (1.0 - tx) + sample(x0 + 1, y0) * tx;
        let bottom = sample(x0, y0 + 1) * (1.0 - tx) + sample(x0 + 1, y0 + 1) * tx;
        top * (1.0 - ty) + bottom * ty
    }
}

#[derive(Clone, Copy, Debug)]
struct DestinationBounds {
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

fn destination_bounds(mask: &CoverageMask, angle: f64) -> DestinationBounds {
    let scale = f64::from(mask.scale);
    let half_width = mask.width as f64 / (2.0 * scale);
    let half_height = mask.height as f64 / (2.0 * scale);
    let cosine = angle.cos().abs();
    let sine = angle.sin().abs();
    let half_x = (half_width * cosine + half_height * sine).ceil() as i32 + 1;
    let half_y = (half_width * sine + half_height * cosine).ceil() as i32 + 1;
    DestinationBounds {
        min_x: -half_x,
        max_x: half_x,
        min_y: -half_y,
        max_y: half_y,
    }
}

fn sampled_rotated_coverage(mask: &CoverageMask, angle: f64, x: i32, y: i32) -> f32 {
    let (cosine, sine) = (angle.cos(), angle.sin());
    let scale = f64::from(mask.scale);
    // Inverse destination-to-source mapping avoids nearest-neighbour forward
    // collisions and leaves out-of-mask samples transparent.
    let source_x = mask.centre.0 + scale * (f64::from(x) * cosine + f64::from(y) * sine);
    let source_y = mask.centre.1 + scale * (-f64::from(x) * sine + f64::from(y) * cosine);
    mask.bilinear(source_x, source_y)
}

fn draw_coverage_rotated_text<DB: DrawingBackend>(
    backend: &mut DB,
    anchor: (i32, i32),
    text: &str,
    style: &TextStyle<'_>,
    angle: f64,
    offset: (i32, i32),
    quality: RotatedTextQuality,
) -> Result<(), DrawingErrorKind<DB::ErrorType>> {
    let mask = coverage_mask(text, style, quality).map_err(DrawingErrorKind::FontError)?;
    let bounds = destination_bounds(&mask, angle);
    let mut color = style.color;
    for y in bounds.min_y..=bounds.max_y {
        for x in bounds.min_x..=bounds.max_x {
            let coverage = sampled_rotated_coverage(&mask, angle, x, y);
            if coverage <= 0.0 {
                continue;
            }
            color.alpha = style.color.alpha * f64::from(coverage);
            backend.draw_pixel((anchor.0 + offset.0 + x, anchor.1 + offset.1 + y), color)?;
        }
    }
    Ok(())
}

fn coverage_mask(
    text: &str,
    style: &TextStyle<'_>,
    quality: RotatedTextQuality,
) -> Result<CoverageMask, Box<dyn std::error::Error + Send + Sync>> {
    let scale = quality.scale()?;
    let font = style.font.resize(style.font.get_size() * f64::from(scale));
    let ((min_x, min_y), (max_x, max_y)) = font.layout_box(text)?;
    let padding = i32::try_from(
        scale
            .checked_mul(2)
            .ok_or(RotatedTextError::DimensionOverflow)?,
    )
    .map_err(|_| RotatedTextError::DimensionOverflow)?;
    let width = i64::from(max_x)
        .checked_sub(i64::from(min_x))
        .and_then(|value| value.checked_add(i64::from(padding) * 2 + 1))
        .ok_or(RotatedTextError::DimensionOverflow)?;
    let height = i64::from(max_y)
        .checked_sub(i64::from(min_y))
        .and_then(|value| value.checked_add(i64::from(padding) * 2 + 1))
        .ok_or(RotatedTextError::DimensionOverflow)?;
    let width = usize::try_from(width.max(1)).map_err(|_| RotatedTextError::DimensionOverflow)?;
    let height = usize::try_from(height.max(1)).map_err(|_| RotatedTextError::DimensionOverflow)?;
    let centre = (
        f64::from(padding) + f64::from(max_x - min_x) / 2.0,
        f64::from(padding) + f64::from(max_y - min_y) / 2.0,
    );
    let mut mask = CoverageMask::new(width, height, centre, scale)?;
    let base = (padding - min_x, padding - min_y);
    match font.draw(text, base, |x, y, alpha| {
        mask.set_max(x, y, alpha);
        Ok::<(), Infallible>(())
    }) {
        Ok(Ok(())) | Ok(Err(_)) => Ok(mask),
        Err(error) => Err(Box::new(error)),
    }
}

#[cfg(test)]
fn source_over(destination: [f32; 4], source: [f32; 4]) -> [f32; 4] {
    let alpha = source[3] + destination[3] * (1.0 - source[3]);
    if alpha <= f32::EPSILON {
        return [0.0; 4];
    }
    [
        (source[0] * source[3] + destination[0] * destination[3] * (1.0 - source[3])) / alpha,
        (source[1] * source[3] + destination[1] * destination[3] * (1.0 - source[3])) / alpha,
        (source[2] * source[3] + destination[2] * destination[3] * (1.0 - source[3])) / alpha,
        alpha,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use plotters::prelude::{BLACK, Color, IntoFont};

    #[test]
    fn quality_rejects_zero_and_excessive_values() {
        assert_eq!(
            RotatedTextQuality::Supersampled { factor: 0 }.scale(),
            Err(RotatedTextError::InvalidQuality { factor: 0 })
        );
        assert_eq!(
            RotatedTextQuality::Supersampled { factor: 5 }.scale(),
            Err(RotatedTextError::InvalidQuality { factor: 5 })
        );
    }

    #[test]
    fn mask_allocation_is_checked() {
        assert!(matches!(
            CoverageMask::new(usize::MAX, 2, (0.0, 0.0), 1),
            Err(RotatedTextError::DimensionOverflow)
        ));
        assert!(matches!(
            CoverageMask::new(10_000, 10_000, (0.0, 0.0), 1),
            Err(RotatedTextError::MaskTooLarge { .. })
        ));
    }

    #[test]
    fn inverse_bilinear_sampling_has_fractional_edges() {
        let mut mask = CoverageMask::new(3, 3, (1.0, 1.0), 1).unwrap();
        mask.set_max(1, 1, 1.0);
        let sample = sampled_rotated_coverage(&mask, std::f64::consts::FRAC_PI_4, 1, 0);
        assert!(sample > 0.0 && sample < 1.0);
    }

    #[test]
    fn destination_bounds_contain_rotated_complete_mask() {
        let mask = CoverageMask::new(80, 32, (40.0, 16.0), 4).unwrap();
        let angle = std::f64::consts::FRAC_PI_3;
        let bounds = destination_bounds(&mask, angle);
        assert!(bounds.min_x <= -10 && bounds.max_x >= 10);
        assert!(bounds.min_y <= -10 && bounds.max_y >= 10);
        for (x, y) in [(-10.0, -4.0), (10.0, -4.0), (10.0, 4.0), (-10.0, 4.0)] {
            let rotated_x = x * angle.cos() - y * angle.sin();
            let rotated_y = x * angle.sin() + y * angle.cos();
            assert!(rotated_x >= f64::from(bounds.min_x) - 1.0);
            assert!(rotated_x <= f64::from(bounds.max_x) + 1.0);
            assert!(rotated_y >= f64::from(bounds.min_y) - 1.0);
            assert!(rotated_y <= f64::from(bounds.max_y) + 1.0);
        }
    }

    #[test]
    fn sixty_degree_solid_stroke_stays_contiguous_without_forward_mapping_holes() {
        let mut mask = CoverageMask::new(48, 16, (24.0, 8.0), 1).unwrap();
        for y in 4..12 {
            for x in 4..44 {
                mask.set_max(x, y, 1.0);
            }
        }
        let angle = std::f64::consts::FRAC_PI_3;
        let bounds = destination_bounds(&mask, angle);
        let covered = (bounds.min_y..=bounds.max_y)
            .map(|y| {
                (bounds.min_x..=bounds.max_x)
                    .filter(|x| sampled_rotated_coverage(&mask, angle, *x, y) > 0.05)
                    .count()
            })
            .collect::<Vec<_>>();
        let non_empty = covered.iter().filter(|count| **count > 0).count();
        assert!(non_empty > 20);
        assert!(covered.iter().filter(|count| **count == 1).count() < 3);
    }

    #[test]
    fn transparent_source_over_uses_coverage_without_a_background_assumption() {
        let background = [0.2_f32, 0.4, 0.8, 1.0];
        let foreground = [1.0_f32, 0.0, 0.0, 0.25];
        let composited = source_over(background, foreground);
        assert!(composited[0] > background[0]);
        assert!(composited[2] < background[2]);
        assert_eq!(composited[3], 1.0);
    }

    #[test]
    fn font_size_is_independent_of_chart_geometry_supersampling() {
        let style = ("sans-serif", 26).into_font().color(&BLACK);
        let mask = coverage_mask("B axis", &style, RotatedTextQuality::default()).unwrap();
        assert_eq!(mask.scale, DEFAULT_ROTATED_TEXT_MASK_SCALE);
        assert!(mask.width > 26);
    }

    #[test]
    fn svg_commands_preserve_text_style_anchor_and_escape_xml() {
        let command = PreparedRotatedText {
            text: "B & <C> \"quoted\" 'axis' \u{03B1}".to_owned(),
            anchor: (100, 200),
            angle_rad: std::f64::consts::FRAC_PI_3,
            offset: (5, -7),
            style: AxisTextStyle::sans_serif(26, FontStyle::Bold, BLACK.to_rgba()),
        };
        let svg = svg_rotated_text_elements(&[command]);
        assert!(svg.contains("<text"));
        assert!(svg.contains("transform=\"rotate(60.000000 105 193)\""));
        assert!(svg.contains("font-family=\"sans-serif\""));
        assert!(svg.contains("font-size=\"26\""));
        assert!(svg.contains("font-weight=\"bold\""));
        assert!(svg.contains("&amp;"));
        assert!(svg.contains("&lt;C&gt;"));
        assert!(svg.contains("&quot;quoted&quot;"));
        assert!(svg.contains("&apos;axis&apos;"));
        assert!(svg.contains("\u{03B1}"));
    }

    #[test]
    fn svg_capture_records_commands_without_drawing_pixel_glyphs() {
        let (_, commands) = capture_rotated_text(|| {
            assert!(capture_svg_command(PreparedRotatedText {
                text: "C axis".to_owned(),
                anchor: (12, 15),
                angle_rad: 0.5,
                offset: (0, 0),
                style: AxisTextStyle::sans_serif(20, FontStyle::Normal, BLACK.to_rgba()),
            }));
        });
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].text(), "C axis");
    }
}
