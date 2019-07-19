//! Data and utilities for working with shaderpacks

use cgmath::Vector2;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ShaderpackData {
    pipelines: Vec<PipelineCreationInfo>,
    /// All the renderpasses that this shaderpack needs, in submission order
    passes: Vec<RenderPassCreationInfo>,
    materials: Vec<MaterialData>,
    resources: ShaderpackResourceData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    #[serde(default = "PipelineCreationInfo::default_depth_bias")]
    depth_bias: f32,
    /// The depth bias, scaled by slope I guess?
    #[serde(default = "PipelineCreationInfo::default_slope_scaled_depth_bias")]
    slope_scaled_depth_bias: f32,
    /// The reference value to use for the stencil test
    #[serde(default = "PipelineCreationInfo::default_stencil_ref")]
    stencil_ref: u32,
    /// The mask to use when reading from the stencil buffer
    #[serde(default = "PipelineCreationInfo::default_stencil_read_mask")]
    stencil_read_mask: u32,
    /// The mask to use when writing to the stencil buffer
    #[serde(default = "PipelineCreationInfo::default_stencil_write_mask")]
    stencil_write_mask: u32,
    /// How to handle MSAA for this state
    #[serde(default = "PipelineCreationInfo::default_msaa_support")]
    msaa_support: MSAASupport,
    /// Decides how the vertices are rendered
    #[serde(default = "PipelineCreationInfo::default_primitive_mode")]
    primitive_mode: PrimitiveTopology,
    /// Where to get the blending factor for the soource
    #[serde(default = "PipelineCreationInfo::default_src_blend_factor")]
    src_blend_factor: BlendFactor,
    /// Where to get the blending factor for the destination
    #[serde(default = "PipelineCreationInfo::default_dst_blend_factor")]
    dst_blend_factor: BlendFactor,
    /// How to get the source alpha in a blend
    #[serde(default = "PipelineCreationInfo::default_alpha_src")]
    alpha_src: BlendFactor,
    /// How to get the destination alpha in a blend
    #[serde(rename = "alphaDest")]
    #[serde(default = "PipelineCreationInfo::default_alpha_dst")]
    alpha_dst: BlendFactor,
    /// The function to use for the depth test
    #[serde(default = "PipelineCreationInfo::default_depth_func")]
    depth_func: CompareOp,
    /// The render queue that this pass belongs to
    /// This may or may not be removed depending on what is actually needed by Nova
    #[serde(default = "PipelineCreationInfo::default_render_queue")]
    render_queue: RenderQueue,
    /// Vertex shader to use
    #[serde(default = "PipelineCreationInfo::default_vertex_shader")]
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

impl PipelineCreationInfo {
    fn default_depth_bias() -> f32 {
        0.0
    }
    fn default_slope_scaled_depth_bias() -> f32 {
        0.0
    }
    fn default_stencil_ref() -> u32 {
        0
    }
    fn default_stencil_read_mask() -> u32 {
        0
    }
    fn default_stencil_write_mask() -> u32 {
        0
    }
    fn default_msaa_support() -> MSAASupport {
        MSAASupport::None
    }
    fn default_primitive_mode() -> PrimitiveTopology {
        PrimitiveTopology::Triangles
    }
    fn default_src_blend_factor() -> BlendFactor {
        BlendFactor::One
    }
    fn default_dst_blend_factor() -> BlendFactor {
        BlendFactor::Zero
    }
    fn default_alpha_src() -> BlendFactor {
        BlendFactor::One
    }
    fn default_alpha_dst() -> BlendFactor {
        BlendFactor::Zero
    }
    fn default_depth_func() -> CompareOp {
        CompareOp::Less
    }
    fn default_render_queue() -> RenderQueue {
        RenderQueue::Opaque
    }
    fn default_vertex_shader() -> ShaderSource {
        ShaderSource {
            filename: PathBuf::from("<NAME_MISSING>"),
            source: Vec::new(),
        }
    }

    pub fn merge_with_parent(&self, _other: &PipelineCreationInfo) -> Self {
        unimplemented!()
    }
}

/// A pass over the scene
///
/// A pass has a few things:
/// - What passes MUST be executed before this one
/// - What inputs this pass's shaders have
///      - What uniform buffers to use
///      - What vertex data to use
///      - Any textures that are needed
/// - What outputs this pass has
///      - Framebuffer attachments
///      - Write buffers
///
/// The inputs and outputs of a pass must be resources declared in the shaderpack's `resources.json` file (or the
/// default resources.json), or a resource that's internal to Nova. For example, Nova provides a UBO of uniforms that
/// change per frame, a UBO for per-model data like the model matrix, and the virtual texture atlases. The default
/// resources.json file sets up sixteen framebuffer color attachments for ping-pong buffers, a depth attachment,
/// some shadow maps, etc
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPassCreationInfo {
    /// The name of this render pass
    #[serde(default = "RenderPassCreationInfo::default_name")]
    name: String,
    /// The materials that MUST execute before this one
    dependencies: Vec<String>,
    /// The textures that this pass will read from
    texture_inputs: Vec<String>,
    /// The textures that this pass will write to
    texture_outputs: Vec<TextureAttachmentInfo>,
    /// The depth texture this pass will write to
    depth_texture: Option<TextureAttachmentInfo>,
    /// All the buffers that this renderpass reads from
    input_buffers: Vec<String>,
    /// All the buffers that this renderpass writes to
    output_buffers: Vec<String>,
}

impl RenderPassCreationInfo {
    fn default_name() -> String {
        String::from("<NAME_MISSING>")
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialData {
    name: String,
    passes: Vec<MaterialPass>,
    geometry_filter: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ShaderpackResourceData {
    textures: Vec<TextureCreateInfo>,
    samplers: Vec<SamplerCreateInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VertexFieldData {
    semantic_name: String,
    field: VertexField,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StencilOpState {
    #[serde(default = "StencilOpState::default_fail_op")]
    fail_op: StencilOp,
    #[serde(default = "StencilOpState::default_pass_op")]
    pass_op: StencilOp,
    #[serde(default = "StencilOpState::default_depth_fail_op")]
    depth_fail_op: StencilOp,
    #[serde(default = "StencilOpState::default_compare_op")]
    compare_op: CompareOp,
    #[serde(default = "StencilOpState::default_compare_mask")]
    compare_mask: u32,
    #[serde(default = "StencilOpState::default_write_mask")]
    write_mask: u32,
}

impl StencilOpState {
    fn default_fail_op() -> StencilOp {
        StencilOp::Keep
    }
    fn default_pass_op() -> StencilOp {
        StencilOp::Keep
    }
    fn default_depth_fail_op() -> StencilOp {
        StencilOp::Keep
    }
    fn default_compare_op() -> CompareOp {
        CompareOp::Equal
    }
    fn default_compare_mask() -> u32 {
        0
    }
    fn default_write_mask() -> u32 {
        0
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShaderSource {
    filename: PathBuf,
    #[serde(skip)]
    source: Vec<u32>,
}

///  A description of a texture that a render pass outputs to
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureAttachmentInfo {
    ///  The name of the texture
    name: String,
    /// Pixel format of the texture
    #[serde(default = "TextureAttachmentInfo::default_pixel_format")]
    pixel_format: PixelFormat,
    ///  Whether to clear the texture
    ///
    /// If the texture is a depth buffer, it gets cleared to 1
    /// If the texture is a stencil buffer, it gets cleared to 0xFFFFFFFF
    /// If the texture is a color buffer, it gets cleared to (0, 0, 0, 0)
    #[serde(default = "TextureAttachmentInfo::default_clear")]
    clear: bool,
}

impl TextureAttachmentInfo {
    fn default_pixel_format() -> PixelFormat {
        PixelFormat::RGBA8
    }
    fn default_clear() -> bool {
        false
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialPass {
    name: String,
    // This is not populated until a post-processing pass _after_ deserialization and comes from
    // the parent struct MaterialData
    #[serde(default)]
    material_name: String,
    pipeline: String,
    bindings: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureCreateInfo {
    ///  The name of the texture
    ///
    /// Nova implicitly defines a few textures for you to use:
    /// - ColorVirtualTexture
    ///      - Virtual texture atlas that holds color textures
    ///      - Textures which have the exact name as requested by Minecraft are in this atlas
    ///      - Things without a color texture get a pure white texture
    ///      - Always has a format of R8G8B8A8
    ///      - Can only be used as a pass's input
    /// - NormalVirtualTexture
    ///      - Virtual texture atlas that holds normal textures
    ///      - Textures which have `_n` after the name requested by Minecraft are in this atlas
    ///      - If no normal texture exists for a given object, a texture with RGBA of (0, 0, 1, 1) is used
    ///      - Always has a format of R8G8B8A8
    ///      - Can only be used as a pass's input
    /// - DataVirtualTexture
    ///      - Virtual texture atlas that holds data textures
    ///      - Textures which have a `_s` after the name requested by Minecraft are in this atlas
    ///      - If no data texture exists for a given object, a texture with an RGBA of (0, 0, 0, 0) is used
    ///      - Always has a format of R8G8B8A8
    ///      - Can only be used as a pass's input
    /// - Lightmap
    ///      - Lightmap, loaded from the current resourcepack
    ///      - Format of RGB8
    ///      - Can only be used as an input
    /// - Backbuffer
    ///      - The texture that gets presented to the screen
    ///      - Always has a format of RGB8
    ///      - Can only be used as a pass's output
    ///
    /// If you use one of the virtual textures, then all fields except the binding are ignored
    /// If you use `Backbuffer`, then all fields are ignored since the backbuffer is always bound to output location 0
    name: String,
    format: TextureFormat,
}

///  Defines a sampler to use for a texture
///
/// At the time of writing I'm not sure how this is correlated with a texture, but all well
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplerCreateInfo {
    #[serde(default = "SamplerCreateInfo::default_name")]
    name: String,
    ///  What kind of texture filter to use
    ///
    /// texel_aa does something that I don't want to figure out right now. Bilinear is your regular bilinear filter,
    /// and point is the point filter. Aniso isn't an option and I kinda hope it stays that way
    #[serde(default = "SamplerCreateInfo::default_filter")]
    filter: TextureFilter,
    ///  How the texture should wrap at the edges
    #[serde(default = "SamplerCreateInfo::default_wrap_mode")]
    wrap_mode: WrapMode,
}

impl SamplerCreateInfo {
    fn default_name() -> String {
        String::new()
    }
    fn default_filter() -> TextureFilter {
        TextureFilter::Point
    }
    fn default_wrap_mode() -> WrapMode {
        WrapMode::Clamp
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureFormat {
    ///  The format of the texture
    #[serde(default = "TextureFormat::default_pixel_format")]
    pixel_format: PixelFormat,
    ///  How to interpret the dimensions of this texture
    #[serde(default = "TextureFormat::default_dimension_type")]
    dimension_type: TextureDimensionType,
    ///  The width, in pixels, of the texture
    #[serde(default = "TextureFormat::default_width")]
    width: f32,
    ///  The height, in pixels, of the texture
    #[serde(default = "TextureFormat::default_height")]
    height: f32,
}

impl TextureFormat {
    fn default_pixel_format() -> PixelFormat {
        PixelFormat::RGBA8
    }
    fn default_dimension_type() -> TextureDimensionType {
        TextureDimensionType::ScreenRelative
    }
    fn default_width() -> f32 {
        0.0
    }
    fn default_height() -> f32 {
        0.0
    }

    pub fn get_size_in_pixels(&self, screen_size: Vector2<f32>) -> Vector2<f32> {
        let (width, height) = match self.dimension_type {
            TextureDimensionType::ScreenRelative => (self.width * screen_size.x, self.height * screen_size.y),
            TextureDimensionType::Absolute => (self.width, self.height),
        };

        Vector2::new(width.round(), height.round())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RasterizerState {
    /// Enable blending for this material state
    Blending,
    /// Render backfaces and cull frontfaces
    InvertCulling,
    /// Don't cull backfaces or frontfaces
    DisableCulling,
    /// Don't write to the depth buffer
    DisableDepthWrite,
    /// Don't perform a depth test
    DisableDepthTest,
    /// Perform the stencil test
    EnableStencilTest,
    /// Write to the stencil buffer
    StencilWrite,
    /// Don't write to the color buffer
    DisableColorWrite,
    /// Enable alpha to coverage
    EnableAlphaToCoverage,
    /// Don't write alpha
    DisableAlphaWrite,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum MSAASupport {
    MSAA,
    Both,
    None,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum PrimitiveTopology {
    Triangles,
    Lines,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
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

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
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

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RenderQueue {
    Transparent,
    Opaque,
    Cutout,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum VertexField {
    ///  The vertex position
    ///
    /// 12 bytes
    Position,
    ///  The vertex color
    ///
    /// 4 bytes
    Color,
    ///  The UV coordinate of this object
    ///
    /// Except not really, because Nova's virtual textures means that the UVs for a block or entity or whatever
    /// could change on the fly, so this is kinda more of a preprocessor define that replaces the UV with a lookup
    /// in the UV table
    ///
    /// 8 bytes (might try 4)
    UV0,
    ///  The UV coordinate in the lightmap texture
    ///
    /// This is a real UV and it doesn't change for no good reason
    ///
    /// 2 bytes
    UV1,
    ///  Vertex normal
    ///
    /// 12 bytes
    Normal,
    ///  Vertex tangents
    ///
    /// 12 bytes
    Tangent,
    ///  The texture coordinate of the middle of the quad
    ///
    /// 8 bytes
    MidTexCoord,
    ///  A uint32_t that's a unique identifier for the texture that this vertex uses
    ///
    /// This is generated at runtime by Nova, so it may change a lot depending on what resourcepacks are loaded and
    /// if they use CTM or random detail textures or whatever
    ///
    /// 4 bytes
    VirtualTextureId,
    ///  Some information about the current block/entity/whatever
    ///
    /// 12 bytes
    McEntityId,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
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

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum PixelFormat {
    RGBA8,
    RGBA16F,
    RGBA32F,
    Depth,
    DepthStencil,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TextureFilter {
    TexelAA,
    Bilinear,
    Point,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum WrapMode {
    Repeat,
    Clamp,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TextureDimensionType {
    ScreenRelative,
    Absolute,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TextureLocation {
    ///  The texture is written to by a shader
    Dynamic,
    ///  The texture is loaded from the textures/ folder in the current shaderpack
    InUserPackage,
    ///  The texture is provided by Nova or by Minecraft
    InAppPackage,
}
