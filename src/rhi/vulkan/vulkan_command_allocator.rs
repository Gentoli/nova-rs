#![allow(unsafe_code)]

use crate::rhi::vulkan::vulkan_command_list::VulkanCommandList;
use crate::rhi::*;

use ash;
use ash::version::DeviceV1_0;
use ash::vk;

pub struct VulkanCommandAllocator {
    instance: ash::Instance,
    device: ash::Device,
    command_pool: vk::CommandPool,
}

impl VulkanCommandAllocator {
    pub fn new(
        create_info: CommandAllocatorCreateInfo,
        rhi_device: &vulkan::vulkan_device::VulkanDevice,
        instance: ash::Instance,
        device: ash::Device,
    ) -> Result<VulkanCommandAllocator, MemoryError> {
        let queue_family_index = match create_info.command_list_type {
            QueueType::Graphics => rhi_device.get_graphics_queue_family_index(),
            QueueType::Copy => rhi_device.get_compute_queue_family_index(),
            QueueType::Compute => match rhi_device.get_compute_queue_family_index() {
                // TODO: We are not really out of device memory, just the device doesn't provide
                //       the requested queue family
                None => Err(MemoryError::OutOfDeviceMemory),
                Some(v) => v,
            },
        };

        let create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .build();

        match unsafe { device.create_command_pool(&create_info, None) } {
            Err(result) => match result {
                vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(MemoryError::OutOfHostMemory),
                vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(MemoryError::OutOfDeviceMemory),
                _ => panic!("Invalid error result returned: {}", result),
            },
            Ok(command_pool) => Ok(VulkanCommandAllocator {
                instance,
                device,
                command_pool,
            }),
        }
    }
}

impl CommandAllocator for VulkanCommandAllocator {
    type CommandList = VulkanCommandList;

    fn create_command_list(&self) -> Result<Self::CommandList, MemoryError> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .command_buffer_count(1)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();

        match unsafe { self.device.allocate_command_buffers(&allocate_info) } {
            Err(result) => match result {
                vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(MemoryError::OutOfHostMemory),
                vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(MemoryError::OutOfDeviceMemory),
                _ => panic!("Invalid error result returned: {}", result),
            },
            Ok(mut buffers) => Ok(VulkanCommandList::new(
                self.instance.clone(),
                self.device.clone(),
                buffers.remove(0),
            )),
        }
    }
}
