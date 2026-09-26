//! The metrics of a font. The metric values are consistent with the Skia y-down coordinate
//! system.

use crate::scalar;
use skia_bindings::{self as sb, SkFontMetrics};

bitflags! {
    /// Indicates when certain metrics are valid; the underline or strikeout metrics may be valid
    /// and zero. Fonts with embedded bitmaps may not have valid underline or strikeout metrics.
    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Flags: u32 {
        const UNDERLINE_THICKNESS_IS_VALID = sb::SkFontMetrics_FontMetricsFlags_kUnderlineThicknessIsValid_Flag as _;
        const UNDERLINE_POSITION_IS_VALID = sb::SkFontMetrics_FontMetricsFlags_kUnderlinePositionIsValid_Flag as _;
        const STRIKEOUT_THICKNESS_IS_VALID = sb::SkFontMetrics_FontMetricsFlags_kStrikeoutThicknessIsValid_Flag as _;
        const STRIKEOUT_POSITION_IS_VALID = sb::SkFontMetrics_FontMetricsFlags_kStrikeoutPositionIsValid_Flag as _;
        const BOUNDS_INVALID = sb::SkFontMetrics_FontMetricsFlags_kBoundsInvalid_Flag as _;
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct FontMetrics {
    flags: Flags,
    /// Greatest extent above origin of any glyph bounding box, typically negative; deprecated with
    /// variable fonts.
    pub top: scalar,
    /// Distance to reserve above baseline, typically negative.
    pub ascent: scalar,
    /// Distance to reserve below baseline, typically positive.
    pub descent: scalar,
    /// Greatest extent below origin of any glyph bounding box, typically positive; deprecated with
    /// variable fonts.
    pub bottom: scalar,
    /// Distance to add between lines, typically positive or zero.
    pub leading: scalar,
    /// Average character width, zero if unknown.
    pub avg_char_width: scalar,
    /// Maximum character width, zero if unknown.
    pub max_char_width: scalar,
    /// Greatest extent to left of origin of any glyph bounding box, typically negative; deprecated
    /// with variable fonts.
    pub x_min: scalar,
    /// Greatest extent to right of origin of any glyph bounding box, typically positive; deprecated
    /// with variable fonts.
    pub x_max: scalar,
    /// Height of lower-case 'x', zero if unknown, typically negative.
    pub x_height: scalar,
    /// Height of an upper-case letter, zero if unknown, typically negative.
    pub cap_height: scalar,
    underline_thickness: scalar,
    underline_position: scalar,
    strikeout_thickness: scalar,
    strikeout_position: scalar,
}

native_transmutable!(SkFontMetrics, FontMetrics);

impl FontMetrics {
    /// Returns `Some(thickness)` if the font metrics have a valid underline thickness, otherwise
    /// `None`.
    pub fn underline_thickness(&self) -> Option<scalar> {
        self.if_valid(
            Flags::UNDERLINE_THICKNESS_IS_VALID,
            self.underline_thickness,
        )
    }

    /// Returns `Some(position)` if the font metrics have a valid underline position, otherwise
    /// `None`.
    pub fn underline_position(&self) -> Option<scalar> {
        self.if_valid(Flags::UNDERLINE_POSITION_IS_VALID, self.underline_position)
    }

    /// Returns `Some(thickness)` if the font metrics have a valid strikeout thickness, otherwise
    /// `None`.
    pub fn strikeout_thickness(&self) -> Option<scalar> {
        self.if_valid(
            Flags::STRIKEOUT_THICKNESS_IS_VALID,
            self.strikeout_thickness,
        )
    }

    /// Returns `Some(position)` if the font metrics have a valid strikeout position, otherwise
    /// `None`.
    pub fn strikeout_position(&self) -> Option<scalar> {
        self.if_valid(Flags::STRIKEOUT_POSITION_IS_VALID, self.strikeout_position)
    }

    fn if_valid(&self, flag: self::Flags, value: scalar) -> Option<scalar> {
        self.flags.contains(flag).then_some(value)
    }

    /// Returns true if the font metrics have a valid `top`, `bottom`, `x_min`, and `x_max`. If the
    /// bounds are not valid, returns false.
    pub fn has_bounds(&self) -> bool {
        !self.flags.contains(Flags::BOUNDS_INVALID)
    }
}
