use bitflags::bitflags;
use failure::Fail;

/// Actual manufacturer of the gpu.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PhysicalDeviceManufacturer {
    /// Made by Nvidia Corporation.
    Nvidia,

    /// Made by Advanced Micro Devices, Inc.
    AMD,

    /// Made by Intel Corporation.
    Intel,

    /// Made by other unknown manufacturer.
    Other,
}

/// The classification of a graphics device.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PhysicalDeviceType {
    /// A physical GPU onboard the CPU.
    Integrated,

    /// A physically separate GPU from the CPU.
    Discrete,

    /// Device is emulated or otherwise altered with.
    Virtual,

    /// The actual cpu itself.
    CPU,

    /// Unknown physical device type.
    Other,
}

/// How a piece of memory will be used.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MemoryUsage {
    /// The memory will only be used by device.
    ///
    /// Useful for things like vertex buffers and dynamic textures.
    DeviceOnly,

    /// The memory will be written to by the CPU, but will only be written to a handful of times per frame.
    ///
    /// Useful for the model matrix buffer, the per-frame data buffer, and other uniform buffers which are updated a
    /// few times per frame.
    LowFrequencyUpload,

    /// The memory will be used for a staging buffer.
    StagingBuffer,
}

/// Describes what kind of object you want to allocate from a new memory pool.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectType {
    /// Allocate buffers for storing vertex data or other data.
    Buffer,

    /// Allocate textures to put in images in.
    Texture,

    /// Allocate framebuffer attachments.
    Attachment,

    /// Allocate surfaces for the swapchain to render onto.
    SwapchainSurface,

    /// Allocate any object.
    Any,
}

/// Describes the operations the queue supports.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum QueueType {
    /// Provides access to full rasterization pipeline.
    Graphics,

    /// Provides access to compute hardware.
    Compute,

    /// Optimized for passing data over PCI-e bus.
    Copy,
}

/// Dictates what abilities a command list has.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum CommandListLevel {
    /// Command List can be executed directly buy a queue
    Primary,

    /// Command List must be executed by a primary command list. Has many restrictions.
    Secondary,
}

/// Failure type for device creation.
#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum DeviceCreationError {
    /// Failed to create device.
    #[fail(display = "Failed to create device.")]
    Failed,
}

/// Failure type for memory-related errors.
#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum MemoryError {
    /// Not enough host memory to create the requested object.
    #[fail(display = "There's not enough host memory to create the requested object.")]
    OutOfHostMemory,

    /// Not enough device memory to create the requested object.
    #[fail(display = "There's not enough device memory to create the requested object.")]
    OutOfDeviceMemory,
}

/// Failure type for errors that can happen when you try to get a queue from a device.
#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum QueueGettingError {
    /// The device does not have enough memory to get you the queue you want.
    #[fail(display = "The device does not have enough memory to get you the queue you want.")]
    OutOfMemory,

    /// The device does not support this queue type.
    #[fail(display = "The device does not support this queue type.")]
    NotSupported,

    /// Queue index is out of range.
    #[fail(display = "Queue index is out of range.")]
    IndexOutOfRange,
}

/// Failure type for errors you get when allocating memory.
#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum AllocationError {
    /// There's not enough host memory to make the requested allocation.
    #[fail(display = "There's not enough host memory to make the requested allocation.")]
    OutOfHostMemory,

    /// There's not enough device memory to make the requested allocation.
    #[fail(display = "There's not enough device memory to make the requested allocation.")]
    OutOfDeviceMemory,

    /// You've made too many memory allocations already.
    #[fail(display = "You've made too many memory allocations already.")]
    TooManyObjects,

    /// Handle Invalid.
    #[fail(display = "Handle Invalid.")]
    InvalidExternalHandle,

    /// Memory mapping failed.
    #[fail(display = "Memory mapping failed.")]
    MappingFailed,

    /// No memory matching the requirements found.
    #[fail(display = "No memory matching the requirements found.")]
    NoSuitableMemoryFound,
}

