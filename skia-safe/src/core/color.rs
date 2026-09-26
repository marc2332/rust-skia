use crate::prelude::*;
use skia_bindings::{self as sb, SkColor, SkColor4f, SkHSVToColor, SkPMColor, SkRGBToHSV, U8CPU};
use std::ops::{BitAnd, BitOr, Index, IndexMut, Mul};

// TODO: What should we do with SkAlpha?
// It does not seem to be used, but if we want to export it, we'd
// like to define Alpha::TRANSPARENT and Alpha::OPAQUE.
// pub type Alpha = u8;

// Note: SkColor _is_ a u32, and therefore its components are
// endian dependent, so we can't expose it as (transmuted) individual
// argb fields.
/// 32-bit ARGB color value, unpremultiplied. Color components are always in a known order. This is
/// different from `SkPMColor`, which has its bytes in a configuration dependent order, to match
/// the format of BGRA 8888 color type bitmaps. [`Color`] is the type used to specify colors in
/// [`crate::Paint`] and in gradients.
///
/// Color that is premultiplied has the same component values as color that is unpremultiplied if
/// alpha is 255, fully opaque, although it may have the component values in a different order.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
#[repr(transparent)]
pub struct Color(SkColor);

native_transmutable!(SkColor, Color);

impl From<u32> for Color {
    fn from(argb: u32) -> Self {
        Color::new(argb)
    }
}

impl From<RGB> for Color {
    fn from(rgb: RGB) -> Self {
        Color::from_rgb(rgb.r, rgb.g, rgb.b)
    }
}

//
// Bitwise operators.
//

impl BitOr for Color {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Color::from_native_c(self.native() | rhs.native())
    }
}

impl BitAnd for Color {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Color::from_native_c(self.native() & rhs.native())
    }
}

impl BitOr<u32> for Color {
    type Output = Self;

    fn bitor(self, rhs: u32) -> Self::Output {
        self | Color::from_native_c(rhs)
    }
}

impl BitAnd<u32> for Color {
    type Output = Self;

    fn bitand(self, rhs: u32) -> Self::Output {
        self & (Color::from_native_c(rhs))
    }
}

impl Color {
    pub const fn new(argb: u32) -> Self {
        Self(argb)
    }

    // Don't use the u8cpu type in the arguments here, because we trust the Rust compiler to
    // optimize the storage type.
    /// Returns a color value from 8-bit component values. Asserts if `a`, `r`, `g`, or `b` exceed
    /// 255. Since the color is unpremultiplied, `a` may be smaller than the largest of `r`, `g`,
    /// and `b`.
    ///
    /// - `a` amount of alpha, from fully transparent (0) to fully opaque (255)
    /// - `r` amount of red, from no red (0) to full red (255)
    /// - `g` amount of green, from no green (0) to full green (255)
    /// - `b` amount of blue, from no blue (0) to full blue (255)
    pub const fn from_argb(a: u8, r: u8, g: u8, b: u8) -> Color {
        Self(((a as U8CPU) << 24) | ((r as U8CPU) << 16) | ((g as U8CPU) << 8) | (b as U8CPU))
    }

