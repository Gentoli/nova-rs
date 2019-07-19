/// Represents an abstract Surface which provides the objects required for the rendering platform
pub trait Surface<T, O> {
    /// Creates or retrieves the object of the type `T` required for the current platform
    ///
    /// # Parameters
    ///
    /// * `options` - The options passed for retrieving the object
    fn platform_object(options: O) -> Result<T, SurfaceError>;
}

/// Errors that can occur during creation/access of the underlying platform object
#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum SurfaceError {
    #[fail(display = "Failed to create or access the underlying object")]
    AccessFailed,

    #[fail(display = "Invalid parameters passed")]
    InvalidParameters,

    // This is a special case and will usually not occur with the implementations provided
    // by nova because the generics will prevent this
    // H
    #[fail(display = "This Surface can not be used for creating this object")]
    NotSupported,
}
