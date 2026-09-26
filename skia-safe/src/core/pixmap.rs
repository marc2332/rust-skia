//! Provides a utility to pair [`ImageInfo`] with pixels and row bytes. [`Pixmap`] is a low level
//! class which provides convenience functions to access raster destinations. [`crate::Canvas`]
//! cannot draw a [`Pixmap`], nor does [`Pixmap`] provide a direct drawing destination.
//!
//! Use [`crate::Bitmap`] to draw pixels referenced by [`Pixmap`]; use [`crate::Surface`] to draw
//! into pixels referenced by [`Pixmap`].
//!
//! [`Pixmap`] does not try to manage the lifetime of the pixel memory. Use [`crate::PixelRef`] to
//! manage pixel memory; [`crate::PixelRef`] is safe across threads.

use crate::{
    AlphaType, Color, Color4f, ColorSpace, ColorType, IPoint, IRect, ISize, ImageInfo,
    SamplingOptions, prelude::*,
};
use skia_bindings::{self as sb, SkPixmap};
use std::{ffi::c_void, fmt, marker::PhantomData, mem, os::raw, ptr, slice};

#[repr(transparent)]
pub struct Pixmap<'a> {
    inner: Handle<SkPixmap>,
    pd: PhantomData<&'a mut [u8]>,
}

impl NativeDrop for SkPixmap {
    fn drop(&mut self) {
        unsafe { sb::C_SkPixmap_destruct(self) }
    }
}

impl Default for Pixmap<'_> {
    fn default() -> Self {
        Self::from_native_c(SkPixmap {
            fPixels: ptr::null(),
            fRowBytes: 0,
            fInfo: construct(|ii| unsafe { sb::C_SkImageInfo_Construct(ii) }),
        })
    }
}

impl fmt::Debug for Pixmap<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pixmap")
            .field("row_bytes", &self.row_bytes())
            .field("info", self.info())
            .finish()
    }
}

impl<'pixels> Pixmap<'pixels> {
    /// Creates a [`Pixmap`] from `info` width, height, alpha type, and color type. `pixels` points
    /// to pixels. `row_bytes` should be `info.width()` times `info.bytes_per_pixel()`, or larger.
    ///
    /// No parameter checking is performed; it is up to the caller to ensure that `pixels` and
    /// `row_bytes` agree with `info`.
    ///
    /// The memory lifetime of pixels is managed by the caller. When the [`Pixmap`] goes out of
    /// scope, the pixel address is unaffected.
    ///
    /// The [`Pixmap`] may be later modified by [`Self::reset()`] to change its size, pixel type, or
    /// storage.
    ///
    /// - `info` width, height, alpha type, color type of the image info
    /// - `pixels` pointer to pixels allocated by the caller
    /// - `row_bytes` size of one row of pixels; width times pixel size, or larger
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Pixmap_reset_2>
    pub fn new(info: &ImageInfo, pixels: &'pixels mut [u8], row_bytes: usize) -> Option<Self> {
        if row_bytes < info.min_row_bytes() {
            return None;
        }
        if pixels.len() < info.compute_byte_size(row_bytes) {
            return None;
        }

        Some(Pixmap::from_native_c(SkPixmap {
            fPixels: pixels.as_mut_ptr() as _,
            fRowBytes: row_bytes,
            fInfo: info.native().clone(),
        }))
    }

    /// Sets width, height, row bytes to zero; pixel address to null; color type to
    /// [`ColorType::Unknown`]; and alpha type to [`AlphaType::Unknown`].
    ///
    /// The prior pixels are unaffected; it is up to the caller to release pixels memory if desired.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Pixmap_reset>
    pub fn reset(&mut self) -> &mut Self {
        unsafe { self.native_mut().reset() }
        self
    }

    // TODO: reset() function that re-borrows pixels?

    /// Changes the [`ColorSpace`] in the [`ImageInfo`]; preserves width, height, alpha type, and
    /// color type, and leaves the pixel address and row bytes unchanged. The [`ColorSpace`]
    /// reference count is incremented.
    ///
    /// - `color_space` color space moved to the image info
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Pixmap_setColorSpace>
    pub fn set_color_space(&mut self, color_space: impl Into<Option<ColorSpace>>) -> &mut Self {
        unsafe {
            sb::C_SkPixmap_setColorSpace(self.native_mut(), color_space.into().into_ptr_or_null())
        }
        self
    }

