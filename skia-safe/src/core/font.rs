//! [`Font`] controls options applied when drawing and measuring text.

use std::{fmt, ptr};

use skia_bindings::{self as sb, SkFont, SkFont_PrivFlags};

use crate::{
    EncodedText, FontHinting, FontMetrics, GlyphId, Paint, Path, Point, Rect, StrikeRef, Typeface,
    Unichar, interop::VecSink, prelude::*, scalar,
};

/// Whether edge pixels draw opaque or with partial transparency.
pub type Edging = skia_bindings::SkFont_Edging;
variant_name!(Edging::Alias);

/// [`Font`] controls options applied when drawing and measuring text.
pub type Font = Handle<SkFont>;
unsafe_send_sync!(Font);

impl NativeDrop for SkFont {
    fn drop(&mut self) {
        unsafe { sb::C_SkFont_destruct(self) }
    }
}

impl NativeClone for SkFont {
    fn clone(&self) -> Self {
        construct(|f| unsafe { sb::C_SkFont_CopyConstruct(f, self) })
    }
}

impl NativePartialEq for SkFont {
    fn eq(&self, rhs: &Self) -> bool {
        unsafe { sb::C_SkFont_Equals(self, rhs) }
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::from_native_c(unsafe { SkFont::new() })
    }
}

impl fmt::Debug for Font {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Font")
            .field("is_force_auto_hinting", &self.is_force_auto_hinting())
            .field("is_embedded_bitmaps", &self.is_embedded_bitmaps())
            .field("is_subpixel", &self.is_subpixel())
            .field("is_linear_metrics", &self.is_linear_metrics())
            .field("is_embolden", &self.is_embolden())
            .field("is_baseline_snap", &self.is_baseline_snap())
            .field("edging", &self.edging())
            .field("hinting", &self.hinting())
            .field("typeface", &self.typeface())
            .field("size", &self.size())
            .field("scale_x", &self.scale_x())
            .field("skew_x", &self.skew_x())
            .field("metrics", &self.metrics())
            .field("spacing", &self.spacing())
            .finish()
    }
}

impl Font {
    /// Constructs a font with default values with [`Typeface`] and size.
    ///
    /// - `typeface` font and style used to draw and measure text
    /// - `size` EM size in local coordinate units
    pub fn new(typeface: impl Into<Typeface>, size: impl Into<Option<scalar>>) -> Self {
        Self::from_typeface(typeface, size)
    }

    /// Constructs a font with default values with [`Typeface`] and size.
    ///
    /// - `typeface` font and style used to draw and measure text
    /// - `size` EM size in local coordinate units
    pub fn from_typeface(typeface: impl Into<Typeface>, size: impl Into<Option<scalar>>) -> Self {
        match size.into() {
            None => Self::construct(|font| unsafe {
                sb::C_SkFont_ConstructFromTypeface(font, typeface.into().into_ptr())
            }),
            Some(size) => Self::construct(|font| unsafe {
                sb::C_SkFont_ConstructFromTypefaceWithSize(font, typeface.into().into_ptr(), size)
            }),
        }
    }

    /// Constructs a font with default values with [`Typeface`] and size in points,
    /// horizontal scale, and horizontal skew. Horizontal scale emulates condensed
    /// and expanded fonts. Horizontal skew emulates oblique fonts.
    ///
    /// - `typeface` font and style used to draw and measure text
    /// - `size` EM size in local coordinate units
    /// - `scale` text horizontal scale
    /// - `skew` additional shear on x-axis relative to y-axis
    pub fn from_typeface_with_params(
        typeface: impl Into<Typeface>,
        size: scalar,
        scale: scalar,
        skew: scalar,
    ) -> Self {
        Self::construct(|font| unsafe {
            sb::C_SkFont_ConstructFromTypefaceWithSizeScaleAndSkew(
                font,
                typeface.into().into_ptr(),
                size,
                scale,
                skew,
            )
        })
    }

    /// If `true`, instructs the font manager to always hint glyphs.
    /// Returned value is only meaningful if platform uses FreeType as the font manager.
    ///
    /// Returns `true` if all glyphs are hinted.
    pub fn is_force_auto_hinting(&self) -> bool {
        self.has_flag(sb::SkFont_PrivFlags_kForceAutoHinting_PrivFlag)
    }

