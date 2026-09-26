//! Global Skia graphics state: initialization and font cache configuration.

use skia_bindings::SkGraphics;

/// Call this at process initialization time if your environment does not permit static global
/// initializers that execute code. `init` is thread-safe and idempotent.
pub fn init() {
    unsafe { SkGraphics::Init() };
}

/// Return the max number of bytes that should be used by the font cache. If the cache needs to
/// allocate more, it will purge previous entries. This max can be changed by calling
/// [`set_font_cache_limit`].
pub fn font_cache_limit() -> usize {
    unsafe { SkGraphics::GetFontCacheLimit() }
}

/// Specify the max number of bytes that should be used by the font cache. If the cache needs to
/// allocate more, it will purge previous entries.
///
/// Returns the previous setting, as if [`font_cache_limit`] had been called before the new limit
/// was set.
///
/// - `bytes` the maximum number of bytes for the font cache
pub fn set_font_cache_limit(bytes: usize) -> usize {
    unsafe { SkGraphics::SetFontCacheLimit(bytes) }
}

/// Return the number of bytes currently used by the font cache.
pub fn font_cache_used() -> usize {
    unsafe { SkGraphics::GetFontCacheUsed() }
}

/// Return the number of entries in the font cache. A cache "entry" is associated with each
/// typeface + point size + matrix.
pub fn font_cache_count_used() -> i32 {
    unsafe { SkGraphics::GetFontCacheCountUsed() }
}

/// Return the current limit to the number of entries in the font cache. A cache "entry" is
/// associated with each typeface + point size + matrix.
pub fn font_cache_count_limit() -> i32 {
    unsafe { SkGraphics::GetFontCacheCountLimit() }
}

/// Set the limit to the number of entries in the font cache, and return the previous value. If
/// this new value is lower than the previous, it will automatically try to purge entries to meet
/// the new limit.
///
/// - `count` the maximum number of entries for the font cache
pub fn set_font_cache_count_limit(count: i32) -> i32 {
    unsafe { SkGraphics::SetFontCacheCountLimit(count) }
}

/// Return the current limit to the number of entries in the typeface cache. A cache "entry" is
/// associated with each typeface.
pub fn typeface_cache_count_limit() -> i32 {
    unsafe { SkGraphics::GetTypefaceCacheCountLimit() }
}

/// Set the limit to the number of entries in the typeface cache, and return the previous value.
/// Changes to this only take effect the next time each cache object is modified.
///
/// - `count` the maximum number of entries for the typeface cache
pub fn set_typeface_cache_count_limit(count: i32) -> i32 {
    unsafe { SkGraphics::SetTypefaceCacheCountLimit(count) }
}

/// For debugging purposes, this will attempt to purge the font cache. It does not change the
/// limit, but will cause subsequent font measures and draws to be recreated, since they will no
/// longer be in the cache.
pub fn purge_font_cache() {
    unsafe { SkGraphics::PurgeFontCache() }
}

/// If the strike cache is above the cache limit, attempt to purge strikes with pinners. This
/// should be called after clients release locks on pinned strikes.
pub fn purge_pinned_font_cache() {
    unsafe { SkGraphics::PurgePinnedFontCache() }
}

/// Returns the memory used for temporary images and other resources.
pub fn resource_cache_total_bytes_used() -> usize {
    unsafe { SkGraphics::GetResourceCacheTotalBytesUsed() }
}

/// Get the memory usage limit for the resource cache, used for temporary bitmaps and other
/// resources. Entries are purged from the cache when the memory usage exceeds this limit.
pub fn resource_cache_total_bytes_limit() -> usize {
    unsafe { SkGraphics::GetResourceCacheTotalByteLimit() }
}

/// Set the memory usage limit for the resource cache, used for temporary bitmaps and other
/// resources. Entries are purged from the cache when the memory usage exceeds this limit.
///
/// - `new_limit` the maximum number of bytes for the resource cache
pub fn set_resource_cache_total_bytes_limit(new_limit: usize) -> usize {
    unsafe { SkGraphics::SetResourceCacheTotalByteLimit(new_limit) }
}

/// For debugging purposes, this will attempt to purge the resource cache. It does not change the
/// limit.
pub fn purge_resource_cache() {
    unsafe { SkGraphics::PurgeResourceCache() }
}

/// When the cacheable entry is very large, adding it to the cache can cause most or all of the
/// existing entries to be purged. To avoid this, a client can set a limit for a single allocation.
/// If a cacheable entry would have been cached, but its size exceeds this limit, then it is not
/// cached at all.
///
/// `None` is the default value, meaning Skia always attempts to cache entries.
pub fn resource_cache_single_allocation_byte_limit() -> Option<usize> {
    let size = unsafe { SkGraphics::GetResourceCacheSingleAllocationByteLimit() };
    if size != 0 { Some(size) } else { None }
}

/// When the cacheable entry is very large, adding it to the cache can cause most or all of the
/// existing entries to be purged. To avoid this, set a limit for a single allocation. If a
/// cacheable entry would have been cached, but its size exceeds this limit, then it is not cached
/// at all.
///
/// `None` is the default value, meaning Skia always attempts to cache entries.
///
/// - `new_limit` the maximum size of a single cache allocation
pub fn set_resource_cache_single_allocation_byte_limit(new_limit: Option<usize>) -> Option<usize> {
    let size = unsafe {
        SkGraphics::SetResourceCacheSingleAllocationByteLimit(new_limit.unwrap_or_default())
    };
    if size != 0 { Some(size) } else { None }
}

// TODO: dump_memory_statistics() (needs SkTraceMemoryDumpWrapper interop wrapper).

/// Free as much globally cached memory as possible. This will purge all private caches in Skia,
/// including font and image caches.
///
/// If there are caches associated with a GPU context, those will not be affected by this call.
pub fn purge_all_caches() {
    unsafe { SkGraphics::PurgeAllCaches() }
}

// TODO: ImageGeneratorFromEncodedDataFactory
// TODO: SetOpenTypeSVGDecoderFactory & GetOpenTypeSVGDecoderFactory