    /// Sets the subset width, height, pixel address to the intersection of the [`Pixmap`] with
    /// `area`, if the intersection is not empty; and returns true. Otherwise, leaves the subset
    /// unchanged and returns false.
    ///
    /// - `area` bounds to intersect with the pixmap
    #[must_use]
    pub fn extract_subset(&self, area: impl AsRef<IRect>) -> Option<Self> {
        let mut pixmap = Pixmap::default();
        unsafe {
            self.native()
                .extractSubset(pixmap.native_mut(), area.as_ref().native())
        }
        .then_some(pixmap)
    }

    /// Returns the width, height, alpha type, color type, and color space.
    pub fn info(&self) -> &ImageInfo {
        ImageInfo::from_native_ref(&self.native().fInfo)
    }

    /// Returns the row bytes, the interval from one pixel row to the next. Row bytes is at least as
    /// large as `width() * info().bytes_per_pixel()`.
    ///
    /// Returns zero if the color type is [`ColorType::Unknown`]. It is up to the bitmap creator to
    /// ensure that row bytes is a useful value.
    pub fn row_bytes(&self) -> usize {
        self.native().fRowBytes
    }

    /// Returns the pixel address, the base address corresponding to the pixel origin.
    ///
    /// It is up to the pixmap creator to ensure that the pixel address is a useful value.
    pub fn addr(&self) -> *const c_void {
        self.native().fPixels
    }

    /// Returns the pixel count in each pixel row. Should be equal to or less than
    /// `row_bytes() / info().bytes_per_pixel()`.
    pub fn width(&self) -> i32 {
        self.info().width()
    }

    /// Returns the pixel row count.
    pub fn height(&self) -> i32 {
        self.info().height()
    }

    /// Returns true if the pixmap is empty (from its [`ImageInfo`]).
    pub fn is_empty(&self) -> bool {
        self.info().is_empty()
    }

    /// Returns the dimensions of the pixmap (from its [`ImageInfo`]).
    pub fn dimensions(&self) -> ISize {
        self.info().dimensions()
    }

    /// Returns the color type.
    pub fn color_type(&self) -> ColorType {
        self.info().color_type()
    }

    /// Returns the alpha type.
    pub fn alpha_type(&self) -> AlphaType {
        self.info().alpha_type()
    }

    /// Returns the [`ColorSpace`], the range of colors, associated with the [`ImageInfo`]. The
    /// returned [`ColorSpace`] is immutable.
    pub fn color_space(&self) -> Option<ColorSpace> {
        ColorSpace::from_unshared_ptr(unsafe { self.native().colorSpace() })
    }

    /// Returns true if the alpha type is [`AlphaType::Opaque`]. Does not check if the color type
    /// allows alpha, or if any pixel value has transparency.
    pub fn is_opaque(&self) -> bool {
        self.alpha_type().is_opaque()
    }

    /// Returns the integral rectangle from the origin to [`Self::width()`] and [`Self::height()`].
    pub fn bounds(&self) -> IRect {
        IRect::from_wh(self.width(), self.height())
    }

    /// Returns the number of pixels that fit on a row. Should be greater than or equal to
    /// [`Self::width()`].
    pub fn row_bytes_as_pixels(&self) -> usize {
        self.row_bytes() >> self.shift_per_pixel()
    }

    /// Returns the bit shift converting row bytes to row pixels. Returns zero for
    /// [`ColorType::Unknown`].
    pub fn shift_per_pixel(&self) -> usize {
        self.info().shift_per_pixel()
    }

    /// Returns the minimum memory required for pixel storage. Does not include unused memory on
    /// the last row when [`Self::row_bytes_as_pixels()`] exceeds [`Self::width()`]. Returns
    /// `usize::MAX` if the result does not fit in `usize`. Returns zero if [`Self::height()`] or
    /// [`Self::width()`] is 0. Returns [`Self::height()`] times [`Self::row_bytes()`] if the color
    /// type is [`ColorType::Unknown`].
    pub fn compute_byte_size(&self) -> usize {
        self.info().compute_byte_size(self.row_bytes())
    }