    /// Returns true if font engine may return glyphs from font bitmaps instead of from outlines.
    ///
    /// Returns `true` if glyphs may be font bitmaps.
    pub fn is_embedded_bitmaps(&self) -> bool {
        self.has_flag(sb::SkFont_PrivFlags_kEmbeddedBitmaps_PrivFlag)
    }

    /// Returns true if glyphs may be drawn at sub-pixel offsets.
    ///
    /// Returns `true` if glyphs may be drawn at sub-pixel offsets.
    pub fn is_subpixel(&self) -> bool {
        self.has_flag(sb::SkFont_PrivFlags_kSubpixel_PrivFlag)
    }

    /// Returns true if font and glyph metrics are requested to be linearly scalable.
    ///
    /// Returns `true` if font and glyph metrics are requested to be linearly scalable.
    pub fn is_linear_metrics(&self) -> bool {
        self.has_flag(sb::SkFont_PrivFlags_kLinearMetrics_PrivFlag)
    }

    /// Returns true if bold is approximated by increasing the stroke width when creating glyph
    /// bitmaps from outlines.
    ///
    /// Returns `true` if bold is approximated through stroke width.
    pub fn is_embolden(&self) -> bool {
        self.has_flag(sb::SkFont_PrivFlags_kEmbolden_PrivFlag)
    }

    /// Returns true if baselines will be snapped to pixel positions when the current transformation
    /// matrix is axis aligned.
    ///
    /// Returns `true` if baselines may be snapped to pixels.
    pub fn is_baseline_snap(&self) -> bool {
        self.has_flag(sb::SkFont_PrivFlags_kBaselineSnap_PrivFlag)
    }

    fn has_flag(&self, flag: SkFont_PrivFlags) -> bool {
        (SkFont_PrivFlags::from(self.native().fFlags) & flag) != 0
    }

    /// Sets whether to always hint glyphs.
    /// If `force_auto_hinting` is set, instructs the font manager to always hint glyphs.
    ///
    /// Only affects platforms that use FreeType as the font manager.
    ///
    /// - `force_auto_hinting` setting to always hint glyphs
    pub fn set_force_auto_hinting(&mut self, force_auto_hinting: bool) -> &mut Self {
        unsafe { self.native_mut().setForceAutoHinting(force_auto_hinting) }
        self
    }

    /// Requests, but does not require, to use bitmaps in fonts instead of outlines.
    ///
    /// - `embedded_bitmaps` setting to use bitmaps in fonts
    pub fn set_embedded_bitmaps(&mut self, embedded_bitmaps: bool) -> &mut Self {
        unsafe { self.native_mut().setEmbeddedBitmaps(embedded_bitmaps) }
        self
    }

    /// Requests, but does not require, that glyphs respect sub-pixel positioning.
    ///
    /// - `subpixel` setting for sub-pixel positioning
    pub fn set_subpixel(&mut self, subpixel: bool) -> &mut Self {
        unsafe { self.native_mut().setSubpixel(subpixel) }
        self
    }

    /// Requests, but does not require, linearly scalable font and glyph metrics.
    ///
    /// For outline fonts `true` means font and glyph metrics should ignore hinting and rounding.
    /// Note that some bitmap formats may not be able to scale linearly and will ignore this flag.
    ///
    /// - `linear_metrics` setting for linearly scalable font and glyph metrics.
    pub fn set_linear_metrics(&mut self, linear_metrics: bool) -> &mut Self {
        unsafe { self.native_mut().setLinearMetrics(linear_metrics) }
        self
    }

    /// Increases stroke width when creating glyph bitmaps to approximate a bold typeface.
    ///
    /// - `embolden` setting for bold approximation
    pub fn set_embolden(&mut self, embolden: bool) -> &mut Self {
        unsafe { self.native_mut().setEmbolden(embolden) }
        self
    }

    /// Requests that baselines be snapped to pixels when the current transformation matrix is axis
    /// aligned.
    ///
    /// - `baseline_snap` setting for baseline snapping to pixels
    pub fn set_baseline_snap(&mut self, baseline_snap: bool) -> &mut Self {
        unsafe { self.native_mut().setBaselineSnap(baseline_snap) }
        self
    }

    /// Whether edge pixels draw opaque or with partial transparency.
    pub fn edging(&self) -> Edging {
        unsafe { sb::C_SkFont_getEdging(self.native()) }
    }

