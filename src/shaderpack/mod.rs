//! Shaderpack loaders.

use cgmath::Vector2;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// A fully parsed Nova Shaderpack
#[derive(Debug, Clone)]
pub struct ShaderpackData {
    /// Pipelines described by the shaderpack.
    pipelines: Vec<PipelineCreationInfo>,

    /// All the renderpasses that this shaderpack specifies, in submission order.
    passes: Vec<RenderPassCreationInfo>,

    /// All materials needed by this shaderpack.
    materials: Vec<MaterialData>,

    /// Resources required by shaderpack.
    resources: ShaderpackResourceData,
}

/// Information needed to create a pipeline
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PipelineCreationInfo {
    /// The name of this pipeline.
    name: String,

    /// The pipeline that this pipeline inherits from.
    parent: Option<String>,

    /// The name of the pass that this pipeline belongs to.
    pass: String,

    /// All of the symbols in the shader that are defined by this state.
    defines: Vec<String>,

    /// Defines the rasterizer state that's active for this pipeline.
    states: Vec<RasterizerState>,

    /// Sets up the vertex fields that Nova will bind to this pipeline.
    vertex_fields: Vec<VertexFieldData>,

    /// The stencil buffer operations to perform on the front faces.
    front_face: Option<StencilOpState>,

    /// The stencil buffer operations to perform on the back faces.
    back_face: Option<StencilOpState>,

    /// The material to use if this one's shaders can't be found.
    fallback: Option<String>,

    /// A bias to apply to the depth.
    #[serde(default = "PipelineCreationInfo::default_depth_bias")]
    depth_bias: f32,

    /// The depth bias, scaled by slope I guess?
    #[serde(default = "PipelineCreationInfo::default_slope_scaled_depth_bias")]
    slope_scaled_depth_bias: f32,

    /// The reference value to use for the stencil test.
    #[serde(default = "PipelineCreationInfo::default_stencil_ref")]
    stencil_ref: u32,

    /// The mask to use when reading from the stencil buffer.
    #[serde(default = "PipelineCreationInfo::default_stencil_read_mask")]
    stencil_read_mask: u32,

    /// The mask to use when writing to the stencil buffer.
    #[serde(default = "PipelineCreationInfo::default_stencil_write_mask")]
    stencil_write_mask: u32,

    /// How to handle MSAA for this state.
    #[serde(default = "PipelineCreationInfo::default_msaa_support")]
    msaa_support: MSAASupport,

    /// Decides how the vertices are rendered.
    #[serde(default = "PipelineCreationInfo::default_primitive_mode")]
    primitive_mode: PrimitiveTopology,

    /// Where to get the blending factor for the source.
    #[serde(default = "PipelineCreationInfo::default_src_blend_factor")]
    src_blend_factor: BlendFactor,

    /// Where to get the blending factor for the destination.
    #[serde(default = "PipelineCreationInfo::default_dst_blend_factor")]
    dst_blend_factor: BlendFactor,

    /// How to get the source alpha in a blend.
    #[serde(default = "PipelineCreationInfo::default_alpha_src")]
    alpha_src: BlendFactor,

    /// How to get the destination alpha in a blend.
    #[serde(rename = "alphaDest")]
    #[serde(default = "PipelineCreationInfo::default_alpha_dst")]
    alpha_dst: BlendFactor,

    /// The function to use for the depth test.
    #[serde(default = "PipelineCreationInfo::default_depth_func")]
    depth_func: CompareOp,

    /// The render queue that this pass belongs to.
    /// This may or may not be removed depending on what is actually needed by Nova.
    #[serde(default = "PipelineCreationInfo::default_render_queue")]
    render_queue: RenderQueue,

    /// Vertex shader to use.
    #[serde(default = "PipelineCreationInfo::default_vertex_shader")]
    vertex_shader: ShaderSource,

    /// Geometry shader to use.
    geometry_shader: Option<ShaderSource>,

    /// Tessellation Control shader to use.
    tessellation_control_shader: Option<ShaderSource>,

    /// Tessellation Evaluation shader to use.
    tessellation_evaluation_shader: Option<ShaderSource>,

    /// Fragment shader to use.
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

    /// Merge a shaderpack with a "parent" shaderpack. Unimplemented.
    ///
    /// # Parameters
    ///
    /// - `_other` - Shaderpack to merge with.
    pub fn merge_with_parent(&mut self, _other: &PipelineCreationInfo) -> Self {
        unimplemented!()
    }
}

