use crate::mesh::StaticMeshDrawCommand;
use crate::rhi::ResourceBarrier;
use crate::{rhi, shaderpack};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Display};

/// All the runtime data needed to execute a single renderpass
pub struct Renderpass<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    /// RHI renderpass object
    pub renderpass: GraphicsApi::Renderpass,

    /// RHI framebuffer to render to
    pub framebuffer: GraphicsApi::Framebuffer,

    /// Pipelines which will be drawn by this renderpass
    pub pipelines: Vec<Pipeline<GraphicsApi>>,

    /// Whether or not this renderpass will write to the backbuffer
    pub writes_to_backbuffer: bool,

    /// Barriers to get this renderpass's read-only image resources into a state needed by this renderpass
    ///
    /// Probably most useful for transitioning images into shader read optimal
    pub read_texture_barriers: Vec<(GraphicsApi::Image, ResourceBarrier)>,

    /// Barriers to get this renderpass's write-only resources into a state needed by this renderpass
    ///
    /// Probably most useful for any image that a shader writes to with image load store
    pub write_texture_barriers: Vec<(GraphicsApi::Image, ResourceBarrier)>,
}

impl<GraphicsApi> Default for Renderpass<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    fn default() -> Self {
        Renderpass {
            renderpass: (),
            framebuffer: (),
            pipelines: vec![],
            writes_to_backbuffer: false,
            read_texture_barriers: vec![],
            write_texture_barriers: vec![],
        }
    }
}

/// All the data needed to issue all drawcalls that use a specific pipeline
pub struct Pipeline<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    /// RHI object for the actual pipeline to use
    pub pipeline: GraphicsApi::Pipeline,

    /// All the material passes that use this pipeline
    pub passes: Vec<MaterialPass<GraphicsApi>>,
}

/// A single pass from a material
#[derive(Default)]
pub struct MaterialPass<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    /// All the static mesh draws that use this material
    pub static_mesh_draws: Vec<MeshBatch<GraphicsApi, StaticMeshDrawCommand>>,

    /// The material's descriptor sets
    pub descriptor_sets: Vec<GraphicsApi::DescriptorSet>,

    /// The interface for the pipeline that this material pass uses
    pub pipeline_interface: GraphicsApi::PipelineInterface,
}

/// A match of mesh calls
///
/// Equivalent to one drawcall
pub struct MeshBatch<GraphicsApi, DrawCommandType>
where
    GraphicsApi: rhi::GraphicsApi,
{
    /// Vertex buffer that this mesh batch uses
    pub vertex_buffer: GraphicsApi::Buffer,

    /// Index buffer for this mesh batch
    pub index_buffer: GraphicsApi::Buffer,

    /// Buffer of data that's unique for each object in the batch
    pub per_renderable_data: GraphicsApi::Buffer,

    /// All the actual draw commands which generated this mesh batch
    /// TODO: Is this needed?
    pub renderables: Vec<DrawCommandType>,
}

/// A key to where a material pass is in a compiled rendergraph
#[derive(Default, Debug)]
pub struct MaterialPassKey {
    /// Index of this material's renderpass
    pub renderpass_index: u32,

    /// Index of this material's pipeline    
    pub pipeline_index: u32,

    /// Index of this material
    pub material_pass_index: u32,
}

/// Metadata about a pipeline
#[derive(Default, Debug)]
pub struct PipelineMetadata {
    /// The data that this pipeline was created with
    pub data: shaderpack::PipelineCreationInfo,

    /// Metadata about all the material passes in this pipeline
    pub material_metadatas: HashMap<FullMaterialPassName, MaterialPassMetadata>,
}

/// The full name of a material pass, mostly useful for logging
#[derive(Default, Debug, Eq, Hash)]
pub struct FullMaterialPassName {
    /// Name of the material that this material pass belongs to
    pub material_name: String,

    pub pass_name: String,
}

impl PartialEq for FullMaterialPassName {
    fn eq(&self, other: &Self) -> bool {
        self.material_name.eq(&other.material_name) && self.pass_name.eq(&other.material_name)
    }
}

/// Metadata about a material pass
#[derive(Default, Debug)]
pub struct MaterialPassMetadata {
    /// The data that the material pass was created with
    pub data: shaderpack::MaterialPass,
}

#[derive(Default, Debug)]
pub struct RenderpassMetadata {
    pub data: shaderpack::RenderPassCreationInfo,

    pub pipeline_metadata: HashMap<String, PipelineMetadata>,
}
