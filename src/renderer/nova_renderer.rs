use crate::rhi;
use crate::shaderpack;

/// Actual renderer boi
pub struct Renderer<ApiType> {}

impl<ApiType> Renderer<ApiType>
where
    ApiType: rhi::GraphicsApi,
{
    pub fn new() -> Self {
        let graphics_api = ApiType::new();

        let adapters = graphics_api.get_adapters();
    }

    pub fn set_render_graph(&self, graph: shaderpack::ShaderpackData) {}
}
