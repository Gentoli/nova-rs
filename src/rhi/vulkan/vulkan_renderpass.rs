use crate::rhi::Renderpass;

use ash::vk;

pub struct VulkanRenderPass {
    pub vk_renderpass: vk::RenderPass,
    pub render_area: vk::Rect2D,
}

impl Renderpass for VulkanRenderPass {}
