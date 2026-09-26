//! The Metal backend for Ganesh: the backend context, backend surfaces, direct-context
//! construction, and Metal types.

mod backend_context;
mod backend_surface;
mod direct_context;
pub(crate) mod surface_metal;
pub(crate) mod types;

pub use backend_context::*;
pub use backend_surface::*;
pub use direct_context::*;
