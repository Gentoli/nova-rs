use crate::rhi::*;

use ash::vk;

pub struct VulkanPipeline {
    pub vk_pipeline: vk::Pipeline,
}

impl Pipeline for VulkanPipeline {}
