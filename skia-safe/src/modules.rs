#[cfg(feature = "textlayout")]
pub(crate) mod paragraph;
#[cfg(any(feature = "svg", feature = "skottie"))]
pub mod resources;
#[cfg(feature = "textlayout")]
pub mod shaper;
#[cfg(feature = "skottie")]
pub mod skottie;
#[cfg(feature = "svg")]
pub mod svg;
#[cfg(feature = "textlayout")]
pub use shaper::{Shaper, icu};

// Export everything below paragraph under textlayout
#[cfg(feature = "textlayout")]
pub mod textlayout {
    //! Layout and rendering of multi-line, styled text via Skia's `skparagraph` module.
    pub use super::paragraph::*;
}

#[cfg(feature = "textlayout")]
pub mod shapers {
    //! Text shaping backends and helpers built on Skia's `SkShaper` (primitive, CoreText, HarfBuzz, and Unicode).
    // Re-exports `shapers::primitive`.
    pub use crate::shaper::shapers::*;

    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "visionos"))]
    pub mod ct {
        //! CoreText-based text shaping, available on macOS, iOS, and visionOS.
        pub use crate::shaper::core_text::*;
    }

    pub mod hb {
        //! HarfBuzz-based text shaping.
        pub use crate::shaper::harfbuzz::*;
    }

    pub mod unicode {
        //! Unicode-based text shaping (Skia's `SkUnicode`).
        pub use crate::shaper::unicode::*;
    }
}
