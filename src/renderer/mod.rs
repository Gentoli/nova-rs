//! The actual rendering code for Nova

pub mod nova_renderer;

/// Interface for rendering things
///
/// Implementors of this trait are implementing a full renderer. It should render all visible objects every frame, in
/// the manner specified by the shaderpack, and it should do so as fast as possible while presenting data to the user
/// at the highest possible quality.
pub trait Renderer {
    /// Ticks the renderer for a single frame, telling the renderer to act like the frame took delta_time seconds to
    /// execute
    ///
    /// This method should potentially do some housekeeping work,
    fn tick(&self, delta_time: f32);
}