/// Failure type for errors when creating a descriptor pool.
#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum DescriptorPoolCreationError {
    /// There's not enough host memory to create the descriptor pool.
    #[fail(display = "There's not enough host memory to create the descriptor pool.")]
    OutOfHostMemory,

    /// There's not enough device memory to create the descriptor pool.
    #[fail(display = "There's not enough device memory to create the descriptor pool.")]
    OutOfDeviceMemory,

    /// Memory is too fragmented to create the descriptor pool.
    #[fail(display = "Memory is too fragmented to create the descriptor pool.")]
    Fragmentation,
}

/// Failure type for errors when creating a pipeline.
#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum PipelineCreationError {
    /// There's not enough host memory to create the pipeline.
    #[fail(display = "There's not enough host memory to create the pipeline.")]
    OutOfHostMemory,

    /// There's not enough device memory to create the pipeline.
    #[fail(display = "There's not enough device memory to create the pipeline.")]
    OutOfDeviceMemory,

    /// One or more shaders failed to compile or link. If debug reports are enabled, details are reported through a
    /// debug report.
    #[fail(
        display = "One or more shaders failed to compile or link. If debug reports are enabled, details are reported through a debug report."
    )]
    InvalidShader(String),
}

/// Failure type for mapping a buffer
#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum MappingError {
    /// The resource is in device local memory and therefore can't be mapped
    #[fail(display = "Resource is in device local memory.")]
    ResourceInDeviceMemory,

    /// Mapping failed for a generic reason
    #[fail(display = "Mapping failed for an unknown reason.")]
    MappingFailed,
}

/// The state of a resource. The resource will be optimized for the given use case, though it may still be used in
/// others.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResourceState {
    /// The state is not defined. The GPU may or may not do _things_ with the resource.
    Undefined,

    /// The resource may be used for anything you want, but it won't be optimal for anything.
    General,

    /// Resource optimized to be a color attachment of a framebuffer.
    ColorAttachment,

    /// Resource optimized to be a depth-stencil attachment of a framebuffer.
    DepthStencilAttachment,

    /// Resource optimized to be a depth-stencil attachment of a framebuffer with _depth_ **read only**.
    DepthReadOnlyStencilAttachment,

    /// Resource optimized to be a depth-stencil attachment of a framebuffer with _stencil_ **read only**.
    DepthAttachmentStencilReadOnly,

    /// Resource optimized to be a depth-stencil attachment of a framebuffer with _both_ **read only**.
    DepthStencilReadOnlyAttachment,

    /// Resource optimized to be presented to the active window
    PresentSource,

    /// Resource optimized to be a non-fragment shader in read-only mode.
    NonFragmentShaderReadOnly,

    /// Resource optimized to be a fragment shader in read-only mode.
    FragmentShaderReadOnly,

    /// Resource optimized to be the source of a transfer.
    TransferSource,

    /// Resource optimized to be the destination of a transfer.
    TransferDestination,
}

/// Type of object current descriptor points to.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DescriptorType {
    /// Handle to a combined image and image sampler.
    CombinedImageSampler,

    /// Handle to a buffer fed into a uniform slot of a shader.
    UniformBuffer,

    /// Handle to a buffer that a shader can read and write.
    StorageBuffer,
}

/// Current use of a buffer.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BufferUsage {
    /// A buffer of uniform values.
    UniformBuffer,

    /// The index buffer for rasterization.
    IndexBuffer,

    /// The vertex buffer for rasterization.
    VertexBuffer,

    /// Buffer waiting for transfer to/from another buffer.
    StagingBuffer,
}

