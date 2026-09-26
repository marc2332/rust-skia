/// Describes how a shader draws outside of its original bounds: [`TileMode::Clamp`] replicates the
/// edge color, [`TileMode::Repeat`] repeats the image horizontally and vertically,
/// [`TileMode::Mirror`] repeats the image with alternating mirror images so adjacent images always
/// seam, and [`TileMode::Decal`] draws only within the original domain, returning transparent-black
/// everywhere else.
pub use skia_bindings::SkTileMode as TileMode;
variant_name!(TileMode::Mirror);
