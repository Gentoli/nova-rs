//! Structs that represent shaderpack data

use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone)]
pub struct ShaderpackData {
    pipelines: Vec<PipelineCreationInfo>,
    passes: Vec<RenderPassCreationInfo>,
    materials: Vec<MaterialData>,
    resources: ShaderpackResourceData,
}

#[derive(Debug, Clone)]
pub struct PipelineCreationInfo {
    /// The name of this pipeline
    name: String,
    /// The pipeline that this pipeline inherits from
    parent: Option<String>,
    /// The name of the pass that this pipeline belongs to
    pass: String,
    /// All of the symbols in the shader that are defined by this state
    defines: Vec<String>,
    /// Defines the rasterizer state that's active for this pipeline
    states: Vec<RasterizerState>,
    /// Sets up the vertex fields that Nova will bind to this pipeline
    vertex_fields: Vec<VertexFieldData>,
    /// The stencil buffer operations to perform on the front faces
    front_face: Option<StencilOpState>,
    /// The stencil buffer operations to perform on the back faces
    back_face: Option<StencilOpState>,
    /// The material to use if this one's shaders can't be found
    fallback: Option<String>,
    /// A bias to apply to the depth
    depth_bias: f32,
    /// The depth bias, scaled by slope I guess?
    slope_scaled_depth_bias: f32,
    /// The reference value to use for the stencil test
    stencil_ref: u32,
    /// The mask to use when reading from the stencil buffer
    stencil_read_mask: u32,
    /// The mask to use when writing to the stencil buffer
    stencil_write_mask: u32,
    /// How to handle MSAA for this state
    msaa_support: MSAASupport,
    /// Decides how the vertices are rendered
    primitive_mode: PrimitiveTopology,
    /// Where to get the blending factor for the soource
    src_blend_factor: BlendFactor,
    /// Where to get the blending factor for the destination
    dst_blend_factor: BlendFactor,
    /// How to get the source alpha in a blend
    alpha_src: BlendFactor,
    /// How to get the destination alpha in a blend
    alpha_dst: BlendFactor,
    /// The function to use for the depth test
    depth_func: CompareOp,
    /// The render queue that this pass belongs to
    /// This may or may not be removed depending on what is actually needed by Nova
    render_queue: RenderQueue,
    /// Vertex shader to use
    vertex_shader: ShaderSource,
    /// Geometry shader to use
    geometry_shader: Option<ShaderSource>,
    /// Tessellation Control shader to use
    tessellation_control_shader: Option<ShaderSource>,
    /// Tessellation Evaluation shader to use
    tessellation_evaluation_shader: Option<ShaderSource>,
    /// Fragment shader to use
    fragment_shader: Option<ShaderSource>,
}

#[derive(Debug, Clone)]
pub struct RenderPassCreationInfo {
    name: String,
    dependencies: Vec<String>,
    texture_inputs: Vec<String>,
    texture_outputs: Vec<TextureAttachmentInfo>,
    depth_texture: Option<TextureAttachmentInfo>,
    input_buffers: Vec<String>,
    output_buffers: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MaterialData {
    name: String,
    passes: Vec<MaterialPass>,
    geometry_filter: String,
}

#[derive(Debug, Clone)]
pub struct ShaderpackResourceData {
    textures: Vec<TextureCreateInfo>,
    samplers: Vec<SamplerCreateInfo>,
}

#[derive(Debug, Clone)]
pub struct VertexFieldData {
    semantic_name: String,
    field: VertexField,
}

#[derive(Debug, Clone)]
pub struct StencilOpState {
    fail_op: StencilOp,
    pass_op: StencilOp,
    depth_fail_op: StencilOp,
    compare_op: StencilOp,
    compare_mask: u32,
    write_mask: u32,
}

#[derive(Debug, Clone)]
pub struct ShaderSource {
    filename: PathBuf,
    source: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureAttachmentInfo {
    name: String,
    pixel_format: PixelFormat,
    clear: bool,
}

#[derive(Debug, Clone)]
pub struct MaterialPass {
    name: String,
    material_name: String,
    pipeline: String,
    bindings: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct TextureCreateInfo {
    name: String,
    format: TextureFormat,
}

#[derive(Debug, Clone)]
pub struct SamplerCreateInfo {
    name: String,
    filter: TextureFilter,
    wrap_mode: WrapMode,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextureFormat {
    pixel_format: PixelFormat,
    dimension_type: TextureDimensionType,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RasterizerState {
    Blending,
    InvertCulling,
    DisableCulling,
    DisableDepthWrite,
    DisableDepthTest,
    EnableStencilTest,
    StencilWrite,
    DisableColorWrite,
    EnableAlphaToCoverage,
    DisableAlphaWrite,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MSAASupport {
    MSAA,
    Both,
    None,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PrimitiveTopology {
    Triangles,
    Lines,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BlendFactor {
    One,
    Zero,
    SrcColor,
    DstColor,
    OneMinusSrcColor,
    OneMinusDstColor,
    SrcAlpha,
    DstAlpha,
    OneMinusSrcAlpha,
    OneMinusDstAlpha,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CompareOp {
    Never,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
    Always,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RenderQueue {
    Transparent,
    Opaque,
    Cutout,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum VertexField {
    Position,
    Color,
    UV0,
    UV1,
    Normal,
    Tangent,
    MidTexCoord,
    VirtualTextureId,
    McEntityId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StencilOp {
    Keep,
    Zero,
    Replace,
    Incr,
    IncrWrap,
    Decr,
    DecrWrap,
    Invert,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PixelFormat {
    RGBA8,
    RGBA16F,
    RGBA32F,
    Depth,
    DepthStencil,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TextureFilter {
    TexelAA,
    Bilinear,
    Point,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WrapMode {
    Repeat,
    Clamp,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TextureDimensionType {
    ScreenRelative,
    Absolute,
}
