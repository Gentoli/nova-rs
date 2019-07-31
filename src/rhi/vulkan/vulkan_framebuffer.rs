use crate::rhi::*;

use ash::vk;

pub struct VulkanFramebuffer {
    pub vk_framebuffer: vk::Framebuffer,
    pub width: u32,
    pub height: u32,
}

impl Framebuffer for VulkanFramebuffer {}
