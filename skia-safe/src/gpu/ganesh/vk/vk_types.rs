//! Low-level Vulkan types used by the Ganesh Vulkan backend, for interacting with resources
//! created externally to Skia.

use std::ptr;

use skia_bindings::{GrVkDrawableInfo, GrVkImageInfo, GrVkSurfaceInfo};

use crate::gpu::{
    self, Protected,
    vk::{self, Alloc, YcbcrConversionInfo},
};

pub use crate::gpu::vk::{GetProc, GetProcOf, GetProcResult};

/// When wrapping a [`crate::gpu::BackendTexture`] or [`crate::gpu::BackendRenderTarget`], the
/// `current_queue_family` should either be [`vk::QUEUE_FAMILY_IGNORED`], `VK_QUEUE_FAMILY_EXTERNAL`,
/// or `VK_QUEUE_FAMILY_FOREIGN_EXT`. If `sharing_mode` is [`vk::SharingMode::EXCLUSIVE`] then
/// `current_queue_family` can also be the graphics queue index passed into Skia.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct ImageInfo {
    image: vk::Image,
    alloc: Alloc,
    pub tiling: vk::ImageTiling,
    pub layout: vk::ImageLayout,
    pub format: vk::Format,
    pub image_usage_flags: vk::ImageUsageFlags,
    pub sample_count: u32,
    pub level_count: u32,
    pub current_queue_family: u32,
    pub protected: Protected,
    pub ycbcr_conversion_info: YcbcrConversionInfo,
    pub sharing_mode: vk::SharingMode,
}
unsafe_send_sync!(ImageInfo);

native_transmutable!(GrVkImageInfo, ImageInfo);

impl Default for ImageInfo {
    fn default() -> Self {
        Self {
            image: vk::NULL_HANDLE.into(),
            alloc: Alloc::default(),
            tiling: vk::ImageTiling::OPTIMAL,
            layout: vk::ImageLayout::UNDEFINED,
            format: vk::Format::UNDEFINED,
            image_usage_flags: 0,
            sample_count: 1,
            level_count: 0,
            current_queue_family: vk::QUEUE_FAMILY_IGNORED,
            protected: Protected::No,
            ycbcr_conversion_info: Default::default(),
            sharing_mode: vk::SharingMode::EXCLUSIVE,
        }
    }
}

impl ImageInfo {
    /// # Safety
    /// The Vulkan `image` and `alloc` must outlive the lifetime of the ImageInfo returned.
    #[allow(clippy::too_many_arguments)]
    pub unsafe fn new(
        image: vk::Image,
        alloc: Alloc,
        tiling: vk::ImageTiling,
        layout: vk::ImageLayout,
        format: vk::Format,
        level_count: u32,
        current_queue_family: impl Into<Option<u32>>,
        ycbcr_conversion_info: impl Into<Option<YcbcrConversionInfo>>,
        protected: impl Into<Option<Protected>>, // m77
        sharing_mode: impl Into<Option<vk::SharingMode>>, // m85
    ) -> Self {
        let current_queue_family = current_queue_family
            .into()
            .unwrap_or(vk::QUEUE_FAMILY_IGNORED);
        let ycbcr_conversion_info = ycbcr_conversion_info.into().unwrap_or_default();
        let protected = protected.into().unwrap_or(Protected::No);
        let sharing_mode = sharing_mode.into().unwrap_or(vk::SharingMode::EXCLUSIVE);
        Self {
            image,
            alloc,
            tiling,
            layout,
            format,
            level_count,
            current_queue_family,
            protected,
            ycbcr_conversion_info,
            sharing_mode,
            ..Self::default()
        }
    }

    /// # Safety
    /// The Vulkan `info.image` and `info.alloc` must outlive the lifetime of the ImageInfo returned.
    pub unsafe fn from_info(info: &ImageInfo, layout: vk::ImageLayout) -> Self {
        unsafe {
            Self::new(
                info.image,
                info.alloc,
                info.tiling,
                layout,
                info.format,
                info.level_count,
                info.current_queue_family,
                info.ycbcr_conversion_info,
                info.protected,
                info.sharing_mode,
            )
        }
    }

