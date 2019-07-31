use crate::rhi::Renderpass;

use ash::vk;

pub struct VulkanRenderPass {
    pub vk_renderpass: vk::RenderPass,
}

impl Renderpass for VulkanRenderPass {}
