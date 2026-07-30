use super::{Error, TernaryCartesian, TernaryGeometry, Tolerance};

/// An axis-aligned logical rectangle in the projected ternary plane.
///
/// The viewport is a clipping and zoom window. It has no drawing behaviour and
/// does not imply a visible frame or Cartesian axes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TernaryViewport {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
}

impl TernaryViewport {
    /// Construct a viewport using [`Tolerance::default`] for span validation.
    pub fn new(x_min: f64, x_max: f64, y_min: f64, y_max: f64) -> Result<Self, Error> {
        Self::new_with_tolerance(x_min, x_max, y_min, y_max, Tolerance::default())
    }

    /// Construct a viewport using an explicit minimum-span tolerance.
    pub fn new_with_tolerance(
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        tolerance: Tolerance,
    ) -> Result<Self, Error> {
        tolerance.validate()?;
        if !x_min.is_finite() || !x_max.is_finite() || !y_min.is_finite() || !y_max.is_finite() {
            return Err(Error::NonFiniteViewportBounds {
                x_min,
                x_max,
                y_min,
                y_max,
            });
        }

        let width = x_max - x_min;
        let height = y_max - y_min;
        if !width.is_finite()
            || !height.is_finite()
            || width <= tolerance.absolute
            || height <= tolerance.absolute
        {
            return Err(Error::InvalidViewport {
                x_min,
                x_max,
                y_min,
                y_max,
                minimum_span: tolerance.absolute,
            });
        }

        Ok(Self {
            x_min,
            x_max,
            y_min,
            y_max,
        })
    }

    /// Return the tight axis-aligned bounds of the complete projected triangle.
    ///
    /// ```
    /// use plotters_ternary::{TernaryGeometry, TernaryViewport};
    ///
    /// let viewport = TernaryViewport::full(TernaryGeometry::default());
    /// assert_eq!(viewport.x_min(), 0.0);
    /// assert_eq!(viewport.x_max(), 1.0);
    /// ```
    pub fn full(geometry: TernaryGeometry) -> Self {
        let vertices = geometry.vertices();
        let x_min = vertices
            .iter()
            .map(|vertex| vertex.x)
            .fold(f64::INFINITY, f64::min);
        let x_max = vertices
            .iter()
            .map(|vertex| vertex.x)
            .fold(f64::NEG_INFINITY, f64::max);
        let y_min = vertices
            .iter()
            .map(|vertex| vertex.y)
            .fold(f64::INFINITY, f64::min);
        let y_max = vertices
            .iter()
            .map(|vertex| vertex.y)
            .fold(f64::NEG_INFINITY, f64::max);
        Self {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }

    /// Minimum logical X coordinate.
    pub const fn x_min(self) -> f64 {
        self.x_min
    }

    /// Maximum logical X coordinate.
    pub const fn x_max(self) -> f64 {
        self.x_max
    }

    /// Minimum logical Y coordinate.
    pub const fn y_min(self) -> f64 {
        self.y_min
    }

    /// Maximum logical Y coordinate.
    pub const fn y_max(self) -> f64 {
        self.y_max
    }

    /// Logical viewport width.
    pub const fn width(self) -> f64 {
        self.x_max - self.x_min
    }

    /// Logical viewport height.
    pub const fn height(self) -> f64 {
        self.y_max - self.y_min
    }

    /// Return whether a point is inside or on the viewport within tolerance.
    pub fn contains(
        self,
        cartesian: TernaryCartesian,
        tolerance: Tolerance,
    ) -> Result<bool, Error> {
        Ok(self.classify(cartesian, tolerance)? != ViewportPointLocation::Outside)
    }

    /// Classify a point against the viewport independently of triangle geometry.
    pub fn classify(
        self,
        cartesian: TernaryCartesian,
        tolerance: Tolerance,
    ) -> Result<ViewportPointLocation, Error> {
        tolerance.validate()?;
        if !cartesian.x.is_finite() || !cartesian.y.is_finite() {
            return Err(Error::NonFiniteCartesian {
                x: cartesian.x,
                y: cartesian.y,
            });
        }

        let outside = (cartesian.x < self.x_min && !tolerance.is_close(cartesian.x, self.x_min))
            || (cartesian.x > self.x_max && !tolerance.is_close(cartesian.x, self.x_max))
            || (cartesian.y < self.y_min && !tolerance.is_close(cartesian.y, self.y_min))
            || (cartesian.y > self.y_max && !tolerance.is_close(cartesian.y, self.y_max));
        if outside {
            return Ok(ViewportPointLocation::Outside);
        }

        if tolerance.is_close(cartesian.x, self.x_min)
            || tolerance.is_close(cartesian.x, self.x_max)
            || tolerance.is_close(cartesian.y, self.y_min)
            || tolerance.is_close(cartesian.y, self.y_max)
        {
            Ok(ViewportPointLocation::Boundary)
        } else {
            Ok(ViewportPointLocation::Inside)
        }
    }
}

/// A point's location relative only to a logical viewport rectangle.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ViewportPointLocation {
    /// Strictly inside by more than the supplied tolerance.
    Inside,
    /// On or within tolerance of a viewport side.
    Boundary,
    /// Materially outside at least one viewport side.
    Outside,
}

