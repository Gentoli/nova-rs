use crate::rhi::*;

use ash::vk;

pub struct VulkanImage {
    pub vk_image: vk::Image,
    pub vk_image_view: vk::ImageView,
}

impl Resource for VulkanImage {}
impl Image for VulkanImage {}