    /// Requests, but does not require, that edge pixels draw opaque or with
    /// partial transparency.
    pub fn set_edging(&mut self, edging: Edging) -> &mut Self {
        unsafe { self.native_mut().setEdging(edging) }
        self
    }

    /// Sets level of glyph outline adjustment.
    /// Does not check for valid values of `hinting`.
    pub fn set_hinting(&mut self, hinting: FontHinting) -> &mut Self {
        unsafe { self.native_mut().setHinting(hinting) }
        self
    }

    /// Returns level of glyph outline adjustment.
    pub fn hinting(&self) -> FontHinting {
        unsafe { sb::C_SkFont_getHinting(self.native()) }
    }

    /// Returns a font with the same attributes of this font, but with the specified size.
    /// Returns `None` if `size` is less than zero, infinite, or NaN.
    ///
    /// - `size` EM size in local coordinate units
    #[must_use]
    pub fn with_size(&self, size: scalar) -> Option<Self> {
        if size >= 0.0 && !size.is_infinite() && !size.is_nan() {
            let mut font = unsafe { SkFont::new() };
            unsafe { sb::C_SkFont_makeWithSize(self.native(), size, &mut font) }
            Some(Self::from_native_c(font))
        } else {
            None
        }
    }

    /// Does not alter [`Typeface`] SkRefCnt.
    ///
    /// Returns non-null [`Typeface`].
    pub fn typeface(&self) -> Typeface {
        Typeface::from_unshared_ptr(self.native().fTypeface.fPtr)
            .expect("typeface is expected to be non-null")
    }

    /// Return EM size in local coordinate units.
    /// See <https://skia.org/docs/user/coordinates/#local-coordinates> .
    ///
    /// Returns EM size in local coordinate units.
    pub fn size(&self) -> scalar {
        self.native().fSize
    }

    /// Returns text scale on x-axis.
    /// Default value is 1.
    ///
    /// Returns text horizontal scale.
    pub fn scale_x(&self) -> scalar {
        self.native().fScaleX
    }

    /// Returns text skew on x-axis.
    /// Default value is zero.
    ///
    /// Returns additional shear on x-axis relative to y-axis.
    pub fn skew_x(&self) -> scalar {
        self.native().fSkewX
    }

    /// Sets [`Typeface`] to `tf`, decreasing SkRefCnt of the previous [`Typeface`].
    /// Pass `None` to clear [`Typeface`] and use an empty typeface (which draws nothing).
    /// Increments `tf` SkRefCnt by one.
    ///
    /// - `tf` font and style used to draw text
    pub fn set_typeface(&mut self, tf: impl Into<Typeface>) -> &mut Self {
        unsafe { sb::C_SkFont_setTypeface(self.native_mut(), tf.into().into_ptr()) }
        self
    }

    /// Sets the EM size in local coordinate units.
    /// See <https://skia.org/docs/user/coordinates/#local-coordinates> .
    /// Has no effect if `size` is not greater than or equal to zero.
    ///
    /// - `size` EM size in local coordinate units
    pub fn set_size(&mut self, size: scalar) -> &mut Self {
        unsafe { self.native_mut().setSize(size) }
        self
    }

    /// Sets text scale on x-axis.
    /// Default value is 1.
    ///
    /// - `scale_x` text horizontal scale
    pub fn set_scale_x(&mut self, scale_x: scalar) -> &mut Self {
        unsafe { self.native_mut().setScaleX(scale_x) }
        self
    }

    /// Sets text skew on x-axis.
    /// Default value is zero.
    ///
    /// - `skew_x` additional shear on x-axis relative to y-axis
    pub fn set_skew_x(&mut self, skew_x: scalar) -> &mut Self {
        unsafe { self.native_mut().setSkewX(skew_x) }
        self
    }

    /// Converts `str` into glyph indices.
    /// Returns the number of glyph indices represented by `str`.
    ///
    /// See [`Self::text_to_glyphs()`] for details.
    pub fn str_to_glyphs(&self, str: impl AsRef<str>, glyphs: &mut [GlyphId]) -> usize {
        self.text_to_glyphs(str.as_ref(), glyphs)
    }

