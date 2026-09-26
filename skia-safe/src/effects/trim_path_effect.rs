//! A [`crate::PathEffect`] that returns a subset of a path, given start and stop `t` values
//! between 0 and 1.

use crate::{PathEffect, scalar};
use skia_bindings as sb;

/// Whether to return the trimmed subset or the complement.
pub use skia_bindings::SkTrimPathEffect_Mode as Mode;
variant_name!(Mode::Inverted);

impl PathEffect {
    /// Takes start and stop "t" values (values between 0...1), and returns a path that is
    /// that subset of the original path.
    ///
    /// e.g.
    /// - `trim(0.5, 1.0, ..)` returns the 2nd half of the path
    /// - `trim(0.33333, 0.66667, ..)` returns the middle third of the path
    ///
    /// The trim values apply to the entire path, so if it contains several contours, all of
    /// them are including in the calculation.
    ///
    /// `start_t` and `stop_t` must be 0..1 inclusive. If they are outside of that interval,
    /// they will be pinned to the nearest legal value. If either is NaN, `None` will be
    /// returned.
    ///
    /// Note: for [`Mode::Normal`], this will return one (logical) segment (even if it is
    /// spread across multiple contours). For [`Mode::Inverted`], this will return 2 logical
    /// segments: stopT..1 and 0...startT, in this order.
    pub fn trim(
        start_t: scalar,
        stop_t: scalar,
        mode: impl Into<Option<Mode>>,
    ) -> Option<PathEffect> {
        new(start_t, stop_t, mode)
    }
}

/// Takes start and stop "t" values (values between 0...1), and returns a path that is that
/// subset of the original path. See [`PathEffect::trim()`] for details.
pub fn new(start_t: scalar, stop_t: scalar, mode: impl Into<Option<Mode>>) -> Option<PathEffect> {
    PathEffect::from_ptr(unsafe {
        sb::C_SkTrimPathEffect_Make(start_t, stop_t, mode.into().unwrap_or(Mode::Normal))
    })
}
