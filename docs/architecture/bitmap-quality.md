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
once at final resolution and are never filtered. Invisible scaled font metrics
may reserve equivalent layout space in the high-resolution geometry pass, but
no glyph pixels are emitted there. Logical ternary coordinates and viewport
coordinates are never scaled. No high-resolution temporary file is created.

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

This module is deliberately a portable, removable fallback, not a permanent
crate antialiasing API and not "native antialiasing". If a future Plotters
bitmap backend exposes native antialiasing, the output helper can select it or
be removed without changing ternary chart construction or any public geometry,
series, or viewport type. The `image` dependency is development-only.