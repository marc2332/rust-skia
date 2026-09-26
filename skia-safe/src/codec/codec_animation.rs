//! Types for animated image codecs, such as GIF, describing how frames are blended and disposed.

use skia_bindings as sb;

/// How to blend the current frame.
///
/// Variants:
/// - [`Blend::SrcOver`]: Blend with the prior frame as if using [`crate::BlendMode::SrcOver`].
/// - [`Blend::Src`]: Blend with the prior frame as if using [`crate::BlendMode::Src`]. This frame's
///   pixels replace the destination pixels.
pub type Blend = sb::SkCodecAnimation_Blend;
variant_name!(Blend::SrcOver);

/// This specifies how the next frame is based on this frame.
///
/// Names are based on the GIF 89a spec.
///
/// The numbers correspond to values in a GIF.
///
/// Variants:
/// - [`DisposalMethod::Keep`]: The next frame should be drawn on top of this one. In a GIF, a value
///   of 0 (not specified) is also treated as `Keep`.
/// - [`DisposalMethod::RestoreBGColor`]: Similar to `Keep`, except the area inside this frame's
///   rectangle should be cleared to the BackGround color (transparent) before drawing the next
///   frame.
/// - [`DisposalMethod::RestorePrevious`]: The next frame should be drawn on top of the previous
///   frame - i.e. disregarding this one. In a GIF, a value of 4 is also treated as
///   `RestorePrevious`.
pub type DisposalMethod = sb::SkCodecAnimation_DisposalMethod;
variant_name!(DisposalMethod::Keep);
