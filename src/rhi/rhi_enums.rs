use bitflags::bitflags;
use failure::Fail;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PhysicalDeviceManufacturer {
    Nvidia,
    AMD,
    Intel,
    Other,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PhysicalDeviceType {
    Integrated,
    Discreet,
    Virtual,
    CPU,
    Other,
}

/// How a piece of memory will be used
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MemoryUsage {
    /// The memory will only be used by device
    ///
    /// Useful for things like vertex buffers and dynamic textures
    DeviceOnly,

    /// The memory will be written to by the CPU, but will only be written to a handful of times per frame
    ///
    /// Useful for the model matrix buffer, the per-frame data buffer, and other uniform buffers which are updated a
    /// few times per frame
    LowFrequencyUpload,

    /// The memory will be used for a staging buffer
    StagingBuffer,
}

/// Describes what kind of object you want to allocate from a new memory pool
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectType {
    Buffer,
    Texture,
    Attachment,
    SwapchainSurface,
    Any,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum QueueType {
    Graphics,
    Compute,
    Copy,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CommandListLevel {
    Primary,
    Secondary,
}

#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum DeviceCreationError {
    #[fail(display = "Failed to create device.")]
    Failed,
}

/// A memory-related error
#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum MemoryError {
    #[fail(display = "There's not enough host memory to create the requested object.")]
    OutOfHostMemory,

    #[fail(display = "There's not enough device memory to create the requested object.")]
    OutOfDeviceMemory,
}

/// Errors tha can happen when you try to get a queue from a device
#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum QueueGettingError {
    #[fail(display = "The device does not have enough memory to get you the queue you want.")]
    OutOfMemory,

    #[fail(display = "The device does not support this queue type")]
    NotSupported,

    #[fail(display = "Queue index is out of range")]
    IndexOutOfRange,
}

/// All the errors you might get when allocating memory
#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum AllocationError {
    #[fail(display = "There's not enough host memory to make the requested allocation")]
    OutOfHostMemory,

    #[fail(display = "There's not enough device memory to make the requested allocation.")]
    OutOfDeviceMemory,

    #[fail(display = "You've made too many memory allocations already.")]
    TooManyObjects,

    #[fail(display = "Handle Invalid")]
    InvalidExternalHandle,

    #[fail(display = "Memory mapping failed")]
    MappingFailed,

    #[fail(display = "No memory matching the requirements found")]
    NoSuitableMemoryFound,
}

#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum DescriptorPoolCreationError {
    #[fail(display = "There's not enough host memory to create the descriptor pool.")]
    OutOfHostMemory,

    #[fail(display = "There's not enough device memory to create the descriptor pool.")]
    OutOfDeviceMemory,

    #[fail(display = "Memory is too fragmented to create the descriptor pool.")]
    Fragmentation,
}

#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum PipelineCreationError {
    #[fail(display = "There's not enough host memory to create the pipeline.")]
    OutOfHostMemory,

    #[fail(display = "There's not enough device memory to create the pipeline.")]
    OutOfDeviceMemory,

    #[fail(
        display = "One or more shaders failed to compile or link. If debug reports are enabled, details are reported through a debug report."
    )]
    InvalidShader,
}

/// The state a resource is in
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResourceState {
    /// The state is not defined. The GPU may or may not do _things_ with the resource
    Undefined,
    /// The resource may be used for anything you want, but it won't be optimal for anything
    General,

    ColorAttachment,
    DepthStencilAttachment,
    DepthReadOnlyStencilAttachment,
    DepthAttachmentStencilReadOnly,
    DepthStencilReadOnlyAttachment,

    PresentSource,

    NonFragmentShaderReadOnly,
    FragmentShaderReadOnly,

    TransferSource,
    TransferDestination,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DescriptorType {
    CombinedImageSampler,
    UniformBuffer,
    StorageBuffer,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BufferUsage {
    UniformBuffer,
    IndexBuffer,
    VertexBuffer,
    StagingBuffer,
}

bitflags! {
    pub struct PipelineStageFlags: u32 {
        const TOP_OF_PIPE = 0x0000_0001;
        const DRAW_INDIRECT = 0x0000_0002;
        const VERTEX_INPUT = 0x0000_0004;
        const VERTEX_SHADER = 0x0000_0008;
        const TESSELLATION_CONTROL_SHADER = 0x0000_0010;
        const TESSELLATION_EVALUATION_SHADER = 0x0000_0020;
        const GEOMETRY_SHADER = 0x0000_0040;
        const FRAGMENT_SHADER = 0x0000_0080;
        const EARLY_FRAGMENT_TESTS = 0x0000_0100;
        const LATE_FRAGMENT_TESTS = 0x0000_0200;
        const COLOR_ATTACHMENT_OUTPUT = 0x0000_0400;
        const COMPUTE_SHADER = 0x0000_0800;
        const TRANSFER = 0x0000_1000;
        const BOTTOM_OF_PIPE = 0x0000_2000;
        const HOST = 0x0000_4000;
        const ALL_GRAPHICS = 0x0000_8000;
        const ALL_COMMANDS = 0x0001_0000;
        const SHADING_RATE_IMAGE = 0x0040_0000;
        const RAY_TRACING_SHADER = 0x0020_0000;
        const ACCELERATION_STRUCTURE_BUILD = 0x0200_0000;
        const TASK_SHADER = 0x0008_0000;
        const MESH_SHADER = 0x0010_0000;
        const FRAGMENT_DENSITY_PROCESS = 0x0080_0000;
    }
}

bitflags! {
    pub struct ResourceAccessFlags: u32 {
        const NO_FLAGS = 0x0000_0000;
        const INDEX_READ_BIT = 0x0000_0002;
        const VERTEX_ATTRIBUTE_READ_BIT = 0x0000_0004;
        const UNIFORM_READ_BIT = 0x0000_0008;
        const INPUT_ATTACHMENT_READ_BIT = 0x0000_0010;
        const SHADER_READ_BIT = 0x0000_0020;
        const SHADER_WRITE_BIT = 0x0000_0040;
        const COLOR_ATTACHMENT_READ_BIT = 0x0000_0080;
        const COLOR_ATTACHMENT_WRITE_BIT = 0x0000_0100;
        const DEPTH_STENCIL_ATTACHMENT_READ_BIT = 0x0000_0200;
        const DEPTH_STENCIL_ATTACHMENT_WRITE_BIT = 0x0000_0400;
        const TRANSFER_READ_BIT = 0x0000_0800;
        const TRANSFER_WRITE_BIT = 0x0000_1000;
        const HOST_READ_BIT = 0x0000_2000;
        const HOST_WRITE_BIT = 0x0000_4000;
        const MEMORY_READ_BIT = 0x0000_8000;
        const MEMORY_WRITE_BIT = 0x0001_0000;
    }
}

bitflags! {
    pub struct ImageAspectFlags: u32 {
        const COLOR = 0x0000_0001;
        const DEPTH = 0x0000_0002;
        const STENCIL = 0x0000_0004;
    }
}

bitflags! {
    pub struct ShaderStageFlags: u32 {
        const VERTEX = 0x0001;
        const TESSELLATION_CONTROL = 0x0002;
        const TESSELLATION_EVALUATION = 0x0004;
        const GEOMETRY = 0x0008;
        const FRAGMENT = 0x0010;
        const COMPUTE = 0x0020;
        const RAYGEN = 0x0100;
        const ANY_HIT = 0x0200;
        const CLOSEST_HIT = 0x0400;
        const MISS = 0x0800;
        const INTERSECTION = 0x1000;
        const TASK = 0x0040;
        const MESH = 0x0080;
    }
}
