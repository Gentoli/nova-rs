use super::{rhi_enums::*, rhi_traits::*};
use crate::shaderpack;
use std::sync::Arc;

/// Describes what kind of command allocator you want to create
#[derive(Debug, Clone)]
pub struct CommandAllocatorCreateInfo {
    /// The type of command lists which will be allocated by this command allocator
    pub command_list_type: QueueType,

    // A bitmask of the GPU that the new command allocator will allocate commands for. Only one GPU mey be used
    pub node_mask: u32,
}

/// Information about a physical device!
///
/// This information can come from multiple API calls, but I've merged all the information together here
///
/// This structure has things like the capabilities of the device, its hardware limits, its manufacturer and model
/// number, etc
#[derive(Debug, Clone)]
pub struct PhysicalDeviceProperties {
    pub manufacturer: PhysicalDeviceManufacturer,

    pub device_id: u32,

    pub device_name: Box<str>,

    pub device_type: PhysicalDeviceType,

    pub max_color_attachments: u32,
}

#[derive(Debug, Clone)]
pub enum ResourceSpecificData {
    Image { aspect: ImageAspectFlags },
    Buffer { offset: u64, size: u64 },
}

#[derive(Clone)]
pub struct ResourceBarrier {
    pub resource: Arc<dyn Resource>,

    pub initial_state: ResourceState,

    pub final_state: ResourceState,

    pub access_before_barrier: ResourceAccessFlags,

    pub access_after_barrier: ResourceAccessFlags,

    pub source_queue: QueueType,

    pub destination_queue: QueueType,

    pub resource_info: ResourceSpecificData,
}

#[derive(Clone)]
pub enum DescriptorUpdateInfo {
    Image {
        image: Arc<dyn Image>,
        format: shaderpack::TextureFormat,
        sampler: Arc<dyn Sampler>,
    },
}

#[derive(Clone)]
pub struct DescriptorSetWrite {
    pub set: Arc<dyn DescriptorSet>,

    pub binding: u32,

    pub update_info: DescriptorUpdateInfo,
}

#[derive(Debug, Clone)]
pub struct ResourceBindingDescription {
    /// Descriptor set that his binding belongs to
    pub set: u32,

    /// Binding of this resource binding
    pub binding: u32,

    /// Number of bindings. Useful if you have an array of descriptors
    pub count: u32,

    /// The type of object that will be bound
    pub descriptor_type: DescriptorType,

    /// The shader stages that need access to this binding
    pub stages: ShaderStageFlags,
}

#[derive(Debug, Clone)]
pub struct BufferCreateInfo {
    pub size: usize,

    pub buffer_usage: BufferUsage,

    pub allocation: DeviceMemoryAllocation,
}

#[derive(Debug, Clone)]
pub struct DeviceMemoryAllocation;