bitflags! {
    /// Pipeline Stage.
    ///
    /// Documentation from [vulkan registry](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkPipelineStageFlagBits.html).
    pub struct PipelineStageFlags: u32 {
        /// Stage of the pipeline where any commands are initially received by the queue.
        const TOP_OF_PIPE = 0x0000_0001;

        /// Stage of the pipeline where Draw/DispatchIndirect data structures are consumed.
        const DRAW_INDIRECT = 0x0000_0002;

        /// Stage of the pipeline where vertex and index buffers are consumed.
        const VERTEX_INPUT = 0x0000_0004;

        /// Vertex shader stage.
        const VERTEX_SHADER = 0x0000_0008;

        /// Tessellation control shader stage.
        const TESSELLATION_CONTROL_SHADER = 0x0000_0010;

        /// Tessellation evaluation shader stage.
        const TESSELLATION_EVALUATION_SHADER = 0x0000_0020;

        /// Geometry shader stage.
        const GEOMETRY_SHADER = 0x0000_0040;

        /// Fragment shader stage.
        const FRAGMENT_SHADER = 0x0000_0080;

        /// Stage of the pipeline where early fragment tests (depth and stencil tests before fragment shading) are
        /// performed. This stage also includes subpass load operations for framebuffer attachments with a depth/stencil format.
        const EARLY_FRAGMENT_TESTS = 0x0000_0100;

        /// Stage of the pipeline where late fragment tests (depth and stencil tests after fragment shading) are
        /// performed. This stage also includes subpass store operations for framebuffer attachments with a depth/stencil format.
        const LATE_FRAGMENT_TESTS = 0x0000_0200;

        /// Stage of the pipeline after blending where the final color values are output from the pipeline.
        /// This stage also includes subpass load and store operations and multisample resolve operations
        /// for framebuffer attachments with a color or depth/stencil format.
        const COLOR_ATTACHMENT_OUTPUT = 0x0000_0400;

        /// Execution of a compute shader
        const COMPUTE_SHADER = 0x0000_0800;

        /// Execution of copy commands. This includes the operations resulting from all copy commands, clear commands
        /// (with the exception of `vkCmdClearAttachments`), and `vkCmdCopyQueryPoolResults`.
        ///
        /// FIXME(cwfitzgerald): Translate vulkan commands to nova?
        const TRANSFER = 0x0000_1000;

        /// Final stage in the pipeline where operations generated by all commands complete execution.
        const BOTTOM_OF_PIPE = 0x0000_2000;

        /// Pseudo-stage indicating execution on the host of reads/writes of device memory.
        /// This stage is not invoked by any commands recorded in a command buffer.
        const HOST = 0x0000_4000;

        /// Execution of all graphics pipeline stages.
        const ALL_GRAPHICS = 0x0000_8000;

        /// Execution of all pipeline stages the queue supports.
        const ALL_COMMANDS = 0x0001_0000;

        /// Stage of the pipeline where the shading rate image is read to determine
        /// the shading rate for portions of a rasterized primitive.
        const SHADING_RATE_IMAGE = 0x0040_0000;

        /// Execution of the ray tracing shader stages.
        const RAY_TRACING_SHADER = 0x0020_0000;

        /// Execution of stage constructing a ray tracing acceleration structure.
        const ACCELERATION_STRUCTURE_BUILD = 0x0200_0000;

        /// Task shader stage.
        const TASK_SHADER = 0x0008_0000;

        /// Mesh shader stage.
        const MESH_SHADER = 0x0010_0000;

        /// Stage of the pipeline where the fragment density map is read to generate the fragment areas.
        const FRAGMENT_DENSITY_PROCESS = 0x0080_0000;
    }
}

