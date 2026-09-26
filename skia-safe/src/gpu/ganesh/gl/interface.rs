//! The OpenGL function-pointer interface used by
//! [`crate::gpu::ganesh::DirectContext`] to make all OpenGL calls.

use crate::{gpu::gl::Extensions, prelude::*};
use skia_bindings::{self as sb, GrGLInterface, SkRefCntBase};
use std::{ffi::c_void, fmt, os::raw};

/// [`crate::gpu::ganesh::DirectContext`] uses the following interface to make all calls into
/// OpenGL. When a [`crate::gpu::ganesh::DirectContext`] is created it is given a [`Self`]. The
/// interface's function pointers must be valid for the OpenGL context associated with the
/// [`crate::gpu::ganesh::DirectContext`]. On some platforms, such as Windows, function pointers
/// for OpenGL extensions may vary between OpenGL contexts. So the caller must be careful to use a
/// [`Self`] initialized for the correct context. All functions that should be available based on
/// the OpenGL's version and extension string must be non-NULL or
/// [`crate::gpu::ganesh::DirectContext`] creation will fail. This can be tested with the
/// [`Self::validate()`] method when the OpenGL context has been made current.
pub type Interface = RCHandle<GrGLInterface>;
require_type_equality!(sb::GrGLInterface_INHERITED, sb::SkRefCnt);

impl NativeRefCountedBase for GrGLInterface {
    type Base = SkRefCntBase;
}

impl fmt::Debug for Interface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Interface")
            .field("extensions", &self.extensions())
            .finish()
    }
}

impl Interface {
    /// Rather than depend on platform-specific GL headers and libraries, we require the client to
    /// provide a struct of GL function pointers. This struct can be specified per
    /// [`crate::gpu::ganesh::DirectContext`] as a parameter to
    /// [`crate::gpu::direct_contexts::make_gl()`]. If no interface is passed to
    /// [`crate::gpu::direct_contexts::make_gl()`] then a default GL interface is created
    /// using [`Self::new_native()`]. If this returns `None` then
    /// [`crate::gpu::direct_contexts::make_gl()`] will fail.
    ///
    /// The implementation of [`Self::new_native()`] is platform-specific. Several implementations
    /// have been provided (for GLX, WGL, EGL, etc), along with an implementation that simply
    /// returns `None`. Clients should select the most appropriate one to build.
    pub fn new_native() -> Option<Self> {
        Self::from_ptr(unsafe { sb::C_GrGLInterface_MakeNativeInterface() as _ })
    }

    /// Generic function for creating an [`Interface`] for either OpenGL or GLES. It calls
    /// `load_fn` to get each function address.
    pub fn new_load_with<F>(load_fn: F) -> Option<Self>
    where
        F: FnMut(&str) -> *const c_void,
    {
        Self::from_ptr(unsafe {
            sb::C_GrGLInterface_MakeAssembledInterface(
                &load_fn as *const _ as *mut c_void,
                Some(gl_get_proc_fn_wrapper::<F>),
            ) as _
        })
    }

    /// Generic function for creating an [`Interface`] for either OpenGL or GLES. It calls
    /// `load_fn` to get each function address.
    pub fn new_load_with_cstr<F>(load_fn: F) -> Option<Self>
    where
        F: FnMut(&std::ffi::CStr) -> *const c_void,
    {
        Self::from_ptr(unsafe {
            sb::C_GrGLInterface_MakeAssembledInterface(
                &load_fn as *const _ as *mut c_void,
                Some(gl_get_proc_fn_wrapper_cstr::<F>),
            ) as _
        })
    }

    pub fn validate(&self) -> bool {
        unsafe { self.native().validate() }
    }

    pub fn extensions(&self) -> &Extensions {
        Extensions::from_native_ref(unsafe {
            &*sb::C_GrGLInterface_extensions(self.native_mut_force())
        })
    }

    pub fn extensions_mut(&mut self) -> &mut Extensions {
        Extensions::from_native_ref_mut(unsafe {
            &mut *sb::C_GrGLInterface_extensions(self.native_mut())
        })
    }

    pub fn has_extension(&self, extension: impl AsRef<str>) -> bool {
        self.extensions().has(extension)
    }
}

unsafe extern "C" fn gl_get_proc_fn_wrapper<F>(
    ctx: *mut c_void,
    name: *const raw::c_char,
) -> *const c_void
where
    F: FnMut(&str) -> *const c_void,
{
    unsafe { (*(ctx as *mut F))(std::ffi::CStr::from_ptr(name).to_str().unwrap()) }
}

unsafe extern "C" fn gl_get_proc_fn_wrapper_cstr<F>(
    ctx: *mut c_void,
    name: *const raw::c_char,
) -> *const c_void
where
    F: FnMut(&std::ffi::CStr) -> *const c_void,
{
    unsafe { (*(ctx as *mut F))(std::ffi::CStr::from_ptr(name)) }
}
