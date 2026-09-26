//! A [`crate::PathEffect`] that turns sharp corners into various treatments (e.g. rounded
//! corners).

use crate::{PathEffect, scalar};
use skia_bindings as sb;

impl PathEffect {
    /// Turns sharp corners into various treatments (e.g. rounded corners).
    ///
    /// - `radius` must be > 0 to have an effect. It specifies the distance from each corner
    ///   that should be "rounded".
    pub fn corner_path(radius: scalar) -> Option<Self> {
        new(radius)
    }
}

/// Turns sharp corners into various treatments (e.g. rounded corners).
pub fn new(radius: scalar) -> Option<PathEffect> {
    PathEffect::from_ptr(unsafe { sb::C_SkCornerPathEffect_Make(radius) })
}
