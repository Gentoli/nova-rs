#![allow(unsafe_code)]

use crate::rhi::*;

use crate::rhi::vulkan::vulkan_buffer::VulkanBuffer;
use crate::rhi::vulkan::vulkan_device::VulkanDeviceQueueFamilies;
use crate::rhi::vulkan::vulkan_image::VulkanImage;
use ash;
use ash::version::DeviceV1_0;
use ash::vk;
use ash::vk::DependencyFlags;

pub struct VulkanCommandList {
    instance: ash::Instance,
    device: ash::Device,
    buffer: vk::CommandBuffer,

    queue_families: VulkanDeviceQueueFamilies,
}

impl VulkanCommandList {
    pub fn new(
        instance: ash::Instance,
        device: ash::Device,
        buffer: vk::CommandBuffer,
        queue_families: VulkanDeviceQueueFamilies,
    ) -> VulkanCommandList {
        VulkanCommandList {
            instance,
            device,
            buffer,
            queue_families,
        }
    }

    #[inline]
    fn nova_stage_flags_to_vulkan_flags(stage_flags: PipelineStageFlags) -> vk::PipelineStageFlags {
        // Currently nova's stage flags match their vulkan counterparts
        stage_flags as vk::PipelineStageFlags
    }

    #[inline]
    fn nova_access_flags_to_vulkan_flags(access_flags: ResourceAccessFlags) -> vk::AccessFlags {
        // Currently nova's access flags match their vulkan counterparts
        access_flags as vk::AccessFlags
    }

    fn nova_resource_state_to_vulkan_layout(resource_state: ResourceState) -> vk::ImageLayout {
        match resource_state {
            ResourceState::Undefined => vk::ImageLayout::UNDEFINED,
            ResourceState::General => vk::ImageLayout::GENERAL,
            ResourceState::ColorAttachment => vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ResourceState::DepthStencilAttachment => vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ResourceState::DepthReadOnlyStencilAttachment => {
                vk::ImageLayout::DEPTH_READ_ONLY_STENCIL_ATTACHMENT_OPTIMAL
            }
            ResourceState::DepthAttachmentStencilReadOnly => {
                vk::ImageLayout::DEPTH_ATTACHMENT_STENCIL_READ_ONLY_OPTIMAL
            }
            ResourceState::DepthStencilReadOnlyAttachment => vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL,
            ResourceState::PresentSource => vk::ImageLayout::PRESENT_SRC_KHR,
            ResourceState::NonFragmentShaderReadOnly | ResourceState::FragmentShaderReadOnly => {
                vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
            }
            ResourceState::TransferSource => vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            ResourceState::TransferDestination => vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        }
    }

    fn get_underlying_buffer(&self) -> vk::CommandBuffer {
        self.buffer
    }
}

impl CommandList for VulkanCommandList {
    type Buffer = VulkanBuffer;
    type CommandList = VulkanCommandList;
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
        let mut image_barriers = Vec::new();
        let mut buffer_barriers = Vec::new();

        for barrier in barriers {
            match barrier.resource_info {
                ResourceSpecificData::Image { .. } => {
                    let image: VulkanImage = barrier.resource.downcast::<VulkanImage>().unwrap();

                    let vk_barrier = vk::ImageMemoryBarrier::builder()
                        .src_access_mask(self.nova_access_flags_to_vulkan_flags(barrier.access_before_barrier))
                        .dst_access_mask(self.nova_access_flags_to_vulkan_flags(barrier.access_after_barrier))
                        .old_layout(self.nova_resource_state_to_vulkan_layout(barrier.initial_state))
                        .new_layout(self.nova_resource_state_to_vulkan_layout(barrier.final_state))
                        .src_queue_family_index(self.queue_families.get(barrier.source_queue))
                        .dst_queue_family_index(self.queue_families.get(barrier.source_queue))
                        .image(image.vk_image)
                        .subresource_range(
                            vk::ImageSubresourceRange::builder()
                            .base_mip_level(0) // TODO: also TODO in original nova
                                               //       "Something smarter with mips"
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1)
                            .build(),
                        )
                        .build();
                    image_barriers.push(vk_barrier);
                }
                ResourceSpecificData::Buffer { offset, size } => {
                    let buffer: VulkanBuffer = barrier.resource.downcast::<VulkanBuffer>().unwrap();

                    let vk_barrier = vk::BufferMemoryBarrier::builder()
                        .src_access_mask(self.nova_access_flags_to_vulkan_flags(barrier.access_before_barrier))
                        .dst_access_mask(self.nova_access_flags_to_vulkan_flags(barrier.access_after_barrier))
                        .src_queue_family_index(self.queue_families.get(barrier.source_queue))
                        .dst_queue_family_index(self.queue_families.get(barrier.source_queue))
                        .buffer(buffer.vk_buffer)
                        .offset(offset)
                        .size(size)
                        .build();

                    buffer_barriers.push(vk_barrier);
                }
            }
        }

        unsafe {
            self.device.cmd_pipeline_barrier(
                self.buffer,
                self.nova_stage_flags_to_vulkan_flags(stages_before_barrier),
                self.nova_stage_flags_to_vulkan_flags(stages_after_barrier),
                0 as DependencyFlags,
                &[],
                buffer_barriers.as_slice(),
                image_barriers.as_slice(),
            )
        }
    }

    fn copy_buffer(
        &self,
        destination_buffer: Self::Buffer,
        destination_offset: u64,
        source_buffer: Self::Buffer,
        source_offset: u64,
        num_bytes: u64,
    ) {
        let buffer_copy = vk::BufferCopy::builder()
            .src_offset(source_offset)
            .dst_offset(destination_offset)
            .size(num_bytes)
            .build();

        unsafe {
            self.device.cmd_copy_buffer(
                self.buffer,
                source_buffer.vk_buffer,
                destination_buffer.vk_buffer,
                &[buffer_copy],
            )
        };
    }

    fn execute_command_lists(&self, lists: Vec<Self::CommandList>) {
        let mut buffers = Vec::new();
        for list in lists {
            buffers.push(list.get_underlying_buffer());
        }

        unsafe { self.device.cmd_execute_commands(self.buffer, buffers.as_slice()) }
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