bitflags! {
    /// Specifies the parts of the pipeline that can access the memory.
    ///
    /// Documentation taken from the [vulkan registry](https://www.khronos.org/registry/vulkan/specs/1.1-extensions/man/html/VkAccessFlagBits.html).
    pub struct ResourceAccessFlags: u32 {
        /// Inaccessible.
        const NO_FLAGS = 0x0000_0000;

        /// Read access to an index buffer as part of an indexed drawing command.
        const INDEX_READ_BIT = 0x0000_0002;

        /// Read access to a vertex buffer as part of a drawing command.
        const VERTEX_ATTRIBUTE_READ_BIT = 0x0000_0004;

        /// Read access to a uniform buffer.
        const UNIFORM_READ_BIT = 0x0000_0008;

        /// Read access to an input attachment within a render pass during fragment shading.
        const INPUT_ATTACHMENT_READ_BIT = 0x0000_0010;

        /// Read access to a storage buffer, physical storage buffer, uniform texel buffer,
        /// storage texel buffer, sampled image, or storage image.
        const SHADER_READ_BIT = 0x0000_0020;

        /// Write write access to a storage buffer, physical storage buffer, storage texel buffer,
        /// or storage image.
        const SHADER_WRITE_BIT = 0x0000_0040;

        /// Read access to a color attachment, such as via blending, logic operations,
        /// or via certain subpass load operations. It does not include advanced blend operations.
        const COLOR_ATTACHMENT_READ_BIT = 0x0000_0080;

        /// Write write access to a color, resolve, or depth/stencil resolve attachment during a
        /// render pass or via certain subpass load and store operations.
        const COLOR_ATTACHMENT_WRITE_BIT = 0x0000_0100;

        /// Read access to a depth/stencil attachment, via depth or stencil operations
        /// or via certain subpass load operations.
        const DEPTH_STENCIL_ATTACHMENT_READ_BIT = 0x0000_0200;

        /// Write access to a depth/stencil attachment, via depth or stencil operations
        /// or via certain subpass load and store operations.
        const DEPTH_STENCIL_ATTACHMENT_WRITE_BIT = 0x0000_0400;

        /// Read access to an image or buffer in a copy operation.
        const TRANSFER_READ_BIT = 0x0000_0800;

        /// Write access to an image or buffer in a clear or copy operation.
        const TRANSFER_WRITE_BIT = 0x0000_1000;

        /// Read access by a host operation. Accesses of this type are not performed
        /// through a resource, but directly on memory.
        const HOST_READ_BIT = 0x0000_2000;

        /// Write access by a host operation. Accesses of this type are not performed
        /// through a resource, but directly on memory.
        const HOST_WRITE_BIT = 0x0000_4000;

        /// Read access via non-specific entities. These entities include the Vulkan device and host,
        /// but may also include entities external to the Vulkan device or otherwise not part of the core Vulkan pipeline.
        /// When included in a destination access mask, makes all available writes visible to all
        /// future read accesses on entities known to the Vulkan device.
        const MEMORY_READ_BIT = 0x0000_8000;

        /// Write access via non-specific entities. These entities include the Vulkan device and host,
        /// but may also include entities external to the Vulkan device or otherwise not part of the core Vulkan pipeline.
        /// When included in a source access mask, all writes that are performed by entities known to the Vulkan device
        /// are made available. When included in a destination access mask, makes all available writes visible to all future
        /// write accesses on entities known to the Vulkan device.
        const MEMORY_WRITE_BIT = 0x0001_0000;
    }
}

bitflags! {
    /// Aspect of an image included in a view.
    pub struct ImageAspectFlags: u32 {
        /// Color aspect.
        const COLOR = 0x0000_0001;
        /// Depth aspect.
        const DEPTH = 0x0000_0002;
        /// Stencil aspect.
        const STENCIL = 0x0000_0004;
    }
}

bitflags! {
    /// Shader stage.
    pub struct ShaderStageFlags: u32 {
        /// Vertex stage.
        const VERTEX = 0x0001;
        /// Tessellation Control stage.
        const TESSELLATION_CONTROL = 0x0002;
        /// Tessellation Evaluation stage.
        const TESSELLATION_EVALUATION = 0x0004;
        /// Geometry stage.
        const GEOMETRY = 0x0008;
        /// Fragment stage.
        const FRAGMENT = 0x0010;
        /// Compute stage.
        const COMPUTE = 0x0020;
        /// Raygen stage.
        const RAYGEN = 0x0100;
        /// Any Hit stage.
        const ANY_HIT = 0x0200;
        /// Closest Hit stage.
        const CLOSEST_HIT = 0x0400;
        /// Miss stage.
        const MISS = 0x0800;
        /// Intersection stage.
        const INTERSECTION = 0x1000;
        /// Task stage.
        const TASK = 0x0040;
        /// Mesh stage.
        const MESH = 0x0080;
    }
}