    /// Returns true if all pixels are opaque. The color type determines how pixels are encoded, and
    /// whether a pixel describes alpha. Returns true for color types without alpha in each pixel;
    /// for other color types, returns true if all pixels have alpha values equivalent to 1.0 or
    /// greater.
    ///
    /// For [`ColorType::RGB565`] or [`ColorType::Gray8`]: always returns true. For
    /// [`ColorType::Alpha8`], [`ColorType::BGRA8888`], [`ColorType::RGBA8888`]: returns true if all
    /// pixel alpha values are 255. For [`ColorType::ARGB4444`]: returns true if all pixel alpha
    /// values are 15. For [`ColorType::RGBAF16`]: returns true if all pixel alpha values are 1.0 or
    /// greater.
    ///
    /// Returns false for [`ColorType::Unknown`].
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Pixmap_computeIsOpaque>
    pub fn compute_is_opaque(&self) -> bool {
        unsafe { self.native().computeIsOpaque() }
    }

    /// Returns the pixel at (`p.x`, `p.y`) as an unpremultiplied color. Returns black with alpha if
    /// the color type is [`ColorType::Alpha8`].
    ///
    /// Input is not validated: out of bounds values of `p` trigger an assert if built with debug
    /// defined; and returns undefined values or may crash if release is defined. Fails if the color
    /// type is [`ColorType::Unknown`] or the pixel address is null.
    ///
    /// The [`ColorSpace`] in the [`ImageInfo`] is ignored. Some color precision may be lost in the
    /// conversion to unpremultiplied color; original pixel data may have additional precision.
    ///
    /// - `p` pixel position
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Pixmap_getColor>
    pub fn get_color(&self, p: impl Into<IPoint>) -> Color {
        let p = p.into();
        self.assert_pixel_exists(p);
        Color::from_native_c(unsafe { self.native().getColor(p.x, p.y) })
    }

    /// Returns the pixel at (`p.x`, `p.y`) as an unpremultiplied color as a [`Color4f`]. Returns
    /// black with alpha if the color type is [`ColorType::Alpha8`].
    ///
    /// Input is not validated: out of bounds values of `p` trigger an assert if built with debug
    /// defined; and returns undefined values or may crash if release is defined. Fails if the color
    /// type is [`ColorType::Unknown`] or the pixel address is null.
    ///
    /// The [`ColorSpace`] in the [`ImageInfo`] is ignored. Some color precision may be lost in the
    /// conversion to unpremultiplied color; original pixel data may have additional precision,
    /// though this is less likely than for [`Self::get_color()`]. Rounding errors may occur if the
    /// underlying type has lower precision.
    ///
    /// - `p` pixel position
    pub fn get_color_4f(&self, p: impl Into<IPoint>) -> Color4f {
        let p = p.into();
        self.assert_pixel_exists(p);
        Color4f::from_native_c(unsafe { self.native().getColor4f(p.x, p.y) })
    }

    /// Looks up the pixel at (`p.x`, `p.y`) and returns its alpha component, normalized to [0..1].
    /// This is roughly equivalent to `get_color().a()`, but can be more efficient (and more precise
    /// if the pixels store more than 8 bits per component).
    ///
    /// - `p` pixel position
    pub fn get_alpha_f(&self, p: impl Into<IPoint>) -> f32 {
        let p = p.into();
        self.assert_pixel_exists(p);
        unsafe { self.native().getAlphaf(p.x, p.y) }
    }

    // Helper to test if the pixel does exist physically in memory.
    fn assert_pixel_exists(&self, p: impl Into<IPoint>) {
        let p = p.into();
        assert!(!self.addr().is_null());
        assert!(p.x >= 0 && p.x < self.width());
        assert!(p.y >= 0 && p.y < self.height());
    }

    /// Returns the readable pixel address at (`p.x`, `p.y`). Returns null if the pixel ref is null.
    ///
    /// Input is not validated: out of bounds values of `p` trigger an assert if built with debug
    /// defined. Returns null if the color type is [`ColorType::Unknown`].
    ///
    /// Performs a lookup of pixel size; for better performance, call one of the typed address
    /// accessors.
    ///
    /// - `p` pixel position
    pub fn addr_at(&self, p: impl Into<IPoint>) -> *const c_void {
        let p = p.into();
        self.assert_pixel_exists(p);
        unsafe {
            (self.addr() as *const raw::c_char).add(self.info().compute_offset(p, self.row_bytes()))
                as _
        }
    }

