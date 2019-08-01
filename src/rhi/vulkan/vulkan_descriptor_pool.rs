use crate::rhi::vulkan::vulkan_descriptor_set::VulkanDescriptorSet;
use crate::rhi::vulkan::vulkan_pipeline_interface::VulkanPipelineInterface;
use crate::rhi::*;

use ash::vk;

pub struct VulkanDescriptorPool {
    vk_descriptor_pool: vk::DescriptorPool,
}

impl DescriptorPool for VulkanDescriptorPool {
    type PipelineInterface = VulkanPipelineInterface;
    type DescriptorSet = VulkanDescriptorSet;

    fn create_descriptor_sets(&self, pipeline_interface: Self::PipelineInterface) -> Vec<Self::DescriptorSet> {
        unimplemented!()
    }
}