    /// # Safety
    /// The Vulkan `info.image` and `info.alloc` must outlive the lifetime of the ImageInfo returned.
    pub unsafe fn from_info_with_queue_index(
        info: &ImageInfo,
        layout: vk::ImageLayout,
        family_queue_index: u32,
    ) -> Self {
        unsafe {
            Self::new(
                info.image,
                info.alloc,
                info.tiling,
                layout,
                info.format,
                info.level_count,
                family_queue_index,
                info.ycbcr_conversion_info,
                info.protected,
                info.sharing_mode,
            )
        }
    }
}

impl ImageInfo {
    pub fn image(&self) -> &vk::Image {
        &self.image
    }

    /// # Safety
    /// `image` must outlive usages of `self`.
    pub unsafe fn set_image(&mut self, image: vk::Image) {
        self.image = image
    }

    pub fn alloc(&self) -> &Alloc {
        &self.alloc
    }

    /// # Safety
    /// `alloc` must outlive usages of `self`.
    pub unsafe fn set_alloc(&mut self, alloc: Alloc) {
        self.alloc = alloc;
    }
}

/// This object is wrapped in a [`crate::gpu::ganesh::vk::BackendDrawableInfo`] and passed in as
/// an argument to [`crate::drawable::gpu_draw_handler::GPUDrawHandler::draw()`] calls on a
/// [`crate::Drawable`]. The drawable will use this info to inject direct Vulkan calls into our
/// stream of GPU draws.
///
/// The [`crate::Drawable`] is given a secondary [`crate::gpu::vk::CommandBuffer`] in which to
/// record draws. The GPU backend will then execute that command buffer within a render pass it is
/// using for its own draws. The drawable is also given the attachment of the color index, a
/// compatible [`crate::gpu::vk::RenderPass`], and the [`crate::gpu::vk::Format`] of the color
/// attachment so that it can make `VkPipeline` objects for the draws. The [`crate::Drawable`] must
/// not alter the state of the [`crate::gpu::vk::RenderPass`] or sub pass.
///
/// Additionally, the [`crate::Drawable`] may fill in the passed in `draw_bounds` with the bounds
/// of the draws that it submits to the command buffer. This will be used by the GPU backend for
/// setting the bounds in `vkCmdBeginRenderPass`. If `draw_bounds` is not updated, we will assume
/// that the entire attachment may have been written to.
///
/// The [`crate::Drawable`] is always allowed to create its own command buffers and submit them to
/// the queue to render offscreen textures which will be sampled in draws added to the passed in
/// [`crate::gpu::vk::CommandBuffer`]. If this is done the [`crate::Drawable`] is in charge of
/// adding the required memory barriers to the queue for the sampled images since the Skia backend
/// will not do this.
#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct DrawableInfo {
    pub secondary_command_buffer: vk::CommandBuffer,
    pub color_attachment_index: u32,
    pub compatible_render_pass: vk::RenderPass,
    pub format: vk::Format,
    pub draw_bounds: *mut vk::Rect2D,
}

native_transmutable!(GrVkDrawableInfo, DrawableInfo);
unsafe_send_sync!(DrawableInfo);

impl Default for DrawableInfo {
    fn default() -> Self {
        DrawableInfo {
            secondary_command_buffer: vk::NULL_HANDLE.into(),
            color_attachment_index: 0,
            compatible_render_pass: vk::NULL_HANDLE.into(),
            format: vk::Format::UNDEFINED,
            draw_bounds: ptr::null_mut(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct SurfaceInfo {
    pub sample_count: u32,
    pub level_count: u32,
    pub protected: gpu::Protected,

    pub image_tiling: vk::ImageTiling,
    pub format: vk::Format,
    pub image_usage_flags: vk::ImageUsageFlags,
    pub ycbcr_conversion_info: vk::YcbcrConversionInfo,
    pub sharing_mode: vk::SharingMode,
}

native_transmutable!(GrVkSurfaceInfo, SurfaceInfo);

impl Default for SurfaceInfo {
    fn default() -> Self {
        Self {
            sample_count: 1,
            level_count: 0,
            protected: Protected::No,
            image_tiling: vk::ImageTiling::OPTIMAL,
            format: vk::Format::UNDEFINED,
            image_usage_flags: 0,
            ycbcr_conversion_info: Default::default(),
            sharing_mode: vk::SharingMode::EXCLUSIVE,
        }
    }
}
