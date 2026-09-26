//! Deprecated factory functions for Perlin noise shaders, use [`crate::shaders::fractal_noise()`]
//! and [`crate::shaders::turbulence()`] instead.

use crate::{ISize, Shader, scalar, shaders};

/// Creates fractal Perlin noise, see [`shaders::fractal_noise()`] for details.
pub fn fractal_noise(
    base_frequency: (scalar, scalar),
    num_octaves: usize,
    seed: scalar,
    tile_size: impl Into<Option<ISize>>,
) -> Option<Shader> {
    shaders::fractal_noise(base_frequency, num_octaves, seed, tile_size)
}

/// Creates Perlin turbulence, see [`shaders::turbulence()`] for details.
pub fn turbulence(
    base_frequency: (scalar, scalar),
    num_octaves: usize,
    seed: scalar,
    tile_size: impl Into<Option<ISize>>,
) -> Option<Shader> {
    shaders::turbulence(base_frequency, num_octaves, seed, tile_size)
}
