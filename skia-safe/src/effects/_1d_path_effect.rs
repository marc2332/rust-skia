use crate::{Path, PathEffect, scalar};

impl PathEffect {
    /// Dashes the path by replicating the specified path.
    ///
    /// - `path` the path to replicate (dash)
    /// - `advance` the space between instances of `path`
    /// - `phase` distance (mod `advance`) along path for its initial position
    /// - `style` how to transform path at each point (based on the current position and
    ///   tangent)
    pub fn path_1d(
        path: &Path,
        advance: scalar,
        phase: scalar,
        style: path_1d_path_effect::Style,
    ) -> Option<PathEffect> {
        path_1d_path_effect::new(path, advance, phase, style)
    }
}

pub mod path_1d_path_effect {
    //! Dash a path by replicating the specified path along it.

    use crate::{Path, PathEffect, prelude::*, scalar};
    use skia_bindings::C_SkPath1DPathEffect_Make;

    /// How the replicated path is transformed at each point.
    pub use skia_bindings::SkPath1DPathEffect_Style as Style;
    variant_name!(Style::Translate);

    /// Dash by replicating the specified path.
    ///
    /// - `path` the path to replicate (dash)
    /// - `advance` the space between instances of `path`
    /// - `phase` distance (mod `advance`) along path for its initial position
    /// - `style` how to transform path at each point (based on the current position and
    ///   tangent)
    pub fn new(path: &Path, advance: scalar, phase: scalar, style: Style) -> Option<PathEffect> {
        PathEffect::from_ptr(unsafe {
            C_SkPath1DPathEffect_Make(path.native(), advance, phase, style)
        })
    }
}
