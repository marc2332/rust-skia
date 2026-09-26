//! Parameters of a variation font: [`VariationAxis`] describes a single axis of a variable font.

pub use variation::Axis as VariationAxis;

pub mod variation {
    //! The [`Axis`] of a variation font: a four character tag with minimum, default, and maximum
    //! values.
    use crate::{FourByteTag, prelude::*};
    use skia_bindings::{self as sb, SkFontParameters_Variation_Axis};

    #[repr(C)]
    #[derive(Clone, PartialEq, Default, Debug)]
    pub struct Axis {
        /// Four character identifier of the font axis (weight, width, slant, italic...).
        pub tag: FourByteTag,
        /// Minimum value supported by this axis.
        pub min: f32,
        /// Default value set by this axis.
        pub def: f32,
        /// Maximum value supported by this axis. The maximum can equal the minimum.
        pub max: f32,
        flags: u16,
    }

    native_transmutable!(SkFontParameters_Variation_Axis, Axis);

    impl Axis {
        pub const fn new(tag: FourByteTag, min: f32, def: f32, max: f32, hidden: bool) -> Self {
            #[allow(clippy::bool_to_int_with_if)]
            Axis {
                tag,
                min,
                def,
                max,
                flags: if hidden { 1 } else { 0 },
            }
        }

        /// Returns whether this axis is recommended to remain hidden in user interfaces.
        pub fn is_hidden(&self) -> bool {
            unsafe { sb::C_SkFontParameters_Variation_Axis_isHidden(self.native()) }
        }

        /// Sets this axis to remain hidden in user interfaces.
        ///
        /// - `hidden` whether the axis should be hidden
        pub fn set_hidden(&mut self, hidden: bool) -> &mut Self {
            unsafe {
                sb::C_SkFontParameters_Variation_Axis_setHidden(self.native_mut(), hidden);
            }
            self
        }
    }
}
