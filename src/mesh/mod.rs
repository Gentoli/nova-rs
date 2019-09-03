use cgmath::{Matrix4, Vector2, Vector3, Vector4};

/// A single vertex in a mesh that Nova handles
///
/// Nova assumes that you'll have significantly less mesh data than texture data. This means that, for the sake of
/// simpler code, Nova uses the same data format for all its vertices
pub struct FullVertex {
    /// Model space position of the vertex
    pub position: Vector3<f32>,

    /// Tangent space normal
    pub normal: Vector3<f32>,

    /// Tangent space tangent
    pub tangent: Vector3<f32>,

    /// Texture space UV
    /// This is the UV of the object's actual texture, not it's UVs in the virtual texture atlas
    pub main_uv: Vector2<f32>,

    /// Secondary lightmap UV
    pub secondary_uv: Vector2<f32>,

    /// Unique ID of the texture that this object should be drawn with. Read back to the CPU to decide what to load
    /// into VRAM
    pub virtual_texture_id: u32,

    /// Additional data that may or may not be used
    pub additional_stuff: Vector4<f32>,
}

/// Data to upload a single mesh
///
/// This struct assumes that your mesh data is already split up per virtual texture. That's the responsibility of the
/// host application
pub struct MeshData {
    /// Vertices of the mesh
    pub vertex_data: Vec<FullVertex>,

    /// Indices of the mesh
    pub indices: Vec<u32>,
}

/// ID of a mesh that's been added to Nova
pub type MeshId = u32;

/// ID of a renderable object
pub type DrawCommandId = u32;

/// A command to render a single model
///
/// These commands don't reference the model they're rendering because the way they're stored means that we know what
/// model they're drawing
pub struct StaticMeshDrawCommand {
    /// ID of the draw command to render
    pub id: DrawCommandId,

    /// Whether or not the model drawn by this command is currently visible
    pub is_visible: bool,

    /// Model matrix that this draw command should use
    pub model_matrix: Matrix4<f32>,
}
