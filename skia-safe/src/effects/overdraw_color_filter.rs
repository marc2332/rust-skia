//! A [`crate::ColorFilter`] that uses the value in the source alpha channel to set the
//! destination pixel, useful for visualizing overdraw.

use crate::{Color, ColorFilter, prelude::*};
use skia_bindings as sb;

pub const NUM_COLORS: usize = 6;

impl ColorFilter {
    /// Uses the value in the src alpha channel to set the dst pixel.
    ///
    /// - 0 > `colors[0]`
    /// - 1 > `colors[1]`
    /// - ...
    /// - 5 (or larger) > `colors[5]`
    pub fn overdraw(colors: &[Color; NUM_COLORS]) -> ColorFilter {
        new(colors)
    }
}

pub fn new(colors: &[Color; NUM_COLORS]) -> ColorFilter {
    ColorFilter::from_ptr(unsafe {
        sb::C_SkOverdrawColorFilter_MakeWithSkColors(colors.native().as_ptr())
    })
    .unwrap()
}
