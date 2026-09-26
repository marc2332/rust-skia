//! The smart container for pixel memory, used with [`crate::Bitmap`]. This class can be
//! shared/accessed between multiple threads.

use crate::{ISize, prelude::*};
use skia_bindings::{self as sb, SkPixelRef, SkRefCntBase};
use std::{fmt, os::raw::c_void};

pub type PixelRef = RCHandle<SkPixelRef>;
unsafe_send_sync!(PixelRef);
require_type_equality!(sb::SkPixelRef_INHERITED, sb::SkRefCnt);

impl NativeRefCountedBase for SkPixelRef {
    type Base = SkRefCntBase;
}

impl fmt::Debug for PixelRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PixelRef")
            .field("dimensions", &self.dimensions())
            .field("row_bytes", &self.row_bytes())
            .field("generation_id", &self.generation_id())
            .field("is_immutable", &self.is_immutable())
            .finish()
    }
}

impl PixelRef {
    // TODO: wrap constructor with pixels borrowed.

    pub fn dimensions(&self) -> ISize {
        ISize::new(self.width(), self.height())
    }

    pub fn width(&self) -> i32 {
        unsafe { sb::C_SkPixelRef_width(self.native()) }
    }

    pub fn height(&self) -> i32 {
        unsafe { sb::C_SkPixelRef_height(self.native()) }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn pixels(&self) -> *mut c_void {
        unsafe { sb::C_SkPixelRef_pixels(self.native()) }
    }

    pub fn row_bytes(&self) -> usize {
        unsafe { sb::C_SkPixelRef_rowBytes(self.native()) }
    }

    /// Returns a non-zero, unique value corresponding to the pixels in this pixel ref. Each time
    /// the pixels are changed (and [`Self::notify_pixels_changed()`] is called), a different
    /// generation ID will be returned.
    pub fn generation_id(&self) -> u32 {
        unsafe { self.native().getGenerationID() }
    }

    /// Call this if you have changed the contents of the pixels. This will in turn cause a
    /// different generation ID value to be returned from [`Self::generation_id()`].
    pub fn notify_pixels_changed(&mut self) {
        unsafe { self.native_mut().notifyPixelsChanged() }
    }

    /// Returns true if this pixel ref is marked as immutable, meaning that the contents of its
    /// pixels will not change for the lifetime of the pixel ref.
    pub fn is_immutable(&self) -> bool {
        unsafe { sb::C_SkPixelRef_isImmutable(self.native()) }
    }

    /// Marks this pixel ref as immutable, meaning that the contents of its pixels will not change
    /// for the lifetime of the pixel ref. This state can be set on a pixel ref, but it cannot be
    /// cleared once it is set.
    pub fn set_immutable(&mut self) {
        unsafe { self.native_mut().setImmutable() }
    }

    // TODO addGenIDChangeListener()

    /// Call when this pixel ref is part of the key to a resource cache entry. This allows the cache
    /// to know automatically those entries can be purged when this pixel ref is changed or deleted.
    pub fn notify_added_to_cache(&mut self) {
        unsafe { sb::C_SkPixelRef_notifyAddedToCache(self.native_mut()) }
    }
}
