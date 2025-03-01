//! Display surface creation and management.

use cgmath::Vector2;
use failure::Fail;

/// Represents an abstract Surface which provides the objects required for the rendering platform.
///
/// For windows this would very likely be a `HWND` (window handle), for Vulkan a `SurfaceKHR`.
/// In the end however it is up to the render engine what it requests, but the 2 shipped with Nova
/// will require the objects described above.
///
/// Furthermore do the generic serve as compile time checks. For example, it will prevent that you
/// can even pass a X11 window to DX12 in the code, as a X11 window won't implement
/// `Surface<HWND>`.
pub trait Surface<T> {
    /// Creates or retrieves the object of the type `T` required for the current platform.
    fn platform_object(&mut self) -> Result<T, SurfaceError>;

    /// Retrieves the current surface size where x is width and y height
    fn get_current_size(&self) -> Vector2<u32>;
}

/// Errors that can occur during creation/access of the underlying platform object.
#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum SurfaceError {
    /// Failed to create or access the underlying platform object.
    #[fail(display = "Failed to create or access the underlying object.")]
    CreationOrAccessFailed,

    /// Invalid parameters passed to surface creation.
    #[fail(display = "Invalid parameters passed: {}", details)]
    InvalidParameters {
        /// Details on invalid parameters, platform specific.
        details: String,
    },

    /// This Surface can not be used for creating this object.
    ///
    /// This is a special case and will usually not occur with the implementations provided
    /// by Nova because the generics will prevent this.
    #[fail(display = "This Surface can not be used for creating this object.")]
    NotSupported,
}
