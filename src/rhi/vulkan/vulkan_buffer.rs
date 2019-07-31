use crate::rhi::*;

use ash::vk;

pub struct VulkanBuffer {
    pub vk_buffer: vk::Buffer,
}

impl Resource for VulkanBuffer {}
impl Buffer for VulkanBuffer {
    fn write_data(&self, data: BufferCreateInfo, num_bytes: u64, offset: u64) {
        unimplemented!()
    }
}
