use crate::rhi::*;

use ash::vk;

pub struct VulkanPipelineInterface {
    pub vk_pipeline_layout: vk::PipelineLayout,
}

impl PipelineInterface for VulkanPipelineInterface {}
