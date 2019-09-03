use crate::mesh::MeshData;
use crate::renderer::Renderer;
use crate::rhi;
use crate::settings::Settings;
use crate::shaderpack::ShaderpackData;

pub fn new_dx12_renderer(settings: Settings) -> Box<dyn Renderer> {
    unimplemented!();
}

pub fn new_vulkan_renderer(settings: Settings) -> Box<dyn Renderer> {
    unimplemented!();
}

enum PlatformRendererCreationError {
    ApiNotSupported,
}

/// Actual renderer boi
///
/// A Renderer which is specialized for a graphics API
pub struct ApiRenderer<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    device: GraphicsApi::Device,

    /// Flag for if we can render frames. If this is false then no frames get rendered, aka execute frame is a no-op
    can_render: bool,

    // Render graph data
    has_rendergraph: bool,
}

impl<GraphicsApi> ApiRenderer<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    /// Creates a new renderer
    pub fn new(settings: Settings) -> Result<Self, PlatformRendererCreationError> {
        let graphics_api = GraphicsApi::new(settings);

        let mut adapters = graphics_api.get_adapters();

        match adapters.len() {
            0 => Err(PlatformRendererCreationError::ApiNotSupported),
            _ => Ok(ApiRenderer {
                device: adapters.remove(0),
                can_render: true,
                has_rendergraph: false,
            }),
        }
    }

    fn destroy_render_passes(&mut self) {}

    fn destroy_rendergraph_resources(&mut self) {}
}

impl<GraphicsApi> Renderer for ApiRenderer<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    fn set_render_graph(&mut self, graph: &ShaderpackData) {
        if self.has_rendergraph {
            self.destroy_render_passes();
            self.destroy_rendergraph_resources();
        }
    }

    fn add_mesh(&mut self, mesh_data: &MeshData) -> u32 {
        unimplemented!()
    }

    fn tick(&self, delta_time: f32) {
        unimplemented!()
    }
}
