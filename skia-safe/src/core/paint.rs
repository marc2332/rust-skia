//! [`Paint`] controls options applied when drawing. [`Paint`] collects all
//! options outside of the [`crate::Canvas`] clip and [`crate::Canvas`] matrix.
//!
//! Various options apply to strokes and fills, and images.
//!
//! [`Paint`] collects effects and filters that describe single-pass and multiple-pass
//! algorithms that alter the drawing geometry, color, and transparency. For instance,
//! [`Paint`] does not directly implement dashing or blur, but contains the objects that do so.

use crate::Blender;
use crate::{
    BlendMode, Color, Color4f, ColorFilter, ColorSpace, ImageFilter, MaskFilter, PathEffect,
    Shader, prelude::*, scalar,
};
use core::fmt;

use skia_bindings::{self as sb, SkPaint};

/// Set the [`Style`] to fill, stroke, or both fill and stroke geometry.
/// The stroke and fill
/// share all paint attributes; for instance, they are drawn with the same color.
///
/// Use [`Style::StrokeAndFill`] to avoid hitting the same pixels twice with a stroke draw and
/// a fill draw.
pub use sb::SkPaint_Style as Style;
variant_name!(Style::Fill);

/// [`Cap`] draws at the beginning and end of an open path contour.
pub use sb::SkPaint_Cap as Cap;
variant_name!(Cap::Butt);

/// [`Join`] specifies how corners are drawn when a shape is stroked. [`Join`]
/// affects the four corners of a stroked rectangle, and the connected segments in a
/// stroked path.
///
/// Choose miter join to draw sharp corners. Choose round join to draw a circle with a
/// radius equal to the stroke width on top of the corner. Choose bevel join to minimally
/// connect the thick strokes.
///
/// The fill path constructed to describe the stroked path respects the join setting but may
/// not contain the actual join. For instance, a fill path constructed with round joins does
/// not necessarily include circles at each connected segment.
pub use sb::SkPaint_Join as Join;
variant_name!(Join::Miter);

/// [`Paint`] controls options applied when drawing.
pub type Paint = Handle<SkPaint>;
unsafe_send_sync!(Paint);

impl NativeDrop for SkPaint {
    fn drop(&mut self) {
        unsafe { sb::C_SkPaint_destruct(self) }
    }
}

impl NativeClone for SkPaint {
    /// Makes a shallow copy of [`Paint`]. [`PathEffect`], [`Shader`], [`MaskFilter`],
    /// [`ColorFilter`], and [`ImageFilter`] are shared between the original paint and the copy.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_copy_const_SkPaint>
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_copy_operator>
    fn clone(&self) -> Self {
        unsafe { SkPaint::new2(self) }
    }
}

impl NativePartialEq for SkPaint {
    fn eq(&self, rhs: &Self) -> bool {
        unsafe { sb::C_SkPaint_Equals(self, rhs) }
    }
}

impl Default for Handle<SkPaint> {
    /// Constructs a [`Paint`] with default values.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_empty_constructor>
    fn default() -> Self {
        Paint::from_native_c(unsafe { SkPaint::new() })
    }
}

impl fmt::Debug for Paint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Paint")
            .field("is_anti_alias", &self.is_anti_alias())
            .field("is_dither", &self.is_dither())
            .field("style", &self.style())
            .field("color", &self.color4f())
            .field("stroke_width", &self.stroke_width())
            .field("stroke_miter", &self.stroke_miter())
            .field("stroke_cap", &self.stroke_cap())
            .field("stroke_join", &self.stroke_join())
            .field("color_filter", &self.color_filter())
            .field("blend_mode", &self.as_blend_mode())
            .field("path_effect", &self.path_effect())
            .field("mask_filter", &self.mask_filter())
            .field("image_filter", &self.image_filter())
            .finish()
    }
}

