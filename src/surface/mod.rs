use failure::Fail;

/// Represents an abstract Surface which provides the objects required for the rendering platform
///
/// For windows this would very likely be a `HWND` (window handle), for Vulkan a `SurfaceKHR`.
/// In the end however it is up to the render engine what it requests, but the 2 shipped with Nova
/// will require the objects described above
///
/// Furthermore do the generic serve as compile time checks. For example, it will prevent that you
/// can even pass a X11 window to DX12 in the code, as a X11 window won't implement
/// `Surface<HWND>`
pub trait Surface<T> {
    /// Creates or retrieves the object of the type `T` required for the current platform
    fn platform_object() -> Result<T, SurfaceError>;
}

/// Errors that can occur during creation/access of the underlying platform object
#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum SurfaceError {
    #[fail(display = "Failed to create or access the underlying object")]
    CreationOrAccessFailed,

    #[fail(display = "Invalid parameters passed: {}", details)]
    InvalidParameters { details: String },

    // This is a special case and will usually not occur with the implementations provided
    // by Nova because the generics will prevent this
    #[fail(display = "This Surface can not be used for creating this object")]
    NotSupported,
}
