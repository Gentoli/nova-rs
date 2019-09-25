//! The actual rendering code for Nova

use crate::mesh::{MeshData, MeshId};
use crate::shaderpack::ShaderpackData;

pub mod api_renderer;
mod rendergraph;

/// Interface for rendering things
///
/// Implementors of this trait are implementing a full renderer. It should render all visible objects every frame, in
/// the manner specified by the shaderpack, and it should do so as fast as possible while presenting data to the user
/// at the highest possible quality.
pub trait Renderer {
    /// Sets this renderer's current render graph as the new render graph
    ///
    /// This method will schedule a task to wait until all in-flight frames are finished processing. Then, it will
    /// delete the old render graph and create a new one from the provided shaderpack data. Finally, it will allow
    /// rendering to continue with the new render graph
    fn set_render_graph(&mut self, graph: ShaderpackData);

    /// Adds a mesh to Nova
    ///
    /// Meshes get uploaded asynchronously, so they may or may not appear a few frames after you initially upload then.
    /// The IDs, however, are available immediately
    fn add_mesh(&mut self, mesh_data: MeshData) -> MeshId;

    /// Ticks the renderer for a single frame, telling the renderer to act like the frame took delta_time seconds to
    /// execute
    ///
    /// This method performs any cleanup/housekeeping tasks before executing the render graph. For example, it deletes
    /// any meshes which are not in-use before executing the render graph.
    fn tick(&self, delta_time: f32);
}