impl Paint {
    /// Constructs a paint with default values and the given color.
    ///
    /// Sets alpha and RGB used when stroking and filling. The color is four floating
    /// point values, unpremultiplied. The color values are interpreted as being in
    /// the `color_space`. If `color_space` is `None`, then color is assumed to be in the
    /// sRGB color space.
    ///
    /// - `color` unpremultiplied RGBA
    /// - `color_space` [`ColorSpace`] describing the encoding of color
    pub fn new<'a>(
        color: impl AsRef<Color4f>,
        color_space: impl Into<Option<&'a ColorSpace>>,
    ) -> Paint {
        let color_space = color_space.into();
        Paint::from_native_c(unsafe {
            SkPaint::new1(
                color.as_ref().native(),
                color_space.native_ptr_or_null_mut_force(),
            )
        })
    }

    /// Sets all [`Paint`] contents to their initial values. This is equivalent to replacing
    /// [`Paint`] with the result of `Paint::default()`.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_reset>
    pub fn reset(&mut self) -> &mut Self {
        unsafe { self.native_mut().reset() }
        self
    }

    /// Returns true if pixels on the active edges of [`crate::Path`] may be drawn with partial transparency.
    ///
    /// Returns antialiasing state.
    pub fn is_anti_alias(&self) -> bool {
        unsafe { self.native().__bindgen_anon_1.fBitfields.fAntiAlias() != 0 }
    }

    /// Requests, but does not require, that edge pixels draw opaque or with
    /// partial transparency.
    ///
    /// - `anti_alias` setting for antialiasing
    pub fn set_anti_alias(&mut self, anti_alias: bool) -> &mut Self {
        unsafe {
            self.native_mut()
                .__bindgen_anon_1
                .fBitfields
                .set_fAntiAlias(anti_alias as _);
        }
        self
    }

    /// Returns true if color error may be distributed to smooth color transition.
    ///
    /// Returns dithering state.
    pub fn is_dither(&self) -> bool {
        unsafe { self.native().__bindgen_anon_1.fBitfields.fDither() != 0 }
    }

    /// Requests, but does not require, to distribute color error.
    ///
    /// - `dither` setting for dithering
    pub fn set_dither(&mut self, dither: bool) -> &mut Self {
        unsafe {
            self.native_mut()
                .__bindgen_anon_1
                .fBitfields
                .set_fDither(dither as _);
        }
        self
    }

    /// Returns whether the geometry is filled, stroked, or filled and stroked.
    pub fn style(&self) -> Style {
        unsafe { sb::C_SkPaint_getStyle(self.native()) }
    }

    /// Sets whether the geometry is filled, stroked, or filled and stroked.
    /// Has no effect if `style` is not a legal [`Style`] value.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_setStyle>
    /// Example (C++): <https://fiddle.skia.org/c/@Stroke_Width>
    pub fn set_style(&mut self, style: Style) -> &mut Self {
        unsafe { self.native_mut().setStyle(style) }
        self
    }

    /// Set paint's style to [`Style::Stroke`] if `true`, or [`Style::Fill`] if `false`.
    pub fn set_stroke(&mut self, stroke: bool) -> &mut Self {
        unsafe { self.native_mut().setStroke(stroke) }
        self
    }

    /// Retrieves alpha and RGB, unpremultiplied, packed into 32 bits.
    ///
    /// Returns unpremultiplied ARGB.
    pub fn color(&self) -> Color {
        self.color4f().to_color()
    }

    /// Retrieves alpha and RGB, unpremultiplied, as four floating point values. RGB are
    /// extended sRGB values (sRGB gamut, and encoded with the sRGB transfer function).
    ///
    /// Returns unpremultiplied RGBA.
    pub fn color4f(&self) -> Color4f {
        Color4f::from_native_c(self.native().fColor4f)
    }

    /// Sets alpha and RGB used when stroking and filling. The color is a 32-bit value,
    /// unpremultiplied, packing 8-bit components for alpha, red, blue, and green.
    ///
    /// - `color` unpremultiplied ARGB
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_setColor>
    pub fn set_color(&mut self, color: impl Into<Color>) -> &mut Self {
        let color = color.into();
        unsafe { self.native_mut().setColor(color.into_native()) }
        self
    }

    /// Sets alpha and RGB used when stroking and filling. The color is four floating
    /// point values, unpremultiplied. The color values are interpreted as being in
    /// the `color_space`. If `color_space` is `None`, then color is assumed to be in the
    /// sRGB color space.
    ///
    /// - `color` unpremultiplied RGBA
    /// - `color_space` [`ColorSpace`] describing the encoding of color
    pub fn set_color4f<'a>(
        &mut self,
        color: impl AsRef<Color4f>,
        color_space: impl Into<Option<&'a ColorSpace>>,
    ) -> &mut Self {
        let color_space: Option<&'a ColorSpace> = color_space.into();
        unsafe {
            self.native_mut().setColor1(
                color.as_ref().native(),
                color_space.native_ptr_or_null_mut_force(),
            )
        }
        self
    }

    /// Retrieves alpha from the color used when stroking and filling.
    ///
    /// Returns alpha ranging from zero, fully transparent, to one, fully opaque.
    pub fn alpha_f(&self) -> f32 {
        self.color4f().a
    }

    /// Helper that scales the alpha by 255.
    ///
    /// Returns alpha, from fully transparent (0) to fully opaque (255).
    pub fn alpha(&self) -> u8 {
        unsafe { sb::C_SkPaint_getAlpha(self.native()) }
    }

    /// Replaces alpha, leaving RGB
    /// unchanged. An out of range value triggers an assert in the debug
    /// build. `alpha` is a value from 0.0 to 1.0.
    /// `alpha` set to zero makes color fully transparent; `alpha` set to 1.0 makes color
    /// fully opaque.
    ///
    /// - `alpha` alpha component of color
    pub fn set_alpha_f(&mut self, alpha: f32) -> &mut Self {
        unsafe { self.native_mut().setAlphaf(alpha) }
        self
    }

    /// Helper that accepts an int between 0 and 255, and divides it by 255.0
    pub fn set_alpha(&mut self, alpha: u8) -> &mut Self {
        self.set_alpha_f(f32::from(alpha) * (1.0 / 255.0))
    }

    /// Sets color used when drawing solid fills. The color components range from 0 to 255.
    /// The color is unpremultiplied; alpha sets the transparency independent of RGB.
    ///
    /// - `a` amount of alpha, from fully transparent (0) to fully opaque (255)
    /// - `r` amount of red, from no red (0) to full red (255)
    /// - `g` amount of green, from no green (0) to full green (255)
    /// - `b` amount of blue, from no blue (0) to full blue (255)
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_setARGB>
    pub fn set_argb(&mut self, a: u8, r: u8, g: u8, b: u8) -> &mut Self {
        unsafe {
            self.native_mut()
                .setARGB(a.into(), r.into(), g.into(), b.into())
        }
        self
    }

    /// Returns the thickness of the pen used by [`Paint`] to
    /// outline the shape.
    ///
    /// Returns zero for hairline, greater than zero for pen thickness.
    pub fn stroke_width(&self) -> scalar {
        self.native().fWidth
    }

    /// Sets the thickness of the pen used by the paint to outline the shape.
    /// A stroke-width of zero is treated as "hairline" width. Hairlines are always exactly one
    /// pixel wide in device space (their thickness does not change as the canvas is scaled).
    /// Negative stroke-widths are invalid; setting a negative width will have no effect.
    ///
    /// - `width` zero thickness for hairline; greater than zero for pen thickness
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Miter_Limit>
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_setStrokeWidth>
    pub fn set_stroke_width(&mut self, width: scalar) -> &mut Self {
        unsafe { self.native_mut().setStrokeWidth(width) }
        self
    }

    /// Returns the limit at which a sharp corner is drawn beveled.
    ///
    /// Returns zero and greater miter limit.
    pub fn stroke_miter(&self) -> scalar {
        self.native().fMiterLimit
    }

    /// When stroking a small joinAngle with miter, the miterLength may be very long.
    /// When miterLength > maxMiterLength (or joinAngle < minJoinAngle) the join will become bevel.
    /// miterLimit = maxMiterLength / strokeWidth or miterLimit = 1 / sin(minJoinAngle / 2).
    ///
    /// This call has no effect if the `miter_limit` passed is less than zero.
    /// Values less than one will be treated as bevel.
    ///
    /// - `miter_limit` zero and greater miter limit
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_setStrokeMiter>
    pub fn set_stroke_miter(&mut self, miter_limit: scalar) -> &mut Self {
        unsafe { self.native_mut().setStrokeMiter(miter_limit) }
        self
    }

    /// Returns the geometry drawn at the beginning and end of strokes.
    pub fn stroke_cap(&self) -> Cap {
        unsafe { sb::C_SkPaint_getStrokeCap(self.native()) }
    }

    /// Sets the geometry drawn at the beginning and end of strokes.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_setStrokeCap_a>
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_setStrokeCap_b>
    pub fn set_stroke_cap(&mut self, cap: Cap) -> &mut Self {
        unsafe { self.native_mut().setStrokeCap(cap) }
        self
    }

    /// Returns the geometry drawn at the corners of strokes.
    pub fn stroke_join(&self) -> Join {
        unsafe { sb::C_SkPaint_getStrokeJoin(self.native()) }
    }

    /// Sets the geometry drawn at the corners of strokes.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_setStrokeJoin>
    pub fn set_stroke_join(&mut self, join: Join) -> &mut Self {
        unsafe { self.native_mut().setStrokeJoin(join) }
        self
    }

    /// Returns optional colors used when filling a path, such as a gradient.
    ///
    /// Does not alter [`Shader`] SkRefCnt.
    ///
    /// Returns [`Shader`] if previously set, `None` otherwise.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_refShader>
    pub fn shader(&self) -> Option<Shader> {
        Shader::from_unshared_ptr(self.native().fShader.fPtr)
    }

    /// Sets optional colors used when filling a path, such as a gradient.
    ///
    /// Sets [`Shader`] to `shader`, decreasing SkRefCnt of the previous [`Shader`].
    /// Increments `shader` SkRefCnt by one.
    ///
    /// - `shader` how geometry is filled with color; if `None`, color is used instead
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Color_Filter_Methods>
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_setShader>
    pub fn set_shader(&mut self, shader: impl Into<Option<Shader>>) -> &mut Self {
        unsafe { sb::C_SkPaint_setShader(self.native_mut(), shader.into().into_ptr_or_null()) }
        self
    }

    /// Returns [`ColorFilter`] if set, or `None`.
    /// Does not alter [`ColorFilter`] SkRefCnt.
    ///
    /// Returns [`ColorFilter`] if previously set, `None` otherwise.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_refColorFilter>
    pub fn color_filter(&self) -> Option<ColorFilter> {
        ColorFilter::from_unshared_ptr(self.native().fColorFilter.fPtr)
    }

    /// Sets [`ColorFilter`] to `color_filter`, decreasing SkRefCnt of the previous
    /// [`ColorFilter`]. Pass `None` to clear [`ColorFilter`].
    ///
    /// Increments `color_filter` SkRefCnt by one.
    ///
    /// - `color_filter` [`ColorFilter`] to apply to subsequent draw
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Blend_Mode_Methods>
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_setColorFilter>
    pub fn set_color_filter(&mut self, color_filter: impl Into<Option<ColorFilter>>) -> &mut Self {
        unsafe {
            sb::C_SkPaint_setColorFilter(self.native_mut(), color_filter.into().into_ptr_or_null())
        }
        self
    }

    /// If the current blender can be represented as a [`BlendMode`] enum, this returns that
    /// enum in the option's value. If it cannot, then the returned option does not
    /// contain a value.
    pub fn as_blend_mode(&self) -> Option<BlendMode> {
        let mut bm = BlendMode::default();
        unsafe { sb::C_SkPaint_asBlendMode(self.native(), &mut bm) }.then_some(bm)
    }

    /// Queries the blender, and if it can be represented as a [`BlendMode`], return that mode,
    /// else return the `default_mode` provided.
    pub fn blend_mode_or(&self, default_mode: BlendMode) -> BlendMode {
        unsafe { self.native().getBlendMode_or(default_mode) }
    }

    #[deprecated(
        since = "0.42.0",
        note = "Use as_blend_mode() or blend_mode_or() instead."
    )]
    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode_or(BlendMode::SrcOver)
    }

    /// Returns true iff the current blender claims to be equivalent to [`BlendMode::SrcOver`].
    ///
    /// Also returns true of the current blender is `None`.
    pub fn is_src_over(&self) -> bool {
        unsafe { self.native().isSrcOver() }
    }

    /// Helper method for calling [`Paint::set_blender()`].
    ///
    /// This sets a blender that implements the specified blendmode enum.
    pub fn set_blend_mode(&mut self, mode: BlendMode) -> &mut Self {
        unsafe { self.native_mut().setBlendMode(mode) }
        self
    }

    /// Returns the user-supplied blend function, if one has been set.
    /// Does not alter [`Blender`]'s SkRefCnt.
    ///
    /// A `None` blender signifies the default SrcOver behavior.
    ///
    /// Returns the [`Blender`] assigned to this paint, otherwise `None`.
    pub fn blender(&self) -> Option<Blender> {
        Blender::from_unshared_ptr(self.native().fBlender.fPtr)
    }

    /// Sets the current blender, increasing its refcnt, and if a blender is already
    /// present, decreasing that object's refcnt.
    ///
    /// A `None` blender signifies the default SrcOver behavior.
    ///
    /// For convenience, you can call [`Paint::set_blend_mode()`] if the blend effect can be
    /// expressed as one of those values.
    ///
    /// - `blender` the [`Blender`] assigned to this paint, or `None` for the default
    pub fn set_blender(&mut self, blender: impl Into<Option<Blender>>) -> &mut Self {
        unsafe { sb::C_SkPaint_setBlender(self.native_mut(), blender.into().into_ptr_or_null()) }
        self
    }

    /// Returns [`PathEffect`] if set, or `None`.
    /// Does not alter [`PathEffect`] SkRefCnt.
    ///
    /// Returns [`PathEffect`] if previously set, `None` otherwise.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_refPathEffect>
    pub fn path_effect(&self) -> Option<PathEffect> {
        PathEffect::from_unshared_ptr(self.native().fPathEffect.fPtr)
    }

    /// Sets [`PathEffect`] to `path_effect`, decreasing SkRefCnt of the previous
    /// [`PathEffect`]. Pass `None` to leave the path geometry unaltered.
    ///
    /// Increments `path_effect` SkRefCnt by one.
    ///
    /// - `path_effect` replace [`crate::Path`] with a modification when drawn
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Mask_Filter_Methods>
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_setPathEffect>
    pub fn set_path_effect(&mut self, path_effect: impl Into<Option<PathEffect>>) -> &mut Self {
        unsafe {
            sb::C_SkPaint_setPathEffect(self.native_mut(), path_effect.into().into_ptr_or_null())
        }
        self
    }

    /// Returns [`MaskFilter`] if set, or `None`.
    /// Does not alter [`MaskFilter`] SkRefCnt.
    ///
    /// Returns [`MaskFilter`] if previously set, `None` otherwise.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_refMaskFilter>
    pub fn mask_filter(&self) -> Option<MaskFilter> {
        MaskFilter::from_unshared_ptr(self.native().fMaskFilter.fPtr)
    }

    /// Sets [`MaskFilter`] to `mask_filter`, decreasing SkRefCnt of the previous
    /// [`MaskFilter`]. Pass `None` to clear [`MaskFilter`] and leave [`MaskFilter`] effect on
    /// mask alpha unaltered.
    ///
    /// Increments `mask_filter` SkRefCnt by one.
    ///
    /// - `mask_filter` modifies clipping mask generated from drawn geometry
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_setMaskFilter>
    /// Example (C++): <https://fiddle.skia.org/c/@Typeface_Methods>
    pub fn set_mask_filter(&mut self, mask_filter: impl Into<Option<MaskFilter>>) -> &mut Self {
        unsafe {
            sb::C_SkPaint_setMaskFilter(self.native_mut(), mask_filter.into().into_ptr_or_null())
        }
        self
    }

    /// Returns [`ImageFilter`] if set, or `None`.
    /// Does not alter [`ImageFilter`] SkRefCnt.
    ///
    /// Returns [`ImageFilter`] if previously set, `None` otherwise.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_refImageFilter>
    pub fn image_filter(&self) -> Option<ImageFilter> {
        ImageFilter::from_unshared_ptr(self.native().fImageFilter.fPtr)
    }

    /// Sets [`ImageFilter`] to `image_filter`, decreasing SkRefCnt of the previous
    /// [`ImageFilter`]. Pass `None` to clear [`ImageFilter`], and remove [`ImageFilter`] effect
    /// on drawing.
    ///
    /// Increments `image_filter` SkRefCnt by one.
    ///
    /// - `image_filter` how [`crate::Image`] is sampled when transformed
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_setImageFilter>
    pub fn set_image_filter(&mut self, image_filter: impl Into<Option<ImageFilter>>) -> &mut Self {
        unsafe {
            sb::C_SkPaint_setImageFilter(self.native_mut(), image_filter.into().into_ptr_or_null())
        }
        self
    }

    /// Returns true if [`Paint`] prevents all drawing;
    /// otherwise, the [`Paint`] may or may not allow drawing.
    ///
    /// Returns true if, for example, [`BlendMode`] combined with alpha computes a
    /// new alpha of zero.
    ///
    /// Returns `true` if [`Paint`] prevents all drawing.
    ///
    /// Example (C++): <https://fiddle.skia.org/c/@Paint_nothingToDraw>
    pub fn nothing_to_draw(&self) -> bool {
        unsafe { self.native().nothingToDraw() }
    }
}

