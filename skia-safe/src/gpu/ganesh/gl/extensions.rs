//! Helpers for querying and caching the current GL context's extensions.

use crate::prelude::*;
use skia_bindings::{self as sb, GrGLExtensions};
use std::{ffi::CString, fmt};

/// This helper queries the current GL context for its extensions, remembers them, and can be
/// queried. It supports both glGetString- and glGetStringi-style extension string APIs and will
/// use the latter if it is available. It also will query for EGL extensions if a eglQueryString
/// implementation is provided.
pub type Extensions = Handle<GrGLExtensions>;
unsafe_send_sync!(Extensions);

impl NativeDrop for GrGLExtensions {
    fn drop(&mut self) {
        unsafe { sb::C_GrGLExtensions_destruct(self) }
    }
}

impl NativeClone for GrGLExtensions {
    fn clone(&self) -> Self {
        unsafe { sb::GrGLExtensions::new(self) }
    }
}

impl fmt::Debug for Extensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Extensions")
            .field("is_initialized", &self.is_initialized())
            .finish()
    }
}

impl Extensions {
    // TODO: support new() / init?

    pub fn is_initialized(&self) -> bool {
        self.native().fInitialized
    }

    /// Queries whether an extension is present. This will fail if init() has not been called.
    pub fn has(&self, extension: impl AsRef<str>) -> bool {
        let extension = CString::new(extension.as_ref()).unwrap();
        unsafe { self.native().has(extension.as_ptr()) }
    }

    /// Removes an extension if present. Returns true if the extension was present before the call.
    pub fn remove(&mut self, extension: impl AsRef<str>) -> bool {
        let extension = CString::new(extension.as_ref()).unwrap();
        unsafe { self.native_mut().remove(extension.as_ptr()) }
    }

    /// Adds an extension to list
    pub fn add(&mut self, extension: impl AsRef<str>) {
        let extension = CString::new(extension.as_ref()).unwrap();
        unsafe { self.native_mut().add(extension.as_ptr()) }
    }

    pub fn reset(&mut self) {
        unsafe { sb::C_GrGLExtensions_reset(self.native_mut()) }
    }

    // TODO: dumpJSON()?
}
