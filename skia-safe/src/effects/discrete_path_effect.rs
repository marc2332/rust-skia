//! A [`crate::PathEffect`] that chops a path into discrete segments and randomly displaces
//! them.

use crate::{PathEffect, scalar};
use skia_bindings as sb;

impl PathEffect {
    /// Chops a path into discrete segments, and randomly displaces them.
    ///
    /// - `seg_length` break the path into segments of this length
    /// - `dev` randomly move the endpoints away from the original path by a maximum of this
    ///   deviation
    /// - `seed_assist` a caller-supplied seed that modifies the seed value that is used to
    ///   randomize the path segments' endpoints. If `None`, it defaults to 0, in which case
    ///   filtering a path multiple times will result in the same set of segments (this is
    ///   useful for testing). If a caller does not want this behaviour they can pass in a
    ///   different `seed_assist` to get a different set of path segments.
    ///
    /// Note: works on filled or framed paths.
    pub fn discrete(
        seg_length: scalar,
        dev: scalar,
        seed_assist: impl Into<Option<u32>>,
    ) -> Option<Self> {
        new(seg_length, dev, seed_assist)
    }
}

/// Chops a path into discrete segments, and randomly displaces them.
pub fn new(
    seg_length: scalar,
    dev: scalar,
    seed_assist: impl Into<Option<u32>>,
) -> Option<PathEffect> {
    PathEffect::from_ptr(unsafe {
        sb::C_SkDiscretePathEffect_Make(seg_length, dev, seed_assist.into().unwrap_or(0))
    })
}
