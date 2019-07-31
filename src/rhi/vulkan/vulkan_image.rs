use crate::rhi::*;

use ash::vk;

pub struct VulkanImage {
    pub vk_image: vk::Image,
}

impl Resource for VulkanImage {}
impl Image for VulkanImage {}
