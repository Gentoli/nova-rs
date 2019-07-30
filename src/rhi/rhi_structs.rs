use super::{rhi_enums::*, rhi_traits::*};
use crate::shaderpack;
use std::sync::Arc;

/// Describes what kind of command allocator you want to create.
#[derive(Debug, Clone)]
pub struct CommandAllocatorCreateInfo {
    /// The type of command lists which will be allocated by this command allocator
    pub command_list_type: QueueType,

    /// A bitmask of the GPU that the new command allocator will allocate commands for.
    ///
    /// Only one GPU may be used.
    pub node_mask: u32,
}

/// Information about a physical device.
///
/// This information can come from multiple API calls, but I've merged all the information together here.
///
/// This structure has things like the capabilities of the device, its hardware limits, its manufacturer and model
/// number, etc
#[derive(Debug, Clone)]
pub struct PhysicalDeviceProperties {
    /// Company that manufactured the physical device.
    pub manufacturer: PhysicalDeviceManufacturer,

    /// Integer device id.
    pub device_id: u32,

    /// Full string device name.
    pub device_name: String,

    /// Type of device we are talking to.
    pub device_type: PhysicalDeviceType,

    /// Count of color attachments usable.
    pub max_color_attachments: u32,
}

/// Data corresponding to a particular resource.
#[derive(Debug, Clone)]
pub enum ResourceSpecificData {
    /// The resource is an image.
    Image {
        /// The aspect of the image.
        aspect: ImageAspectFlags,
    },

    /// The resource is a buffer.
    Buffer {
        /// Offset into the underlying storage.
        offset: u64,
        /// Size of the buffer.
        size: u64,
    },
}

/// Barrier for resources.
#[derive(Clone)]
pub struct ResourceBarrier {
    /// The resource the barrier is guarding.
    pub resource: Arc<dyn Resource>,

    /// Initial resource state before the barrier.
    pub initial_state: ResourceState,

    /// Final resource state after the barrier.
    pub final_state: ResourceState,

    /// Accessibility before barrier.
    pub access_before_barrier: ResourceAccessFlags,

    /// Accessibility before barrier.
    pub access_after_barrier: ResourceAccessFlags,

    /// Starting queue.
    pub source_queue: QueueType,

    /// Finishing queue.
    pub destination_queue: QueueType,

    /// Information about the resource being guarded.
    pub resource_info: ResourceSpecificData,
}

/// Data that goes into updating a descriptor.
#[derive(Clone)]
pub enum DescriptorUpdateInfo {
    /// The descriptor is an image.
    Image {
        /// The image that will form the descriptor data.
        image: Arc<dyn Image>,

        /// The image format.
        format: shaderpack::TextureFormat,

        /// The image sampler to use.
        sampler: Arc<dyn Sampler>,
    },
}

/// Data for writing to a descriptor set.
#[derive(Clone)]
pub struct DescriptorSetWrite {
    /// Set to be written to.
    pub set: Arc<dyn DescriptorSet>,

    /// Descriptor binding id.
    pub binding: u32,

    /// What to update the descriptor with.
    pub update_info: DescriptorUpdateInfo,
}

/// Binding of a resource to a descriptor.
#[derive(Debug, Clone)]
pub struct ResourceBindingDescription {
    /// Descriptor set that his binding belongs to.
    pub set: u32,

    /// Binding of this resource binding.
    pub binding: u32,

    /// Number of bindings. Useful if you have an array of descriptors.
    pub count: u32,

    /// The type of object that will be bound.
    pub descriptor_type: DescriptorType,

    /// The shader stages that need access to this binding.
    pub stages: ShaderStageFlags,
}

/// Data for buffer creation.
#[derive(Debug, Clone)]
pub struct BufferCreateInfo {
    /// Size of the buffer.
    pub size: usize,

    /// The usage profile of the buffer
    pub buffer_usage: BufferUsage,

    /// The allocation to use for the buffer
    pub allocation: DeviceMemoryAllocation,
}

/// Memory allocation on a specific device.
#[derive(Debug, Clone)]
pub struct DeviceMemoryAllocation;
