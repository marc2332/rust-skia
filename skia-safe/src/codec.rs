//! Decoding of encoded images into [`crate::Image`]s and [`crate::Pixmap`]s.
//!
//! The [`Codec`] type decodes an encoded image from a stream or data, and the [`codecs`] module
//! provides the [`codecs::Decoder`] implementations for the supported formats.

// TODO: wrap SkAndroidCodec.h, SkCodecAnimation.h

mod _codec;
pub mod codec_animation;
mod decoders;
mod encoded_image_format;
mod encoded_origin;
pub mod pixmap_utils;

pub use _codec::*;
pub use decoders::*;
pub use encoded_image_format::*;
pub use encoded_origin::*;
