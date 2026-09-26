/// Describes geometric operations (ala [`crate::region::RegionOp`]) that can be applied to coverage bytes.
/// These can be thought of as variants of porter-duff ([`crate::BlendMode`]) modes, but only
/// applied to the alpha channel.
///
/// See [`crate::MaskFilter`] for ways to use these when combining two different masks.
pub use skia_bindings::SkCoverageMode as CoverageMode;
variant_name!(CoverageMode::Union);