    /// Returns a color value from 8-bit component values, with alpha set fully opaque to 255.
    ///
    /// - `r` amount of red, from no red (0) to full red (255)
    /// - `g` amount of green, from no green (0) to full green (255)
    /// - `b` amount of blue, from no blue (0) to full blue (255)
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Self::from_argb(0xff, r, g, b)
    }

    /// Returns the alpha byte from the color value.
    pub fn a(self) -> u8 {
        (self.into_native() >> 24) as _
    }

    /// Returns the red component of the color, from zero to 255.
    pub fn r(self) -> u8 {
        (self.into_native() >> 16) as _
    }

    /// Returns the green component of the color, from zero to 255.
    pub fn g(self) -> u8 {
        (self.into_native() >> 8) as _
    }

    /// Returns the blue component of the color, from zero to 255.
    pub fn b(self) -> u8 {
        self.into_native() as _
    }

    /// Returns an unpremultiplied color with red, blue, and green set from this color, and alpha
    /// set from `a`. The alpha component of this color is ignored and is replaced by `a` in the
    /// result.
    ///
    /// - `a` alpha: transparent at zero, fully opaque at 255
    #[must_use]
    pub fn with_a(self, a: u8) -> Self {
        Self::from_argb(a, self.r(), self.g(), self.b())
    }

    /// Represents a fully transparent color. May be used to initialize a destination containing a
    /// mask or a non-rectangular image.
    pub const TRANSPARENT: Self = Self(sb::SK_ColorTRANSPARENT);
    /// Represents fully opaque black.
    pub const BLACK: Self = Self(sb::SK_ColorBLACK);
    /// Represents fully opaque dark gray. Note that SVG dark gray is equivalent to `0xFFA9A9A9`.
    pub const DARK_GRAY: Self = Self(sb::SK_ColorDKGRAY);
    /// Represents fully opaque gray. Note that HTML gray is equivalent to `0xFF808080`.
    pub const GRAY: Self = Self(sb::SK_ColorGRAY);
    /// Represents fully opaque light gray. HTML silver is equivalent to `0xFFC0C0C0`. Note that
    /// SVG light gray is equivalent to `0xFFD3D3D3`.
    pub const LIGHT_GRAY: Self = Self(sb::SK_ColorLTGRAY);
    /// Represents fully opaque white.
    pub const WHITE: Self = Self(sb::SK_ColorWHITE);
    /// Represents fully opaque red.
    pub const RED: Self = Self(sb::SK_ColorRED);
    /// Represents fully opaque green. HTML lime is equivalent. Note that HTML green is equivalent
    /// to `0xFF008000`.
    pub const GREEN: Self = Self(sb::SK_ColorGREEN);
    /// Represents fully opaque blue.
    pub const BLUE: Self = Self(sb::SK_ColorBLUE);
    /// Represents fully opaque yellow.
    pub const YELLOW: Self = Self(sb::SK_ColorYELLOW);
    /// Represents fully opaque cyan. HTML aqua is equivalent.
    pub const CYAN: Self = Self(sb::SK_ColorCYAN);
    /// Represents fully opaque magenta. HTML fuchsia is equivalent.
    pub const MAGENTA: Self = Self(sb::SK_ColorMAGENTA);

    pub fn to_rgb(self) -> RGB {
        (self.r(), self.g(), self.b()).into()
    }

    pub fn to_hsv(self) -> HSV {
        self.to_rgb().to_hsv()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<(u8, u8, u8)> for RGB {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}

impl RGB {
    /// Converts RGB to its HSV components.
    ///
    /// `h` contains the HSV hue, a value from zero to less than 360. `s` contains the HSV
    /// saturation, a value from zero to one. `v` contains the HSV value, a value from zero to one.
    pub fn to_hsv(self) -> HSV {
        let mut hsv: [f32; 3] = Default::default();
        unsafe {
            SkRGBToHSV(
                self.r.into(),
                self.g.into(),
                self.b.into(),
                hsv.as_mut_ptr(),
            );
        }
        HSV {
            h: hsv[0],
            s: hsv[1],
            v: hsv[2],
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct HSV {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl From<(f32, f32, f32)> for HSV {
    fn from((h, s, v): (f32, f32, f32)) -> Self {
        Self { h, s, v }
    }
}

impl HSV {
    /// Converts HSV components to an ARGB color. Alpha is passed through unchanged.
    ///
    /// `h` represents the HSV hue, an angle from zero to less than 360. `s` represents the HSV
    /// saturation, and varies from zero to one. `v` represents the HSV value, and varies from zero
    /// to one.
    ///
    /// Out of range HSV values are pinned.
    ///
    /// - `alpha` alpha component of the returned ARGB color
    pub fn to_color(self, alpha: u8) -> Color {
        Color::from_native_c(unsafe {
            SkHSVToColor(alpha.into(), [self.h, self.s, self.v].as_ptr())
        })
    }
}

pub type PMColor = SkPMColor;

/// Returns a `PMColor` value from unpremultiplied 8-bit component values.
///
/// - `a` amount of alpha, from fully transparent (0) to fully opaque (255)
/// - `r` amount of red, from no red (0) to full red (255)
/// - `g` amount of green, from no green (0) to full green (255)
/// - `b` amount of blue, from no blue (0) to full blue (255)
pub fn pre_multiply_argb(a: U8CPU, r: U8CPU, g: U8CPU, b: U8CPU) -> PMColor {
    unsafe { sb::SkPreMultiplyARGB(a, r, g, b) }
}

/// Returns the premultiplied color closest to `c`. Multiplies the `c` RGB components by the `c`
/// alpha, and arranges the bytes to match the format of [`crate::ColorType::N32`].
///
/// - `c` unpremultiplied ARGB color
pub fn pre_multiply_color(c: impl Into<Color>) -> PMColor {
    unsafe { sb::SkPreMultiplyColor(c.into().into_native()) }
}

pub use sb::SkColorChannel as ColorChannel;

#[test]
fn color_channel_naming() {
    let _ = ColorChannel::R;
}

bitflags! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ColorChannelFlag: u32 {
        const RED = sb::SkColorChannelFlag::kRed_SkColorChannelFlag as _;
        const GREEN = sb::SkColorChannelFlag::kGreen_SkColorChannelFlag as _;
        const BLUE = sb::SkColorChannelFlag::kBlue_SkColorChannelFlag as _;
        const ALPHA = sb::SkColorChannelFlag::kAlpha_SkColorChannelFlag as _;
        const GRAY = sb::SkColorChannelFlag::kGray_SkColorChannelFlag as _;
        const GRAY_ALPHA = Self::GRAY.bits() | Self::ALPHA.bits();
        const RG = Self::RED.bits() | Self::GREEN.bits();
        const RGB = Self::RG.bits() | Self::BLUE.bits();
        const RGBA = Self::RGB.bits() | Self::ALPHA.bits();
    }
}

// TODO: SkRGBA4f

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct Color4f {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

native_transmutable!(SkColor4f, Color4f);

impl AsRef<Self> for Color4f {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl Mul<f32> for Color4f {
    type Output = Self;
    fn mul(self, scale: f32) -> Self {
        let r = self.r * scale;
        let g = self.g * scale;
        let b = self.b * scale;
        let a = self.a * scale;
        Self { r, g, b, a }
    }
}

impl Mul for Color4f {
    type Output = Self;
    fn mul(self, scale: Self) -> Self {
        self.mul(&scale)
    }
}

impl Mul<&Self> for Color4f {
    type Output = Self;
    fn mul(self, scale: &Self) -> Self {
        Self {
            r: self.r * scale.r,
            g: self.g * scale.g,
            b: self.b * scale.b,
            a: self.a * scale.a,
        }
    }
}

impl Index<usize> for Color4f {
    type Output = f32;
    fn index(&self, index: usize) -> &f32 {
        &self.as_array()[index]
    }
}

impl IndexMut<usize> for Color4f {
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        &mut self.as_array_mut()[index]
    }
}

impl From<Color> for Color4f {
    fn from(color: Color) -> Self {
        fn c(c: u8) -> f32 {
            (f32::from(c)) * (1.0 / 255.0)
        }
        let r = c(color.r());
        let g = c(color.g());
        let b = c(color.b());
        let a = c(color.a());
        Self { r, g, b, a }
    }
}

// Add all Color::From implementations to Color4f, so that
// function signatures can promote Into<Color> to Into<Color4f>.

impl From<u32> for Color4f {
    fn from(color: u32) -> Self {
        Color::from(color).into()
    }
}

impl From<RGB> for Color4f {
    fn from(rgb: RGB) -> Self {
        Color::from(rgb).into()
    }
}

impl Color4f {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Color4f {
        Self { r, g, b, a }
    }

    // corresponding Skia function: vec()
    /// Returns a pointer to the components of this color, for array access.
    ///
    /// The returned slice is `[r, g, b, a]`.
    pub fn as_array(&self) -> &[f32; 4] {
        unsafe { transmute_ref(self) }
    }

    // corresponding Skia function: vec()
    /// Returns a mutable pointer to the components of this color, for array access.
    ///
    /// The returned slice is `[r, g, b, a]`.
    pub fn as_array_mut(&mut self) -> &mut [f32; 4] {
        unsafe { transmute_ref_mut(self) }
    }

    /// Returns true if this color is an opaque color. Asserts if `a` is out of range and `SK_DEBUG`
    /// is defined.
    #[allow(clippy::float_cmp)]
    pub fn is_opaque(&self) -> bool {
        self.a == 1.0
    }

    /// Returns true if all channels are in `[0, 1]`.
    // TODO: This is the copied implementation, it would probably be better
    //       to call the Skia function.
    pub fn fits_in_bytes(&self) -> bool {
        debug_assert!(self.a >= 0.0 && self.a <= 1.0);
        self.r >= 0.0
            && self.r <= 1.0
            && self.g >= 0.0
            && self.g <= 1.0
            && self.b >= 0.0
            && self.b <= 1.0
    }

    /// Returns the closest [`Color`] to this color. Only allowed if this color is
    /// unpremultiplied.
    pub fn to_color(self) -> Color {
        fn c(f: f32) -> u8 {
            (f.clamp(0.0, 1.0) * 255.0) as u8
        }
        let a = c(self.a);
        let r = c(self.r);
        let g = c(self.g);
        let b = c(self.b);
        Color::from_argb(a, r, g, b)
    }

    // TODO: FromPMColor
    // TODO: premul()
    // TODO: unpremul()

    #[must_use]
    pub fn to_bytes(self) -> u32 {
        unsafe { sb::C_SkColor4f_toBytes_RGBA(self.native()) }
    }

    #[must_use]
    pub fn from_bytes_rgba(color: u32) -> Self {
        Self::from_native_c(unsafe { sb::C_SkColor4f_FromBytes_RGBA(color) })
    }

    #[must_use]
    /// Returns a copy of this color, but with the alpha component set to `1.0`.
    pub fn to_opaque(self) -> Self {
        Self { a: 1.0, ..self }
    }

    #[must_use]
    /// Returns a copy of this color, but with the alpha component pinned to `[0, 1]`.
    pub fn pin_alpha(self) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: 0f32.max(self.a.min(1.0)),
        }
    }
}

pub mod colors {
    //! Named [`crate::Color4f`] constants for common colors.
    use crate::Color4f;

    pub const TRANSPARENT: Color4f = Color4f::new(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Color4f = Color4f::new(0.0, 0.0, 0.0, 1.0);
    pub const DARK_GREY: Color4f = Color4f::new(0.25, 0.25, 0.25, 1.0);
    pub const GREY: Color4f = Color4f::new(0.5, 0.5, 0.5, 1.0);
    pub const LIGHT_GREY: Color4f = Color4f::new(0.75, 0.75, 0.75, 1.0);
    pub const WHITE: Color4f = Color4f::new(1.0, 1.0, 1.0, 1.0);
    pub const RED: Color4f = Color4f::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Color4f = Color4f::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Color4f = Color4f::new(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Color4f = Color4f::new(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Color4f = Color4f::new(0.0, 1.0, 1.0, 1.0);
    pub const MAGENTA: Color4f = Color4f::new(1.0, 0.0, 1.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::{Color, Color4f, colors};

    #[test]
    #[allow(clippy::float_cmp)]
    pub fn color4f_array_access() {
        let mut color = Color4f {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 0.4,
        };
        color[1] = 0.5;
        assert_eq!(0.5, color.g);
    }

    #[test]
    pub fn color_color4f_conversion() {
        let c = Color::from_argb(1, 2, 3, 4);
        let cf = Color4f::from(c);
        let c2 = cf.to_color();
        assert_eq!(c, c2);
    }

    #[test]
    pub fn color4f_value_can_be_passed_as_ref() {
        fn passed_as_ref(_c: impl AsRef<Color4f>) {}
        passed_as_ref(colors::BLACK);
    }
}