/// A pass over the scene.
///
/// A pass has a few things:
/// - What passes MUST be executed before this one.
/// - What inputs this pass's shaders have:
///      - What uniform buffers to use.
///      - What vertex data to use.
///      - Any textures that are needed.
/// - What outputs this pass has:
///      - Framebuffer attachments.
///      - Write buffers.
///
/// The inputs and outputs of a pass must be resources declared in the shaderpack's `resources.json` file (or the
/// default resources.json), or a resource that's internal to Nova. For example, Nova provides a UBO of uniforms that
/// change per frame, a UBO for per-model data like the model matrix, and the virtual texture atlases. The default
/// resources.json file sets up sixteen framebuffer color attachments for ping-pong buffers, a depth attachment,
/// some shadow maps, etc.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPassCreationInfo {
    /// The name of this render pass.
    #[serde(default = "RenderPassCreationInfo::default_name")]
    name: String,

    /// The materials that MUST execute before this one.
    dependencies: Vec<String>,

    /// The textures that this pass will read from.
    texture_inputs: Vec<String>,

    /// The textures that this pass will write to.
    texture_outputs: Vec<TextureAttachmentInfo>,

    /// The depth texture this pass will write to.
    depth_texture: Option<TextureAttachmentInfo>,

    /// All the buffers that this renderpass reads from.
    input_buffers: Vec<String>,

    /// All the buffers that this renderpass writes to.
    output_buffers: Vec<String>,
}

impl RenderPassCreationInfo {
    fn default_name() -> String {
        String::from("<NAME_MISSING>")
    }
}

/// A single renderable material.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialData {
    /// The name of the material.
    name: String,

    /// The information needed for each renderpass the material is in.
    passes: Vec<MaterialPass>,

    /// Name of the geometry filter to use.
    geometry_filter: String,
}

/// Holds all resources that are required by the shaderpack.
#[derive(Debug, Clone, Deserialize)]
pub struct ShaderpackResourceData {
    /// Specification for needed textures.
    textures: Vec<TextureCreateInfo>,

    /// Specification for needed samplers.
    samplers: Vec<SamplerCreateInfo>,
}

/// Connects a [`VertexField`] with a semantic name.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VertexFieldData {
    /// Name of the vertex field.
    semantic_name: String,

    /// Type of vertex data.
    field: VertexField,
}

/// State of all the stencil operations.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StencilOpState {
    /// Operation if stencil test fails.
    #[serde(default = "StencilOpState::default_fail_op")]
    fail_op: StencilOp,

    /// Operation if stencil test passes.
    #[serde(default = "StencilOpState::default_pass_op")]
    pass_op: StencilOp,

    /// Operation if depth test fails.
    #[serde(default = "StencilOpState::default_depth_fail_op")]
    depth_fail_op: StencilOp,

    /// Comparison with the stencil buffer.
    #[serde(default = "StencilOpState::default_compare_op")]
    compare_op: CompareOp,

    /// Stencil buffer comparison mask.
    #[serde(default = "StencilOpState::default_compare_mask")]
    compare_mask: u32,

    /// Stencil buffer write mask.
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

/// Shader source file as a compiled SPIR-V file.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShaderSource {
    /// Filename of the shader source file.
    filename: PathBuf,

    /// Compiled SPIR-V shader.
    #[serde(skip)]
    source: Vec<u32>,
}

/// A description of a texture that a render pass outputs to.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureAttachmentInfo {
    ///  The name of the texture.
    name: String,

    /// Pixel format of the texture.
    #[serde(default = "TextureAttachmentInfo::default_pixel_format")]
    pixel_format: PixelFormat,

    /// Whether to clear the texture.
    ///
    /// If the texture is a depth buffer, it gets cleared to 1.
    /// If the texture is a stencil buffer, it gets cleared to 0xFFFFFFFF.
    /// If the texture is a color buffer, it gets cleared to (0, 0, 0, 0).
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

/// The per-renderpass data for a material
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaterialPass {
    /// Name of the render pass.
    name: String,

    /// Name of the material itself.
    ///
    /// This is not populated by serde, this is populated by a post processing pass _after_ serde.
    ///
    /// TODO(cwfitzgerald): Which function does that?
    #[serde(default)]
    material_name: String,

    /// Name of the pipeline.
    pipeline: String,

    /// All named bindings for this renderpass.
    bindings: HashMap<String, String>,
}