    /// Converts text into glyph indices.
    /// Returns the number of glyph indices represented by `text`.
    /// [`crate::TextEncoding`] specifies how text represents characters or glyphs.
    /// `glyphs` may be empty, to compute the glyph count.
    ///
    /// Does not check text for valid character codes or valid glyph indices.
    ///
    /// If the text length is zero, returns zero.
    /// If the text includes a partial character, the partial character is ignored.
    ///
    /// If the encoding is UTF-8 and `text` contains an invalid UTF-8 sequence,
    /// zero is returned.
    ///
    /// When the encoding is UTF-8, UTF-16, or UTF-32, then each Unicode codepoint is mapped to a
    /// single glyph. This function uses the default character-to-glyph
    /// mapping from the [`Typeface`] and maps characters not found in the
    /// [`Typeface`] to zero.
    ///
    /// If `glyphs` is not sufficient to store all the glyphs, no glyphs are copied.
    /// The total glyph count is returned for subsequent buffer reallocation.
    ///
    /// - `text` character storage encoded with [`crate::TextEncoding`]
    /// - `glyphs` storage for glyph indices; may be empty
    pub fn text_to_glyphs(&self, text: impl EncodedText, glyphs: &mut [GlyphId]) -> usize {
        let (ptr, size, encoding) = text.as_raw();
        unsafe {
            sb::C_SkFont_textToGlyphs(
                self.native(),
                ptr,
                size,
                encoding.into_native(),
                glyphs.as_mut_ptr(),
                glyphs.len(),
            )
        }
    }

    /// Returns number of glyphs represented by `str`.
    ///
    /// See [`Self::count_text()`] for details.
    pub fn count_str(&self, str: impl AsRef<str>) -> usize {
        self.count_text(str.as_ref())
    }

    /// Returns number of glyphs represented by `text`.
    ///
    /// If the encoding is UTF-8, UTF-16, or UTF-32, then each Unicode codepoint is mapped to a
    /// single glyph.
    ///
    /// - `text` character storage encoded with [`crate::TextEncoding`]
    pub fn count_text(&self, text: impl EncodedText) -> usize {
        let (ptr, size, encoding) = text.as_raw();
        unsafe {
            sb::C_SkFont_textToGlyphs(
                self.native(),
                ptr,
                size,
                encoding.into_native(),
                ptr::null_mut(),
                0usize,
            )
        }
    }

    /// Convenience variant of [`Self::str_to_glyphs()`] that returns the glyphs as a `Vec`.
    pub fn str_to_glyphs_vec(&self, str: impl AsRef<str>) -> Vec<GlyphId> {
        self.text_to_glyphs_vec(str.as_ref())
    }

    /// Convenience variant of [`Self::text_to_glyphs()`] that returns the glyphs as a `Vec`.
    pub fn text_to_glyphs_vec(&self, text: impl EncodedText) -> Vec<GlyphId> {
        let count = self.count_text(&text);
        let mut glyphs: Vec<GlyphId> = vec![Default::default(); count];
        let resulting_count = self.text_to_glyphs(text, glyphs.as_mut_slice());
        assert_eq!(count, resulting_count);
        glyphs
    }

    /// Returns the advance width of `str` and the bounding box of text.
    ///
    /// See [`Self::measure_text()`] for details.
    pub fn measure_str(&self, str: impl AsRef<str>, paint: Option<&Paint>) -> (scalar, Rect) {
        self.measure_text(str.as_ref(), paint)
    }

    /// Returns the advance width of text.
    /// The advance is the normal distance to move before drawing additional text.
    /// Returns the bounding box of text relative to (0, 0). The paint
    /// stroke settings, mask filter, or path effect may modify the bounds.
    ///
    /// - `text` character storage encoded with [`crate::TextEncoding`]
    /// - `paint` optional; may be `None`
    ///
    /// Returns the sum of the default advance widths.
    pub fn measure_text(&self, text: impl EncodedText, paint: Option<&Paint>) -> (scalar, Rect) {
        let mut bounds = Rect::default();
        let (ptr, size, encoding) = text.as_raw();
        let width = unsafe {
            self.native().measureText(
                ptr,
                size,
                encoding.into_native(),
                bounds.native_mut(),
                paint.native_ptr_or_null(),
            )
        };

        (width, bounds)
    }

    /// Returns glyph index for Unicode character.
    ///
    /// If the character is not supported by the [`Typeface`], returns 0.
    ///
    /// - `uni` Unicode character
    pub fn unichar_to_glyph(&self, uni: Unichar) -> GlyphId {
        unsafe { self.native().unicharToGlyph(uni) }
    }