/// A backend-independent allocated pixel rectangle.
///
/// `(x, y)` is the top-left corner. X increases rightward, Y increases
/// downward, and `width`/`height` are continuous coordinate extents.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PixelRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl PixelRect {
    /// Construct a non-empty allocated pixel rectangle.
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Result<Self, Error> {
        let rectangle = Self {
            x,
            y,
            width,
            height,
        };
        rectangle.validate()?;
        Ok(rectangle)
    }

    /// Validate a rectangle, including one built through its public fields.
    pub fn validate(self) -> Result<(), Error> {
        if self.width == 0 || self.height == 0 {
            return Err(Error::InvalidPixelRect {
                x: self.x,
                y: self.y,
                width: self.width,
                height: self.height,
            });
        }
        Ok(())
    }
}

/// A floating-point pixel-space point used before backend rounding.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PixelPoint {
    pub x: f64,
    pub y: f64,
}

impl PixelPoint {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Floating-point bounds actually occupied inside an allocated pixel rectangle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PixelBounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

impl PixelBounds {
    pub const fn width(self) -> f64 {
        self.x_max - self.x_min
    }

    pub const fn height(self) -> f64 {
        self.y_max - self.y_min
    }

    fn contains(self, point: PixelPoint) -> bool {
        point.x >= self.x_min
            && point.x <= self.x_max
            && point.y >= self.y_min
            && point.y <= self.y_max
    }
}

/// How a logical viewport uses an allocated pixel rectangle.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ViewportFit {
    /// Use one scale for X and Y, potentially leaving unused pixel space.
    #[default]
    PreserveAspect,
    /// Scale X and Y independently to fill the allocation.
    Stretch,
}

/// Placement of aspect-fitted bounds when unused pixel space remains.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ViewportAlignment {
    #[default]
    Center,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl ViewportAlignment {
    const fn factors(self) -> (f64, f64) {
        match self {
            Self::Center => (0.5, 0.5),
            Self::Top => (0.5, 0.0),
            Self::Bottom => (0.5, 1.0),
            Self::Left => (0.0, 0.5),
            Self::Right => (1.0, 0.5),
            Self::TopLeft => (0.0, 0.0),
            Self::TopRight => (1.0, 0.0),
            Self::BottomLeft => (0.0, 1.0),
            Self::BottomRight => (1.0, 1.0),
        }
    }
}

/// A reversible mapping between a logical viewport and floating pixel space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewportTransform {
    viewport: TernaryViewport,
    allocated: PixelRect,
    fitted: PixelBounds,
    fit: ViewportFit,
    alignment: ViewportAlignment,
}

impl ViewportTransform {
    /// Fit a logical viewport into an allocated pixel rectangle.
    ///
    /// ```
    /// use plotters_ternary::{
    ///     PixelRect, TernaryCartesian, TernaryViewport, ViewportAlignment,
    ///     ViewportFit, ViewportTransform,
    /// };
    ///
    /// let transform = ViewportTransform::new(
    ///     TernaryViewport::new(0.0, 2.0, 0.0, 1.0)?,
    ///     PixelRect::new(10, 20, 200, 100)?,
    ///     ViewportFit::PreserveAspect,
    ///     ViewportAlignment::Center,
    /// )?;
    /// let pixel = transform.logical_to_pixel(TernaryCartesian::new(0.0, 1.0))?;
    /// assert_eq!(pixel, plotters_ternary::PixelPoint::new(10.0, 20.0));
    /// # Ok::<(), plotters_ternary::Error>(())
    /// ```
    pub fn new(
        viewport: TernaryViewport,
        allocated: PixelRect,
        fit: ViewportFit,
        alignment: ViewportAlignment,
    ) -> Result<Self, Error> {
        allocated.validate()?;
        let allocated_width = f64::from(allocated.width);
        let allocated_height = f64::from(allocated.height);
        let allocated_x = f64::from(allocated.x);
        let allocated_y = f64::from(allocated.y);

        let (fitted_width, fitted_height) = match fit {
            ViewportFit::PreserveAspect => {
                let scale =
                    (allocated_width / viewport.width()).min(allocated_height / viewport.height());
                (viewport.width() * scale, viewport.height() * scale)
            }
            ViewportFit::Stretch => (allocated_width, allocated_height),
        };
        let (horizontal, vertical) = alignment.factors();
        let x_min = allocated_x + (allocated_width - fitted_width) * horizontal;
        let y_min = allocated_y + (allocated_height - fitted_height) * vertical;
        let fitted = PixelBounds {
            x_min,
            x_max: x_min + fitted_width,
            y_min,
            y_max: y_min + fitted_height,
        };

        Ok(Self {
            viewport,
            allocated,
            fitted,
            fit,
            alignment,
        })
    }