    // TODO: addr8(), addr16(), addr32(), addr64(), addrF16(),
    //       addr8_at(), addr16_at(), addr32_at(), addr64_at(), addrF16_at()

    /// Returns the writable base pixel address.
    pub fn writable_addr(&self) -> *mut c_void {
        self.addr() as _
    }

    /// Returns the writable pixel address at (`p.x`, `p.y`).
    ///
    /// Input is not validated: out of bounds values of `p` trigger an assert if built with debug
    /// defined. Returns zero if the color type is [`ColorType::Unknown`].
    ///
    /// - `p` pixel position
    pub fn writable_addr_at(&self, p: impl Into<IPoint>) -> *mut c_void {
        self.addr_at(p) as _
    }

    // TODO: writable_addr8
    // TODO: writable_addr16
    // TODO: writable_addr32
    // TODO: writable_addr64
    // TODO: writable_addrF16

    /// Copies a rectangle of pixels to `pixels`. Copy starts at (`src.x`, `src.y`), and does not
    /// exceed the pixmap (`width()`, `height()`).
    ///
    /// `dst_info` specifies width, height, color type, alpha type, and color space of the
    /// destination. `dst_row_bytes` specifies the gap from one destination row to the next. Returns
    /// true if pixels are copied. Returns false if `dst_info` address equals null, or `dst_row_bytes`
    /// is less than `dst_info.min_row_bytes()`.
    ///
    /// Pixels are copied only if pixel conversion is possible. If the pixmap color type is
    /// [`ColorType::Gray8`] or [`ColorType::Alpha8`], `dst_info.color_type()` must match. If the
    /// pixmap color type is [`ColorType::Gray8`], `dst_info.color_space()` must match. If the pixmap
    /// alpha type is [`AlphaType::Opaque`], `dst_info.alpha_type()` must match. If the pixmap color
    /// space is `None`, `dst_info.color_space()` must match. Returns false if pixel conversion is
    /// not possible.
    ///
    /// `src.x` and `src.y` may be negative to copy only the top or left of the source. Returns
    /// false if the pixmap width() or height() is zero or negative. Returns false if `abs(src.x)`
    /// is greater than or equal to the pixmap width(), or if `abs(src.y)` is greater than or equal
    /// to the pixmap height().
    ///
    /// - `dst_info` destination width, height, color type, alpha type, color space
    /// - `pixels` destination pixel storage
    /// - `dst_row_bytes` destination row length
    /// - `src` source position
    pub fn read_pixels<P>(
        &self,
        dst_info: &ImageInfo,
        pixels: &mut [P],
        dst_row_bytes: usize,
        src: impl Into<IPoint>,
    ) -> bool {
        if !dst_info.valid_pixels(dst_row_bytes, pixels) {
            return false;
        }

        let src = src.into();

        unsafe {
            self.native().readPixels(
                dst_info.native(),
                pixels.as_mut_ptr() as _,
                dst_row_bytes,
                src.x,
                src.y,
            )
        }
    }