/// Description of a texture
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureCreateInfo {
    /// The name of the texture.
    ///
    /// Nova implicitly defines a few textures for you to use:
    /// - `ColorVirtualTexture`:
    ///      - Virtual texture atlas that holds color textures.
    ///      - Textures which have the exact name as requested by Minecraft are in this atlas.
    ///      - Things without a color texture get a pure white texture.
    ///      - Always has a format of `RGBA8`.
    ///      - Can only be used as a pass's input.
    /// - `NormalVirtualTexture`:
    ///      - Virtual texture atlas that holds normal textures.
    ///      - Textures which have `_n` after the name requested by Minecraft are in this atlas.
    ///      - If no normal texture exists for a given object, a texture with RGBA of (0, 0, 1, 1) is used.
    ///      - Always has a format of `RGBA8`.
    ///      - Can only be used as a pass's input.
    /// - `DataVirtualTexture`:
    ///      - Virtual texture atlas that holds data textures.
    ///      - Textures which have a `_s` after the name requested by Minecraft are in this atlas.
    ///      - If no data texture exists for a given object, a texture with an RGBA of (0, 0, 0, 0) is used.
    ///      - Always has a format of `RGBA8`.
    ///      - Can only be used as a pass's input.
    /// - `Lightmap`:
    ///      - Lightmap, loaded from the current resourcepack.
    ///      - Format of RGB8.
    ///      - Can only be used as an input.
    /// - `Backbuffer`:
    ///      - The texture that gets presented to the screen.
    ///      - Always has a format of RGB8.
    ///      - Can only be used as a pass's output.
    ///
    /// If you use one of the virtual textures, then all fields except the binding are ignored.
    ///
    /// If you use `Backbuffer`, then all fields are ignored since the backbuffer is always bound to output location 0.
    ///
    /// TODO(cwfitzgerald): This can have a more elegant representation with an enum
    name: String,

    /// Texture format for the image.
    ///
    /// All members except the bindings are ignored if the texture is virtual. Everything is
    /// ignored if the texture is the BackBuffer.
    format: TextureFormat,
}

/// Defines a sampler to use for a texture.
///
/// At the time of writing I'm not sure how this is correlated with a texture, but all well.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplerCreateInfo {
    /// String name of the sampler.
    #[serde(default = "SamplerCreateInfo::default_name")]
    name: String,

    /// What kind of texture filter to use.
    ///
    /// texel_aa does something that I don't want to figure out right now. Bilinear is your regular bilinear filter,
    /// and point is the point filter. Aniso isn't an option and I kinda hope it stays that way.
    #[serde(default = "SamplerCreateInfo::default_filter")]
    filter: TextureFilter,

    /// How the texture should wrap at the edges.
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

/// The formatting information of a texture in memory.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureFormat {
    /// The format of the texture.
    #[serde(default = "TextureFormat::default_pixel_format")]
    pixel_format: PixelFormat,

    /// How to interpret the dimensions of this texture.
    #[serde(default = "TextureFormat::default_dimension_type")]
    dimension_type: TextureDimensionType,

    /// The width, in pixels, of the texture.
    #[serde(default = "TextureFormat::default_width")]
    width: f32,

    /// The height, in pixels, of the texture.
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

    /// Returns the screen size in pixels.
    ///
    /// # Parameters
    ///
    /// - `screen_size` - Needed if the texture resolution is relative to the screen size
    pub fn get_size_in_pixels(&self, screen_size: Vector2<f32>) -> Vector2<f32> {
        let (width, height) = match self.dimension_type {
            TextureDimensionType::ScreenRelative => (self.width * screen_size.x, self.height * screen_size.y),
            TextureDimensionType::Absolute => (self.width, self.height),
        };

        Vector2::new(width.round(), height.round())
    }
}

/// Rasterizer hardware features that shaderpacks can enable.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RasterizerState {
    /// Enable blending for this material state.
    Blending,

    /// Render backfaces and cull frontfaces.
    InvertCulling,

    /// Don't cull backfaces or frontfaces.
    DisableCulling,

    /// Don't write to the depth buffer.
    DisableDepthWrite,

    /// Don't perform a depth test.
    DisableDepthTest,

    /// Perform the stencil test.
    EnableStencilTest,

    /// Write to the stencil buffer.
    StencilWrite,

    /// Don't write to the color buffer.
    DisableColorWrite,

    /// Enable alpha to coverage.
    EnableAlphaToCoverage,

    /// Don't write alpha.
    DisableAlphaWrite,
}

/// Multisample Antialiasing mode.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum MSAASupport {
    /// Enable MSAA.
    MSAA,

    /// Disable antialiasing.
    None,
}

/// Primitive to interpret vertex buffer as.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum PrimitiveTopology {
    /// Rasterize triangles.
    Triangles,

    /// Rasterize lines.
    Lines,
}

