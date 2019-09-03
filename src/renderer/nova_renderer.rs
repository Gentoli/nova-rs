use crate::renderer::Renderer;
use crate::rhi;
use crate::settings::Settings;
use crate::shaderpack;

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
pub struct PlatformRenderer<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    device: GraphicsApi::Device,

    /// Flag for if we can render frames. If this is false then no frames get rendered, aka execute frame is a no-op
    can_render: bool,
}

impl<GraphicsApi> PlatformRenderer<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    /// Creates a new renderer
    pub fn new(settings: Settings) -> Result<Self, PlatformRendererCreationError> {
        let graphics_api = GraphicsApi::new(settings);

        let mut adapters = graphics_api.get_adapters();

        match adapters.len() {
            0 => Err(PlatformRendererCreationError::ApiNotSupported),
            _ => Ok(PlatformRenderer {
                device: adapters.remove(0),
                can_render: true,
            }),
        }
    }

    /// Sets this renderer's current render graph as the new render graph
    ///
    /// This method will stall for a little bit as we wait for the GPU to finish all its current frames, then
    pub fn set_render_graph(&mut self, graph: shaderpack::ShaderpackData) {}
}
