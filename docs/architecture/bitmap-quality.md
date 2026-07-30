# Temporary bitmap-quality fallback

Plotters 0.3.7's standard `BitMapBackend` does not expose a general
antialiasing toggle. The crate's geometry, viewport, chart, and series APIs
therefore remain independent of bitmap-quality policy and of the `image` crate.

The shared example-only module `examples/output_support/mod.rs` provides:

```rust
pub enum BitmapQuality {
    Native,
    Supersampled { factor: u32 },
}

pub struct BitmapRenderOptions {
    pub output_size: (u32, u32),
    pub quality: BitmapQuality,
}
```

The helper always uses two explicit closures. The geometry closure draws into
an RGB buffer using scale 1 for `Native` or the configured factor for
`Supersampled`; it contains boundaries, grids, data paths, markers, legend
symbols, and other non-text shapes, but no visible text. Supersampled geometry
is resized with `image::imageops::FilterType::Lanczos3`.

The text closure then receives a `BitMapBackend` over the already-downsampled
final RGB image. Captions, corner labels, axis names, and legend text are drawn
once at final resolution and are never filtered. The geometry pass reserves a
caption strip measured at final resolution before scaling it, but emits no
caption glyphs. The shared series-example legend adapter also reconciles
Plotters' high-resolution integer callback anchors with the final text rows,
so symbols and labels retain the same centres. Logical ternary coordinates and
viewport coordinates are never scaled. No high-resolution temporary file is
created.

Factors must be in `1..=4`; zero and larger values are rejected, scaled
arithmetic is checked for overflow, and the RGB allocation is limited to 512
MiB. A 2x factor is the faster quality option, 3x is the current default
balance, and 4x gives higher quality at substantially greater memory cost.

SVG generation remains entirely vector-based and is never converted from a
raster image. The example helper renders independent geometry and text SVG
fragments with `SVGBackend::with_string`, then wraps the unchanged Plotters
primitives in narrowly scoped groups. `ternary-geometry` carries
`shape-rendering="geometricPrecision"`, round caps, and round joins;
`ternary-text` carries no forced text-rendering attribute. The wrapper never
interpolates, subdivides, resamples, or rewrites any path coordinate. SVG is
still the preferred publication-quality output.

## Arbitrary-angle axis text

Geometry supersampling intentionally excludes all text. Sloped B/C axis names
are also excluded from that layer: their prepared final-layout command is
captured during the bitmap text pass and rendered at the final output resolution
with a transparent coverage mask. The default text-mask scale is 4x (bounded to
1x through 4x), followed by inverse-mapped bilinear rotation and source-over
alpha compositing. This improves sloped glyph edges without rescaling nominal
font sizes, offsets, or anchors, and it works over any background colour.

The SVG adapter consumes the identical prepared command as native rotated SVG
`<text>` with no raster image or per-pixel glyph rectangles. Horizontal text,
captions, tick labels, corner labels, and legend text remain Plotters-native
text. The helper performs no global SVG text replacement and changes no
geometry path coordinates.

This module is deliberately a portable, removable fallback, not a permanent
crate antialiasing API and not "native antialiasing". If a future Plotters
bitmap backend exposes native antialiasing, the output helper can select it or
be removed without changing ternary chart construction or any public geometry,
series, or viewport type. The `image` dependency is development-only.