    /// Returns the glyph indices for each Unicode character in `uni`.
    pub fn unichar_to_glyphs(&self, uni: &[Unichar], glyphs: &mut [GlyphId]) {
        assert_eq!(uni.len(), glyphs.len());
        unsafe {
            sb::C_SkFont_unicharsToGlyphs(
                self.native(),
                uni.as_ptr(),
                uni.len(),
                glyphs.as_mut_ptr(),
            )
        }
    }

    /// Retrieves the advance for each glyph in `glyphs`.
    ///
    /// See [`Self::get_widths_bounds()`] for details.
    pub fn get_widths(&self, glyphs: &[GlyphId], widths: &mut [scalar]) {
        self.get_widths_bounds(glyphs, Some(widths), None, None)
    }

    /// Returns a strike ref for this font's current settings.
    ///
    /// A [`StrikeRef`] caches the resolved strike (font metrics engine), avoiding the overhead
    /// of descriptor construction, hashing, and global cache lookup on each glyph query.
    /// This is useful when making many glyph metric calls (e.g. [`Self::get_widths()`]) with the
    /// same font configuration.
    ///
    /// The returned [`StrikeRef`] is independent of this [`Font`]; subsequent changes to this
    /// [`Font`] do not affect it. Create a new [`StrikeRef`] after changing font properties.
    pub fn make_strike_ref(&self) -> StrikeRef {
        let strike_ref =
            StrikeRef::construct(|s| unsafe { sb::C_SkFont_makeStrikeRef(self.native(), s) });
        assert!(
            unsafe { sb::C_SkStrikeRef_isValid(strike_ref.native()) },
            "SkFont::makeStrikeRef() returned an invalid SkStrikeRef"
        );
        strike_ref
    }

    /// Retrieves the advance and bounds for each glyph in `glyphs`.
    ///
    /// - `glyphs` array of glyph indices to be measured
    /// - `widths` returns text advances for each glyph; may be `None`
    /// - `bounds` returns bounds for each glyph relative to (0, 0); may be `None`
    /// - `paint` optional, specifies stroking, [`crate::PathEffect`] and [`crate::MaskFilter`]
    pub fn get_widths_bounds(
        &self,
        glyphs: &[GlyphId],
        mut widths: Option<&mut [scalar]>,
        mut bounds: Option<&mut [Rect]>,
        paint: Option<&Paint>,
    ) {
        let count = glyphs.len();

        {
            if let Some(slice) = &widths {
                assert_eq!(count, slice.len())
            };
            if let Some(slice) = &bounds {
                assert_eq!(count, slice.len())
            };
        }

        let bounds_ptr = bounds.native_mut().as_ptr_or_null_mut();
        let widths_ptr = widths.as_ptr_or_null_mut();
        let paint_ptr = paint.native_ptr_or_null();

        unsafe {
            sb::C_SkFont_getWidthBounds(
                self.native(),
                glyphs.as_ptr(),
                count,
                widths_ptr,
                bounds_ptr,
                paint_ptr,
            )
        }
    }

    /// Retrieves the bounds for each glyph in `glyphs`.
    /// If `paint` is not `None`, its stroking, [`crate::PathEffect`], and [`crate::MaskFilter`] fields are
    /// respected.
    ///
    /// - `glyphs` array of glyph indices to be measured
    /// - `bounds` returns bounds for each glyph relative to (0, 0)
    /// - `paint` optional, specifies stroking, [`crate::PathEffect`], and [`crate::MaskFilter`]
    pub fn get_bounds(&self, glyphs: &[GlyphId], bounds: &mut [Rect], paint: Option<&Paint>) {
        self.get_widths_bounds(glyphs, None, Some(bounds), paint)
    }

    /// Retrieves the positions for each glyph, beginning at the specified origin.
    ///
    /// - `glyphs` array of glyph indices to be positioned
    /// - `pos` returns glyphs positions
    /// - `origin` location of the first glyph. Defaults to `{0, 0}`.
    pub fn get_pos(&self, glyphs: &[GlyphId], pos: &mut [Point], origin: Option<Point>) {
        let count = glyphs.len();
        assert_eq!(count, pos.len());

        let origin = origin.unwrap_or_default();

        unsafe {
            sb::C_SkFont_getPos(
                self.native(),
                glyphs.as_ptr(),
                count,
                pos.native_mut().as_mut_ptr(),
                *origin.native(),
            )
        }
    }

