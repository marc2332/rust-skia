use std::fmt;

use skia_bindings::{self as sb, SkRefCntBase, skgpu_MutableTextureState};

use super::BackendApi;
use crate::prelude::*;

/// Since Skia and clients can both modify gpu textures and their connected state, Skia needs a
/// way for clients to inform us if they have modifiend any of this state. In order to not need
/// setters for every single API and state, we use this class to be a generic wrapper around all
/// the mutable state. This class is used for calls that inform Skia of these texture/image state
/// changes by the client as well as for requesting state changes to be done by Skia. The backend
/// specific state that is wrapped by this class are located in files like:
///   - `include/gpu/vk/VulkanMutableTextureState.h`
pub type MutableTextureState = RCHandle<skgpu_MutableTextureState>;
unsafe_send_sync!(MutableTextureState);

impl NativeRefCountedBase for skgpu_MutableTextureState {
    type Base = SkRefCntBase;
}

impl Default for MutableTextureState {
    fn default() -> Self {
        MutableTextureState::from_ptr(unsafe { sb::C_MutableTextureState_Construct() }).unwrap()
    }
}

impl fmt::Debug for MutableTextureState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut str = f.debug_struct("MutableTextureState");
        #[cfg(feature = "vulkan")]
        {
            str.field(
                "image_layout",
                &crate::gpu::vk::mutable_texture_states::get_vk_image_layout(self),
            )
            .field(
                "queue_family_index",
                &crate::gpu::vk::mutable_texture_states::get_vk_queue_family_index(self),
            );
        }
        str.field("backend", &self.backend()).finish()
    }
}

impl MutableTextureState {
    pub fn copied(&self) -> Self {
        MutableTextureState::from_ptr(unsafe {
            sb::C_MutableTextureState_CopyConstruct(self.native())
        })
        .unwrap()
    }

    #[cfg(feature = "vulkan")]
    #[deprecated(
        since = "0.72.0",
        note = "use gpu::vk::mutable_texture_states::new_vulkan()"
    )]
    pub fn new_vk(layout: crate::gpu::vk::ImageLayout, queue_family_index: u32) -> Self {
        crate::gpu::vk::mutable_texture_states::new_vulkan(layout, queue_family_index)
    }

    #[cfg(feature = "vulkan")]
    #[deprecated(
        since = "0.72.0",
        note = "use gpu::vk::mutable_texture_states::get_vk_image_layout()"
    )]
    pub fn vk_image_layout(&self) -> sb::VkImageLayout {
        crate::gpu::vk::mutable_texture_states::get_vk_image_layout(self)
    }

    #[cfg(feature = "vulkan")]
    #[deprecated(
        since = "0.72.0",
        note = "use gpu::vk::mutable_texture_states::get_vk_queue_family_index()"
    )]
    pub fn queue_family_index(&self) -> u32 {
        crate::gpu::vk::mutable_texture_states::get_vk_queue_family_index(self)
    }

    pub fn backend(&self) -> BackendApi {
        unsafe { sb::C_MutableTextureState_backend(self.native()) }
    }
}
