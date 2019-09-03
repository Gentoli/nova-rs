use crate::rhi;

struct Renderpass<GraphicsApi>
where
    GraphicsApi: rhi::GraphicsApi,
{
    renderpass: GraphicsApi::Renderpass,

    framebuffer: GraphicsApi::Framebuffer,
}