    pub const fn viewport(self) -> TernaryViewport {
        self.viewport
    }

    pub const fn allocated_pixel_rect(self) -> PixelRect {
        self.allocated
    }

    pub const fn fitted_pixel_bounds(self) -> PixelBounds {
        self.fitted
    }

    pub const fn fit(self) -> ViewportFit {
        self.fit
    }

    pub const fn alignment(self) -> ViewportAlignment {
        self.alignment
    }

    /// Map a finite logical point to floating pixel coordinates.
    pub fn logical_to_pixel(self, logical: TernaryCartesian) -> Result<PixelPoint, Error> {
        if !logical.x.is_finite() || !logical.y.is_finite() {
            return Err(Error::NonFiniteCartesian {
                x: logical.x,
                y: logical.y,
            });
        }
        let x_fraction = (logical.x - self.viewport.x_min) / self.viewport.width();
        let y_fraction = (self.viewport.y_max - logical.y) / self.viewport.height();
        Ok(PixelPoint::new(
            self.fitted.x_min + x_fraction * self.fitted.width(),
            self.fitted.y_min + y_fraction * self.fitted.height(),
        ))
    }

    /// Map a finite pixel point to logical coordinates, including points outside the fit.
    pub fn pixel_to_logical(self, pixel: PixelPoint) -> Result<TernaryCartesian, Error> {
        if !pixel.x.is_finite() || !pixel.y.is_finite() {
            return Err(Error::NonFinitePixelCoordinate {
                x: pixel.x,
                y: pixel.y,
            });
        }
        let x_fraction = (pixel.x - self.fitted.x_min) / self.fitted.width();
        let y_fraction = (pixel.y - self.fitted.y_min) / self.fitted.height();
        Ok(TernaryCartesian::new(
            self.viewport.x_min + x_fraction * self.viewport.width(),
            self.viewport.y_max - y_fraction * self.viewport.height(),
        ))
    }