    /// Access the underlying pixels as a byte array. This is a rust-skia specific function.
    pub fn bytes(&self) -> Option<&'pixels [u8]> {
        let addr = self.addr().into_non_null()?;
        let len = self.compute_byte_size();
        Some(unsafe { slice::from_raw_parts(addr.as_ptr() as *const _, len) })
    }

    pub fn bytes_mut(&mut self) -> Option<&'pixels mut [u8]> {
        let addr = self.writable_addr().into_non_null()?;
        let len = self.compute_byte_size();
        Some(unsafe { slice::from_raw_parts_mut(addr.as_ptr() as *mut u8, len) })
    }

    /// Access the underlying pixels. This is a rust-skia specific function.
    ///
    /// The `Pixel` type must implement the _unsafe_ trait [`Pixel`] and must return `true` in
    /// [`Pixel::matches_color_type()`] when matched against the [`ColorType`] of this Pixmap's
    /// pixels.
    pub fn pixels<P: Pixel>(&self) -> Option<&'pixels [P]> {
        let addr = self.addr().into_non_null()?;

        let info = self.info();
        let ct = info.color_type();
        let pixel_size = mem::size_of::<P>();

        if info.bytes_per_pixel() == pixel_size && P::matches_color_type(ct) {
            let len = self.compute_byte_size() / pixel_size;
            return Some(unsafe { slice::from_raw_parts(addr.as_ptr() as *const _, len) });
        }

        None
    }

    /// Copies a rectangle of pixels to `dst`. Copy starts at `src`, and does not exceed the pixmap
    /// (`width()`, `height()`). `dst` specifies width, height, color type, alpha type, and color
    /// space of the destination. Returns true if pixels are copied. Returns false if the
    /// destination address is null, or `dst.row_bytes()` is less than `dst.info().min_row_bytes()`.
    ///
    /// Pixels are copied only if pixel conversion is possible. If the pixmap color type is
    /// [`ColorType::Gray8`] or [`ColorType::Alpha8`], `dst.info().color_type()` must match. If the
    /// pixmap color type is [`ColorType::Gray8`], `dst.info().color_space()` must match. If the
    /// pixmap alpha type is [`AlphaType::Opaque`], `dst.info().alpha_type()` must match. If the
    /// pixmap color space is `None`, `dst.info().color_space()` must match. Returns false if pixel
    /// conversion is not possible.
    ///
    /// `src.x` and `src.y` may be negative to copy only the top or left of the source. Returns
    /// false if the pixmap width() or height() is zero or negative. Returns false if `abs(src.x)`
    /// is greater than or equal to the pixmap width(), or if `abs(src.y)` is greater than or equal
    /// to the pixmap height().
    ///
    /// - `dst` image info and pixel address to write to
    /// - `src` source position
    pub fn read_pixels_to_pixmap(&self, dst: &mut Pixmap, src: impl Into<IPoint>) -> bool {
        let Some(dst_bytes) = dst.bytes_mut() else {
            return false;
        };
        self.read_pixels(dst.info(), dst_bytes, dst.row_bytes(), src)
    }

    /// Copies the pixmap to `dst`, scaling pixels to fit `dst.width()` and `dst.height()`, and
    /// converting pixels to match `dst.color_type()` and `dst.alpha_type()`. Returns true if pixels
    /// are copied. Returns false if the destination address is null, or the destination row bytes
    /// is less than the destination minimum row bytes.
    ///
    /// Pixels are copied only if pixel conversion is possible. If the pixmap color type is
    /// [`ColorType::Gray8`] or [`ColorType::Alpha8`], the destination color type must match. If the
    /// pixmap color type is [`ColorType::Gray8`], the destination color space must match. If the
    /// pixmap alpha type is [`AlphaType::Opaque`], the destination alpha type must match. If the
    /// pixmap color space is `None`, the destination color space must match. Returns false if pixel
    /// conversion is not possible.
    ///
    /// Returns false if the pixmap width() or height() is zero or negative.
    ///
    /// - `dst` image info and pixel address to write to
    /// - `sampling` sampling options
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Pixmap_scalePixels>
    pub fn scale_pixels(&self, dst: &mut Pixmap, sampling: impl Into<SamplingOptions>) -> bool {
        let sampling = sampling.into();
        unsafe { self.native().scalePixels(dst.native(), sampling.native()) }
    }

    /// Writes `color` to pixels bounded by `subset`; returns true on success. Returns false if the
    /// color type is [`ColorType::Unknown`], or if `subset` does not intersect the bounds. If
    /// `subset` is `None`, writes color to pixels inside the bounds.
    ///
    /// - `color` sRGB unpremultiplied color to write
    /// - `subset` bounding integer rectangle of pixels to write
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Pixmap_erase>
    pub fn erase(&mut self, color: impl Into<Color>, subset: Option<&IRect>) -> bool {
        let color = color.into().into_native();
        unsafe {
            match subset {
                Some(subset) => self.native().erase(color, subset.native()),
                None => self.native().erase(color, self.bounds().native()),
            }
        }
    }

    /// Writes `color` to pixels bounded by `subset`; returns true on success. If `subset` is `None`,
    /// writes color to pixels inside the bounds. Returns false if the color type is
    /// [`ColorType::Unknown`], if `subset` is not `None` and does not intersect the bounds, or if
    /// `subset` is `None` and the bounds is empty.
    ///
    /// - `color` unpremultiplied color to write
    /// - `subset` bounding integer rectangle of pixels to write; may be `None`
    pub fn erase_4f(&mut self, color: impl AsRef<Color4f>, subset: Option<&IRect>) -> bool {
        let color = color.as_ref();
        unsafe {
            self.native()
                .erase1(color.native(), subset.native_ptr_or_null())
        }
    }

    fn from_native_c(pixmap: SkPixmap) -> Self {
        Self {
            inner: Handle::from_native_c(pixmap),
            pd: PhantomData,
        }
    }

    #[must_use]
    pub(crate) fn from_native_ref(n: &SkPixmap) -> &Self {
        unsafe { transmute_ref(n) }
    }

    #[must_use]
    pub(crate) fn from_native_ptr(np: *const SkPixmap) -> *const Self {
        // Should be safe as long `Pixmap` is represented with repr(Transparent).
        np as _
    }

    pub(crate) fn native_mut(&mut self) -> &mut SkPixmap {
        self.inner.native_mut()
    }

    pub(crate) fn native(&self) -> &SkPixmap {
        self.inner.native()
    }
}

