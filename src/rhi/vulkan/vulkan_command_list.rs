use crate::rhi::*;

use ash;
use ash::vk;

pub struct VulkanCommandList {
    instance: ash::Instance,
    device: ash::Device,
    buffer: vk::CommandBuffer,
}

impl VulkanCommandList {
    pub fn new(instance: ash::Instance, device: ash::Device, buffer: vk::CommandBuffer) -> VulkanCommandList {
        VulkanCommandList {
            instance,
            device,
            buffer,
        }
    }
}

impl CommandList for VulkanCommandList {
    type Buffer = ();
    type CommandList = ();
    type Renderpass = ();
    type Framebuffer = ();
    type Pipeline = ();
    type DescriptorSet = ();
    type PipelineInterface = ();

    fn resource_barriers(
        &self,
        stages_before_barrier: PipelineStageFlags,
        stages_after_barrier: PipelineStageFlags,
        barriers: Vec<ResourceBarrier>,
    ) {
        unimplemented!()
    }

    fn copy_buffer(
        &self,
        destination_buffer: Self::Buffer,
        destination_offset: u64,
        source_buffer: Self::Buffer,
        source_offset: u64,
        num_bytes: u64,
    ) {
        unimplemented!()
    }

    fn execute_command_lists(&self, lists: Vec<Self::CommandList>) {
        unimplemented!()
    }

    fn begin_renderpass(&self, renderpass: Self::Renderpass, framebuffer: Self::Framebuffer) {
        unimplemented!()
    }

    fn end_renderpass(&self) {
        unimplemented!()
    }

    fn bind_pipeline(&self, pipeline: Self::Pipeline) {
        unimplemented!()
    }

    fn bind_descriptor_sets(
        &self,
        descriptor_sets: Vec<Self::DescriptorSet>,
        pipeline_interface: Self::PipelineInterface,
    ) {
        unimplemented!()
    }

    fn bind_vertex_buffers(&self, buffers: Vec<Self::Buffer>) {
        unimplemented!()
    }

    fn bind_index_buffer(&self, buffer: Self::Buffer) {
        unimplemented!()
    }

    fn draw_indexed_mesh(&self, num_indices: u32, num_instances: u32) {
        unimplemented!()
    }
}