    /// Retrieves the x-positions for each glyph, beginning at the specified origin.
    ///
    /// - `glyphs` array of glyph indices to be positioned
    /// - `x_pos` returns glyphs x-positions
    /// - `origin` x-position of the first glyph. Defaults to 0.
    pub fn get_x_pos(&self, glyphs: &[GlyphId], x_pos: &mut [scalar], origin: Option<scalar>) {
        let count = glyphs.len();
        assert_eq!(count, x_pos.len());
        let origin = origin.unwrap_or_default();

        unsafe {
            sb::C_SkFont_getXPos(
                self.native(),
                glyphs.as_ptr(),
                count,
                x_pos.as_mut_ptr(),
                origin,
            )
        }
    }

    /// Returns intervals `[start, end]` describing lines parallel to the advance that intersect
    /// with the glyphs.
    ///
    /// - `glyphs` the glyphs to intersect
    /// - `pos` the position of each glyph
    /// - `top` the top of the line intersecting
    /// - `bottom` the bottom of the line intersecting
    /// - `paint` optional; may be `None`
    ///
    /// Returns an array of pairs of x values `[start, end]`. May be empty.
    pub fn get_intercepts<'a>(
        &self,
        glyphs: &[GlyphId],
        pos: &[Point],
        (top, bottom): (scalar, scalar),
        paint: impl Into<Option<&'a Paint>>,
    ) -> Vec<scalar> {
        assert_eq!(glyphs.len(), pos.len());
        let count = glyphs.len();
        let mut r: Vec<scalar> = Vec::new();
        let mut set = |scalars: &[scalar]| r = scalars.to_vec();
        unsafe {
            sb::C_SkFont_getIntercepts(
                self.native(),
                glyphs.as_ptr(),
                count,
                pos.native().as_ptr(),
                top,
                bottom,
                paint.into().native_ptr_or_null(),
                VecSink::new(&mut set).native_mut(),
            );
        }
        r
    }

    /// If the specified glyph can be represented as a path, return its path.
    /// If it is not (e.g. it is represented with a bitmap) return `None`.
    ///
    /// Note: an 'empty' glyph (e.g. what a space " " character might map to) can return
    /// a path, but that path may have zero contours.
    pub fn get_path(&self, glyph_id: GlyphId) -> Option<Path> {
        let mut path = Path::default();
        unsafe { sb::C_SkFont_getPath(self.native(), glyph_id, path.native_mut()) }.then_some(path)
    }

    // TODO: getPaths() (needs a function to be passed, but supports a context).

    /// Returns [`FontMetrics`] associated with [`Typeface`].
    /// The return value is the recommended spacing between lines: the sum of metrics
    /// descent, ascent, and leading.
    /// Results are scaled by text size but does not take into account
    /// dimensions required by text scale, text skew, fake bold,
    /// style stroke, and [`crate::PathEffect`].
    ///
    /// Returns the recommended spacing between lines.
    pub fn metrics(&self) -> (scalar, FontMetrics) {
        let mut line_spacing = 0.0;
        let fm =
            FontMetrics::construct(|fm| line_spacing = unsafe { self.native().getMetrics(fm) });
        (line_spacing, fm)
    }

    /// Returns the recommended spacing between lines: the sum of metrics
    /// descent, ascent, and leading.
    /// Result is scaled by text size but does not take into account
    /// dimensions required by stroking and [`crate::PathEffect`].
    /// Returns the same result as [`Self::metrics()`].
    pub fn spacing(&self) -> scalar {
        unsafe { self.native().getMetrics(ptr::null_mut()) }
    }
}

#[cfg(test)]
mod tests {
    use crate::{FontMgr, FontStyle};

    use super::*;

    #[test]
    fn test_flags() {
        let font_mgr = FontMgr::new();
        let typeface = font_mgr
            .legacy_make_typeface(None, FontStyle::normal())
            .unwrap();
        let mut font = Font::new(typeface, 10.0);

        font.set_force_auto_hinting(true);
        assert!(font.is_force_auto_hinting());
        font.set_force_auto_hinting(false);
        assert!(!font.is_force_auto_hinting());

        font.set_embolden(true);
        assert!(font.is_embolden());
        font.set_embolden(false);
        assert!(!font.is_embolden());
    }
}
