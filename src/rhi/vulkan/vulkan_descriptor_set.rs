use crate::rhi::*;

use ash::vk;

pub struct VulkanDescriptorSet {
    pub vk_descriptor_set: vk::DescriptorSet,
}

impl DescriptorSet for VulkanDescriptorSet {}
