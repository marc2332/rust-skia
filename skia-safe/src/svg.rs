//! SVG (Scalable Vector Graphics) support: an SVG [`Canvas`] and the SVG DOM types for rendering SVG documents.
pub mod canvas;

pub use self::canvas::Canvas;

#[cfg(feature = "svg")]
pub use crate::modules::svg::*;