/// Implement this trait to use a pixel type in [`Handle<Pixmap>::pixels()`].
///
/// # Safety
///
/// This trait is unsafe because external [`Pixel`] implementations may lie about their
/// [`ColorType`] or fail to match the alignment of the pixels stored in [`Handle<Pixmap>`].
pub unsafe trait Pixel: Copy {
    /// `true` if the type matches the color type's format.
    fn matches_color_type(ct: ColorType) -> bool;
}

unsafe impl Pixel for u8 {
    fn matches_color_type(ct: ColorType) -> bool {
        matches!(ct, ColorType::Alpha8 | ColorType::Gray8)
    }
}

unsafe impl Pixel for [u8; 2] {
    fn matches_color_type(ct: ColorType) -> bool {
        matches!(ct, ColorType::R8G8UNorm | ColorType::A16UNorm)
    }
}

unsafe impl Pixel for (u8, u8) {
    fn matches_color_type(ct: ColorType) -> bool {
        matches!(ct, ColorType::R8G8UNorm | ColorType::A16UNorm)
    }
}

unsafe impl Pixel for [u8; 4] {
    fn matches_color_type(ct: ColorType) -> bool {
        matches!(
            ct,
            ColorType::RGBA8888 | ColorType::RGB888x | ColorType::BGRA8888
        )
    }
}

unsafe impl Pixel for (u8, u8, u8, u8) {
    fn matches_color_type(ct: ColorType) -> bool {
        matches!(
            ct,
            ColorType::RGBA8888 | ColorType::RGB888x | ColorType::BGRA8888
        )
    }
}

unsafe impl Pixel for [f32; 4] {
    fn matches_color_type(ct: ColorType) -> bool {
        matches!(ct, ColorType::RGBAF32)
    }
}

unsafe impl Pixel for (f32, f32, f32, f32) {
    fn matches_color_type(ct: ColorType) -> bool {
        matches!(ct, ColorType::RGBAF32)
    }
}

unsafe impl Pixel for u32 {
    fn matches_color_type(ct: ColorType) -> bool {
        ct == ColorType::N32
    }
}

unsafe impl Pixel for Color {
    fn matches_color_type(ct: ColorType) -> bool {
        ct == ColorType::N32
    }
}

unsafe impl Pixel for Color4f {
    fn matches_color_type(ct: ColorType) -> bool {
        ct == ColorType::RGBAF32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixmap_mutably_borrows_pixels() {
        let mut pixels = [0u8; 2 * 2 * 4];
        let info = ImageInfo::new(
            (2, 2),
            ColorType::RGBA8888,
            AlphaType::Premul,
            ColorSpace::new_srgb(),
        );
        let mut pixmap = Pixmap::new(&info, &mut pixels, info.min_row_bytes()).unwrap();
        // this must fail to compile:
        // let _pixel = pixels[0];
        // use `.bytes()`, or `bytes_mut()` instead.
        pixmap.reset();
    }

    #[test]
    fn addr_may_return_null_from_a_default_pixmap() {
        let pixmap = Pixmap::default();
        assert!(pixmap.addr().is_null());
        assert!(pixmap.writable_addr().is_null());
    }
}
