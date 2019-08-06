use crate::rhi::*;

use ash::vk;
use std::collections::HashMap;

pub struct VulkanPipelineInterface {
    pub vk_pipeline_layout: vk::PipelineLayout,
    pub vk_renderpass: vk::RenderPass,
    pub bindings: HashMap<String, ResourceBindingDescription>,
    pub layouts_by_set: Vec<vk::DescriptorSetLayout>,
}

impl PipelineInterface for VulkanPipelineInterface {}
