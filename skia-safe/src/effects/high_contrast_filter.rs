//! A [`crate::ColorFilter`] that improves contrast for users with low vision, see
//! [`crate::HighContrastConfig`].

use crate::{ColorFilter, high_contrast_config::InvertStyle, prelude::*, scalar};
use skia_bindings::{self as sb, SkHighContrastConfig};

pub mod high_contrast_config {
    //! Types used by [`crate::HighContrastConfig`], e.g. the [`InvertStyle`] enum.

    /// Whether to invert brightness, lightness, or neither.
    pub use skia_bindings::SkHighContrastConfig_InvertStyle as InvertStyle;
    #[test]
    fn invert_style_naming() {
        let _ = InvertStyle::InvertLightness;
    }
}

/// Configuration struct for [`ColorFilter::high_contrast()`].
///
/// Provides transformations to improve contrast for users with low vision.
#[repr(C)]
#[derive(Clone, PartialEq, Debug)]
pub struct HighContrastConfig {
    /// If true, the color will be converted to grayscale.
    pub grayscale: bool,
    /// Whether to invert brightness, lightness, or neither.
    pub invert_style: InvertStyle,
    /// After grayscale and inverting, the contrast can be adjusted linearly. The valid range
    /// is -1.0 through 1.0, where 0.0 is no adjustment.
    pub contrast: scalar,
}

native_transmutable!(SkHighContrastConfig, HighContrastConfig);

impl Default for HighContrastConfig {
    fn default() -> Self {
        Self {
            grayscale: false,
            invert_style: InvertStyle::NoInvert,
            contrast: 0.0,
        }
    }
}

impl HighContrastConfig {
    /// Creates a config with the given settings.
    ///
    /// - `grayscale` if true, the color will be converted to grayscale
    /// - `invert_style` whether to invert brightness, lightness, or neither
    /// - `contrast` after grayscale and inverting, the contrast can be adjusted linearly.
    ///   The valid range is -1.0 through 1.0, where 0.0 is no adjustment.
    pub fn new(grayscale: bool, invert_style: InvertStyle, contrast: scalar) -> Self {
        Self {
            grayscale,
            invert_style,
            contrast,
        }
    }

    /// Returns true if all of the fields are set within the valid range.
    pub fn is_valid(&self) -> bool {
        self.contrast >= -1.0 && self.contrast <= 1.0
    }
}

impl ColorFilter {
    /// Color filter that provides transformations to improve contrast for users with low
    /// vision.
    ///
    /// Applies the following transformations in this order. Each of these can be configured
    /// using [`HighContrastConfig`].
    ///
    /// - Conversion to grayscale
    /// - Color inversion (either in RGB or HSL space)
    /// - Increasing the resulting contrast.
    ///
    /// Returns `None` if the config is invalid, e.g. if the contrast is outside the range of
    /// -1.0 to 1.0.
    pub fn high_contrast(config: &HighContrastConfig) -> Option<Self> {
        new(config)
    }
}

/// Returns the filter, or `None` if the config is invalid.
pub fn new(config: &HighContrastConfig) -> Option<ColorFilter> {
    ColorFilter::from_ptr(unsafe { sb::C_SkHighContrastFilter_Make(config.native()) })
}
