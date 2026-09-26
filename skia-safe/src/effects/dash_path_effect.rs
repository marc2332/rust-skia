//! A [`crate::PathEffect`] that dashes a path by copying it with the specified intervals.
//! Only affects stroked paths.

use skia_bindings as sb;

use crate::{PathEffect, scalar};

impl PathEffect {
    /// Dashes the path by copying it with the specified intervals.
    ///
    /// - `intervals` array containing an even number of entries (>=2), with the even indices
    ///   specifying the length of "on" intervals, and the odd indices specifying the length of
    ///   "off" intervals
    /// - `phase` offset into the intervals array (mod the sum of all of the intervals). For
    ///   example: if `intervals` is `[10, 20]` and `phase` is 25, this will set up a dashed
    ///   path like so: 5 pixels off, 10 pixels on, 20 pixels off, 10 pixels on, 20 pixels off,
    ///   ... A phase of -5, 25, 55, 85, etc. would all result in the same path, because the
    ///   sum of all the intervals is 30.
    ///
    /// Note: only affects stroked paths.
    pub fn dash(intervals: &[scalar], phase: scalar) -> Option<Self> {
        new(intervals, phase)
    }
}

/// Dashes the path by copying it with the specified intervals.
///
/// - `intervals` array containing an even number of entries (>=2), with the even indices
///   specifying the length of "on" intervals, and the odd indices specifying the length of
///   "off" intervals
/// - `phase` offset into the intervals array (mod the sum of all of the intervals)
///
/// Note: only affects stroked paths.
pub fn new(intervals: &[scalar], phase: scalar) -> Option<PathEffect> {
    PathEffect::from_ptr(unsafe {
        sb::C_SkDashPathEffect_Make(intervals.as_ptr(), intervals.len(), phase)
    })
}
