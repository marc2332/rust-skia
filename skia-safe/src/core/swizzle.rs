use skia_bindings as sb;

/// Swizzles the byte order of 32-bit pixels, swapping R and B (RGBA ↔ BGRA).
///
/// - `dest` destination pixels
/// - `src` source pixels
pub fn swap_rb(dest: &mut [u32], src: &[u32]) {
    assert_eq!(dest.len(), src.len());
    unsafe {
        sb::SkSwapRB(
            dest.as_mut_ptr(),
            src.as_ptr(),
            dest.len().try_into().unwrap(),
        )
    }
}

/// Swaps R and B in place (RGBA ↔ BGRA).
///
/// - `pixels` pixels to swizzle in place
pub fn swap_rb_inplace(pixels: &mut [u32]) {
    unsafe {
        sb::SkSwapRB(
            pixels.as_mut_ptr(),
            pixels.as_ptr(),
            pixels.len().try_into().unwrap(),
        )
    }
}
