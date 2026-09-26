//! A [`crate::ColorFilter`] that multiplies the luma of its input into the alpha channel and
//! sets the color channels to zero.

use crate::ColorFilter;
use skia_bindings as sb;

impl ColorFilter {
    /// Multiplies the luma of its input into the alpha channel, and sets the red, green, and
    /// blue channels to zero.
    ///
    /// `luma(r, g, b, a)` = `{0, 0, 0, a * luma(r, g, b)}`
    ///
    /// This is similar to a luminanceToAlpha feColorMatrix, but note how this filter folds in
    /// the previous alpha, something an feColorMatrix cannot do:
    ///
    /// `feColorMatrix(luminanceToAlpha; r, g, b, a)` = `{0, 0, 0, luma(r, g, b)}`
    pub fn luma() -> Self {
        new()
    }
}

pub fn new() -> ColorFilter {
    ColorFilter::from_ptr(unsafe { sb::C_SkLumaColorFilter_Make() }).unwrap()
}