    /// Map only pixel points inside the fitted bounds.
    pub fn pixel_to_logical_checked(
        self,
        pixel: PixelPoint,
    ) -> Result<Option<TernaryCartesian>, Error> {
        if !pixel.x.is_finite() || !pixel.y.is_finite() {
            return Err(Error::NonFinitePixelCoordinate {
                x: pixel.x,
                y: pixel.y,
            });
        }
        if self.fitted.contains(pixel) {
            self.pixel_to_logical(pixel).map(Some)
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EQUILATERAL_TRIANGLE_HEIGHT, TriangleOrientation, VertexOrder};

    const TOLERANCE: Tolerance = Tolerance {
        absolute: 1.0e-9,
        relative: 1.0e-9,
    };
    const EPSILON: f64 = 1.0e-10;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < EPSILON,
            "{actual} != {expected}"
        );
    }

    fn test_viewport() -> TernaryViewport {
        TernaryViewport::new(0.0, 2.0, 0.0, 1.0).unwrap()
    }

    #[test]
    fn viewport_construction_and_full_bounds_cover_both_orientations() {
        let arbitrary = TernaryViewport::new(-2.0, 3.0, -4.0, 5.0).unwrap();
        assert_eq!(arbitrary.width(), 5.0);
        assert_eq!(arbitrary.height(), 9.0);

        let upward = TernaryViewport::full(TernaryGeometry::default());
        assert_eq!((upward.x_min(), upward.x_max()), (0.0, 1.0));
        assert_eq!(
            (upward.y_min(), upward.y_max()),
            (0.0, EQUILATERAL_TRIANGLE_HEIGHT)
        );

        let downward = TernaryViewport::full(TernaryGeometry::new(
            TriangleOrientation::Down,
            VertexOrder::default(),
        ));
        assert_eq!((downward.x_min(), downward.x_max()), (0.0, 1.0));
        assert_eq!(
            (downward.y_min(), downward.y_max()),
            (-EQUILATERAL_TRIANGLE_HEIGHT, 0.0)
        );

        assert!(TernaryViewport::new(10.0, 11.0, 10.0, 11.0).is_ok());
    }

    #[test]
    fn viewport_validation_rejects_reversed_degenerate_and_non_finite_bounds() {
        for result in [
            TernaryViewport::new(1.0, 0.0, 0.0, 1.0),
            TernaryViewport::new(0.0, 1.0, 1.0, 0.0),
            TernaryViewport::new(0.0, 0.0, 0.0, 1.0),
            TernaryViewport::new(0.0, 1.0, 0.0, 0.0),
            TernaryViewport::new_with_tolerance(0.0, 5.0e-10, 0.0, 1.0, TOLERANCE),
            TernaryViewport::new_with_tolerance(0.0, 1.0, 0.0, 5.0e-10, TOLERANCE),
        ] {
            assert!(matches!(result, Err(Error::InvalidViewport { .. })));
        }

        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                TernaryViewport::new(value, 1.0, 0.0, 1.0),
                Err(Error::NonFiniteViewportBounds { .. })
            ));
        }
    }

    #[test]
    fn viewport_classification_is_independent_and_boundary_aware() {
        let viewport = TernaryViewport::new(0.2, 0.8, 0.1, 0.6).unwrap();
        assert_eq!(
            viewport
                .classify(TernaryCartesian::new(0.5, 0.3), TOLERANCE)
                .unwrap(),
            ViewportPointLocation::Inside
        );
        assert_eq!(
            viewport
                .classify(TernaryCartesian::new(0.2, 0.3), TOLERANCE)
                .unwrap(),
            ViewportPointLocation::Boundary
        );
        assert!(
            viewport
                .contains(TernaryCartesian::new(0.2 - 5.0e-10, 0.3), TOLERANCE)
                .unwrap()
        );
        assert!(
            !viewport
                .contains(TernaryCartesian::new(0.2 - 1.0e-5, 0.3), TOLERANCE)
                .unwrap()
        );
    }

    #[test]
    fn transform_maps_corners_centre_and_inverts_y() {
        let transform = ViewportTransform::new(
            test_viewport(),
            PixelRect::new(10, 20, 200, 100).unwrap(),
            ViewportFit::PreserveAspect,
            ViewportAlignment::Center,
        )
        .unwrap();
        let fitted = transform.fitted_pixel_bounds();
        assert_eq!(
            fitted,
            PixelBounds {
                x_min: 10.0,
                x_max: 210.0,
                y_min: 20.0,
                y_max: 120.0
            }
        );
        assert_eq!(
            transform
                .logical_to_pixel(TernaryCartesian::new(0.0, 1.0))
                .unwrap(),
            PixelPoint::new(10.0, 20.0)
        );
        assert_eq!(
            transform
                .logical_to_pixel(TernaryCartesian::new(2.0, 0.0))
                .unwrap(),
            PixelPoint::new(210.0, 120.0)
        );
        assert_eq!(
            transform
                .logical_to_pixel(TernaryCartesian::new(1.0, 0.5))
                .unwrap(),
            PixelPoint::new(110.0, 70.0)
        );
    }

    #[test]
    fn preserve_aspect_handles_wide_tall_portrait_landscape_and_offsets() {
        let cases = [
            (
                PixelRect::new(0, 0, 300, 100).unwrap(),
                PixelBounds {
                    x_min: 50.0,
                    x_max: 250.0,
                    y_min: 0.0,
                    y_max: 100.0,
                },
            ),
            (
                PixelRect::new(0, 0, 200, 200).unwrap(),
                PixelBounds {
                    x_min: 0.0,
                    x_max: 200.0,
                    y_min: 50.0,
                    y_max: 150.0,
                },
            ),
            (
                PixelRect::new(0, 0, 100, 300).unwrap(),
                PixelBounds {
                    x_min: 0.0,
                    x_max: 100.0,
                    y_min: 125.0,
                    y_max: 175.0,
                },
            ),
            (
                PixelRect::new(7, 11, 400, 100).unwrap(),
                PixelBounds {
                    x_min: 107.0,
                    x_max: 307.0,
                    y_min: 11.0,
                    y_max: 111.0,
                },
            ),
        ];
        for (allocated, expected) in cases {
            let transform = ViewportTransform::new(
                test_viewport(),
                allocated,
                ViewportFit::PreserveAspect,
                ViewportAlignment::Center,
            )
            .unwrap();
            assert_eq!(transform.fitted_pixel_bounds(), expected);
            assert_close(
                transform.fitted_pixel_bounds().width() / transform.fitted_pixel_bounds().height(),
                2.0,
            );
        }
    }

    #[test]
    fn every_alignment_places_unused_horizontal_and_vertical_space() {
        let alignments = [
            (ViewportAlignment::Center, 0.5, 0.5),
            (ViewportAlignment::Top, 0.5, 0.0),
            (ViewportAlignment::Bottom, 0.5, 1.0),
            (ViewportAlignment::Left, 0.0, 0.5),
            (ViewportAlignment::Right, 1.0, 0.5),
            (ViewportAlignment::TopLeft, 0.0, 0.0),
            (ViewportAlignment::TopRight, 1.0, 0.0),
            (ViewportAlignment::BottomLeft, 0.0, 1.0),
            (ViewportAlignment::BottomRight, 1.0, 1.0),
        ];
        for (alignment, horizontal, vertical) in alignments {
            let wide = ViewportTransform::new(
                test_viewport(),
                PixelRect::new(10, 20, 300, 100).unwrap(),
                ViewportFit::PreserveAspect,
                alignment,
            )
            .unwrap();
            assert_close(wide.fitted_pixel_bounds().x_min, 10.0 + 100.0 * horizontal);

            let tall = ViewportTransform::new(
                test_viewport(),
                PixelRect::new(10, 20, 200, 200).unwrap(),
                ViewportFit::PreserveAspect,
                alignment,
            )
            .unwrap();
            assert_close(tall.fitted_pixel_bounds().y_min, 20.0 + 100.0 * vertical);
        }
    }

    #[test]
    fn stretch_fills_allocation_and_round_trips_inside_and_outside_points() {
        let transform = ViewportTransform::new(
            test_viewport(),
            PixelRect::new(-20, 30, 300, 200).unwrap(),
            ViewportFit::Stretch,
            ViewportAlignment::BottomRight,
        )
        .unwrap();
        assert_eq!(
            transform.fitted_pixel_bounds(),
            PixelBounds {
                x_min: -20.0,
                x_max: 280.0,
                y_min: 30.0,
                y_max: 230.0
            }
        );

        for logical in [
            TernaryCartesian::new(0.25, 0.75),
            TernaryCartesian::new(1.0, 0.5),
            TernaryCartesian::new(2.5, -0.5),
        ] {
            let pixel = transform.logical_to_pixel(logical).unwrap();
            let recovered = transform.pixel_to_logical(pixel).unwrap();
            assert_close(recovered.x, logical.x);
            assert_close(recovered.y, logical.y);
        }

        let outside_pixel = PixelPoint::new(-100.0, -100.0);
        assert!(transform.pixel_to_logical(outside_pixel).is_ok());
        assert_eq!(
            transform.pixel_to_logical_checked(outside_pixel).unwrap(),
            None
        );
    }

    #[test]
    fn invalid_pixel_rectangles_and_non_finite_mapping_inputs_are_rejected() {
        assert!(matches!(
            PixelRect::new(0, 0, 0, 10),
            Err(Error::InvalidPixelRect { .. })
        ));
        assert!(matches!(
            ViewportTransform::new(
                test_viewport(),
                PixelRect {
                    x: 0,
                    y: 0,
                    width: 10,
                    height: 0
                },
                ViewportFit::PreserveAspect,
                ViewportAlignment::Center,
            ),
            Err(Error::InvalidPixelRect { .. })
        ));
        let transform = ViewportTransform::new(
            test_viewport(),
            PixelRect::new(0, 0, 100, 100).unwrap(),
            ViewportFit::PreserveAspect,
            ViewportAlignment::Center,
        )
        .unwrap();
        assert!(matches!(
            transform.logical_to_pixel(TernaryCartesian::new(f64::NAN, 0.0)),
            Err(Error::NonFiniteCartesian { .. })
        ));
        assert!(matches!(
            transform.pixel_to_logical(PixelPoint::new(f64::INFINITY, 0.0)),
            Err(Error::NonFinitePixelCoordinate { .. })
        ));
    }
}