#[test]
fn default_creation() {
    let paint = Paint::default();
    drop(paint)
}

#[test]
fn method_chaining_compiles() {
    let mut paint = Paint::default();
    let _paint = paint.reset().reset();
}

#[test]
fn union_flags() {
    let mut paint = Paint::default();
    assert!(!paint.is_anti_alias());
    assert!(!paint.is_dither());
    assert_eq!(paint.style(), Style::Fill);

    {
        paint.set_anti_alias(true);

        assert!(paint.is_anti_alias());
        assert!(!paint.is_dither());
        assert_eq!(paint.style(), Style::Fill);

        paint.set_anti_alias(false);
    }

    {
        paint.set_style(Style::StrokeAndFill);

        assert!(!paint.is_anti_alias());
        assert!(!paint.is_dither());
        assert_eq!(paint.style(), Style::StrokeAndFill);

        paint.set_style(Style::Fill);
    }
}

#[test]
fn set_color4f_color_space() {
    let mut paint = Paint::default();
    let color = Color4f::from(Color::DARK_GRAY);
    let color_space = ColorSpace::new_srgb();
    paint.set_color4f(color, None);
    paint.set_color4f(color, &color_space);
    let color2 = Color4f::from(Color::DARK_GRAY);
    paint.set_color4f(color2, Some(&color_space));
}