/// How to blend the new image with the old image.
///
/// See [opengl wiki](https://www.khronos.org/opengl/wiki/Blending#Blend_Equations) for more info.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum BlendFactor {
    /// 1 * color
    One,

    /// 0 * color
    Zero,

    /// Pull from source color.
    SrcColor,

    /// Pull from destination color.
    DstColor,

    /// 1 - src
    OneMinusSrcColor,

    /// 1 - dst
    OneMinusDstColor,

    /// Pull from source alpha.
    SrcAlpha,

    /// Pull from destination alpha.
    DstAlpha,

    /// 1 - srcA
    OneMinusSrcAlpha,

    /// 1 - dstA
    OneMinusDstAlpha,
}

/// Comparator used for fixed function operations.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum CompareOp {
    /// false
    Never,

    /// a < b
    Less,

    /// a <= b
    LessEqual,

    /// a > b
    Greater,

    /// a >= b
    GreaterEqual,

    /// a == b
    Equal,

    /// a != b
    NotEqual,

    /// true
    Always,
}

/// Objects join a queue based on the type of transparency they need.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum RenderQueue {
    /// Full alpha transparency.
    Transparent,

    /// No transparency.
    Opaque,

    /// Cutout transparency (full transparent or opaque).
    Cutout,
}

/// Identifier for a type and data format for vertex data.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum VertexField {
    /// The vertex position.
    ///
    /// 12 bytes
    Position,

    /// The vertex color.
    ///
    /// 4 bytes.
    Color,

    /// The UV coordinate of this object.
    ///
    /// Except not really, because Nova's virtual textures means that the UVs for a block or entity or whatever
    /// could change on the fly, so this is kinda more of a preprocessor define that replaces the UV with a lookup
    /// in the UV table.
    ///
    /// 8 bytes (might try 4).
    UV0,

    /// The UV coordinate in the lightmap texture.
    ///
    /// This is a real UV and it doesn't change for no good reason.
    ///
    /// 2 bytes.
    UV1,

    /// Vertex normal.
    ///
    /// 12 bytes.
    Normal,

    /// Vertex tangents.
    ///
    /// 12 bytes.
    Tangent,

    /// The texture coordinate of the middle of the quad.
    ///
    /// 8 bytes
    MidTexCoord,

    /// A uint32_t that's a unique identifier for the texture that this vertex uses.
    ///
    /// This is generated at runtime by Nova, so it may change a lot depending on what resourcepacks are loaded and
    /// if they use CTM or random detail textures or whatever.
    ///
    /// 4 bytes.
    VirtualTextureId,

    /// Some information about the current block/entity/whatever.
    ///
    /// 12 bytes
    McEntityId,
}

/// Which operation to determine the value of the stencil buffer after a write.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum StencilOp {
    /// Do not change the stencil buffer.
    Keep,

    /// Set the stencil buffer to zero.
    Zero,

    /// Replace the stencil buffer with the current replacement value.
    Replace,

    /// Increments the stencil buffer value.
    Incr,

    /// Increments the stencil buffer value, wrapping around to zero on overflow.
    IncrWrap,

    /// Decrements the stencil buffer value.
    Decr,

    /// Decrements the stencil buffer value, wrapping around to 2^n-1 on underflow.
    DecrWrap,

    /// Bitwise invert the current stencil value.
    Invert,
}

/// Layout of pixels in memory
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum PixelFormat {
    /// R, G, B, and A channels, all taking up 8 bits integers each. 4 bytes.
    RGBA8,

    /// R, G, B, and A channels, all taking up 16 bits floats each. 8 bytes.
    RGBA16F,

    /// R, G, B, and A channels, all taking up 32 bits floats each. 16 bytes.
    RGBA32F,

    /// Depth channel only.
    Depth,

    /// Depth and stencil channel.
    DepthStencil,
}

/// Filter to use when reading from texture.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TextureFilter {
    /// Bedrock features texel manipulation based AA.
    TexelAA,

    /// Bilinear filtering.
    Bilinear,

    /// Normal filtering.
    Point,
}

/// Texture wrap mode.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum WrapMode {
    /// Repeat the texture when out of UV bounds.
    Repeat,

    /// Clamp to the edge of the UV when out of UV bounds.
    Clamp,
}

/// Frame of reference for texture dimensions.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TextureDimensionType {
    /// Dimensions are relative to the screen to allow screen space textures of the appropriate size.
    ScreenRelative,

    /// Dimensions are absolute.
    Absolute,
}

/// Origin location of a texture
#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TextureLocation {
    /// The texture is written to by a shader.
    Dynamic,

    /// The texture is loaded from the textures/ folder in the current shaderpack.
    InUserPackage,

    /// The texture is provided by Nova or by Minecraft.
    InAppPackage,
}
