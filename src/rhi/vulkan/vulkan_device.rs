use crate::rhi::shaderpack::*;
use crate::rhi::vulkan::vulkan_memory::VulkanMemory;
use crate::rhi::vulkan::vulkan_queue::VulkanQueue;
use crate::rhi::*;

use ash::version::DeviceV1_0;
use ash::vk;
use cgmath::Vector2;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;

pub struct VulkanDevice {
    instance: ash::Instance,
    device: ash::Device,

    graphics_queue_family_index: u32,
    transfer_queue_family_index: u32,
    compute_queue_family_index: Option<u32>,

    memory_properties: vk::PhysicalDeviceMemoryProperties,
}

impl VulkanDevice {
    fn find_memory_by_flags(&self, memory_flags: vk::MemoryPropertyFlags, exact: bool) -> Option<u32> {
        self.memory_properties
            .memory_types
            .iter()
            .find(|t| {
                if exact {
                    t.property_flags == memory_flags
                } else {
                    t.property_flags & memory_flags != 0
                }
            })
            .map(|t| t.heap_index)
    }
}

impl Device for VulkanDevice {
    type Queue = VulkanQueue;
    type Memory = VulkanMemory;
    type CommandAllocator = ();
    type Image = ();
    type Renderpass = ();
    type Framebuffer = ();
    type PipelineInterface = ();
    type DescriptorPool = ();
    type Pipeline = ();
    type Semaphore = ();
    type Fence = ();

    fn get_queue(&self, queue_type: QueueType, queue_index: u32) -> Result<Self::Queue, QueueGettingError> {
        let queue_family_index = match queue_type {
            QueueType::Graphics => self.graphics_queue_family_index,
            QueueType::Copy => self.transfer_queue_family_index,
            QueueType::Compute => match self.compute_queue_family_index {
                None => Err(QueueGettingError::NotSupported),
                Some(i) => i,
            },
        };

        if queue_index > 0 {
            // We only support queue index 0 at the moment
            Err(QueueGettingError::IndexOutOfRange)
        } else {
            let queue = unsafe { self.device.get_device_queue(queue_family_index, queue_index) };
            Ok(VulkanQueue { queue })
        }
    }

    fn allocate_memory(
        &self,
        size: u64,
        memory_usage: MemoryUsage,
        allowed_objects: ObjectType,
    ) -> Result<Self::Memory, AllocationError> {
        let memory_type_index = match memory_usage {
            MemoryUsage::DeviceOnly => {
                let i = self.find_memory_by_flags(vk::MemoryPropertyFlags::DEVICE_LOCAL, true);
                if i.is_none() {
                    self.find_memory_by_flags(vk::MemoryPropertyFlags::DEVICE_LOCAL, false)
                } else {
                    i
                }
            }
            MemoryUsage::LowFrequencyUpload => {
                let i = self.find_memory_by_flags(
                    vk::MemoryPropertyFlags::DEVICE_LOCAL | vk::MemoryPropertyFlags::HOST_VISIBLE,
                    false,
                );
                if i.is_none() {
                    self.find_memory_by_flags(vk::MemoryPropertyFlags::HOST_CACHED, false)
                } else {
                    i
                }
            }
            MemoryUsage::StagingBuffer => self.find_memory_by_flags(
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_CACHED,
                false,
            ),
        };

        let memory_type_index = match memory_type_index {
            None => return Err(AllocationError::NoSuitableMemoryFound),
            Some(i) => i,
        };

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(size)
            .memory_type_index(memory_type_index)
            .build();

        let allocated = {
            let allocated = unsafe { self.device.allocate_memory(&alloc_info, None) };
            match allocated {
                Err(result) => {
                    return match result {
                        vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(AllocationError::OutOfHostMemory),
                        vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(AllocationError::OutOfDeviceMemory),
                        vk::Result::ERROR_TOO_MANY_OBJECTS => Err(AllocationError::TooManyObjects),
                        vk::Result::ERROR_INVALID_EXTERNAL_HANDLE => Err(AllocationError::InvalidExternalHandle),
                        result => unreachable!("Invalid vk result returned: {:?}", result),
                    };
                }
                Ok(v) => v,
            }
        };

        match memory_usage {
            MemoryUsage::LowFrequencyUpload | MemoryUsage::StagingBuffer => {
                let mapped = unsafe {
                    self.device
                        .map_memory(allocated, 0, vk::WHOLE_SIZE, 0u32 as vk::MemoryMapFlags)
                };

                match mapped {
                    Err(result) => {
                        return match result {
                            vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(AllocationError::OutOfDeviceMemory),
                            vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(AllocationError::OutOfHostMemory),
                            vk::Result::ERROR_MEMORY_MAP_FAILED => Err(AllocationError::MappingFailed),
                            result => unreachable!("Invalid vk result returned: {:?}", result),
                        };
                    }
                    Ok(mem) => Ok(VulkanMemory {
                        // TODO: Save allocated memory!
                        device: self.device.clone(),
                        memory: allocated,
                    }),
                }
            }
            _ => Ok(VulkanMemory {
                device: self.device.clone(),
                memory: allocated,
            }),
        }
    }

    fn create_command_allocator(
        &self,
        create_info: CommandAllocatorCreateInfo,
    ) -> Result<Self::CommandAllocator, MemoryError> {
        unimplemented!()
    }

    fn create_renderpass(&self, data: RenderPassCreationInfo) -> Result<Self::Renderpass, MemoryError> {
        unimplemented!()
    }

    fn create_framebuffer(
        &self,
        renderpass: Self::Renderpass,
        attachments: Vec<Self::Image>,
        framebuffer_size: Vector2<f32>,
    ) -> Result<Self::Framebuffer, MemoryError> {
        unimplemented!()
    }

    fn create_pipeline_interface(
        &self,
        bindings: &HashMap<String, ResourceBindingDescription, RandomState>,
        color_attachments: &Vec<TextureAttachmentInfo>,
        depth_texture: &Option<TextureAttachmentInfo>,
    ) -> Result<Self::PipelineInterface, MemoryError> {
        unimplemented!()
    }

    fn create_descriptor_pool(
        &self,
        num_sampled_images: u32,
        num_samplers: u32,
        num_uniform_buffers: u32,
    ) -> Result<Vec<Self::DescriptorPool>, DescriptorPoolCreationError> {
        unimplemented!()
    }

    fn create_pipeline(
        &self,
        pipeline_interface: Self::PipelineInterface,
        data: PipelineCreationInfo,
    ) -> Result<Self::Pipeline, PipelineCreationError> {
        unimplemented!()
    }

    fn create_image(&self, data: TextureCreateInfo) -> Result<Self::Image, MemoryError> {
        unimplemented!()
    }

    fn create_semaphore(&self) -> Result<Self::Semaphore, MemoryError> {
        unimplemented!()
    }

    fn create_semaphores(&self, count: u32) -> Result<Vec<Self::Semaphore>, MemoryError> {
        unimplemented!()
    }

    fn create_fence(&self) -> Result<Self::Fence, MemoryError> {
        unimplemented!()
    }

    fn create_fences(&self, count: u32) -> Result<Vec<Self::Fence>, MemoryError> {
        unimplemented!()
    }

    fn wait_for_fences(&self, fences: Vec<Self::Fence>) {
        unimplemented!()
    }

    fn reset_fences(&self, fences: Vec<Self::Fence>) {
        unimplemented!()
    }

    fn update_descriptor_sets(&self, updates: Vec<DescriptorSetWrite>) {
        unimplemented!()
    }
}
