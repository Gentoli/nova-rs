//! Nova's Render Hardware Interface
//!
//! This is an interface to the GPU which has been designed for Nova. It abstracts away parts of the
//! underlying APIs which Nova doesn't use, providing an interface that's more productive and more
//! fun. The RHI is actually split into two sections: the synchronous parts and the asynchronous
//! part. The synchronous part of the API is where your calls happen immediately on the GPU, while
//! the asynchronous part is where your calls get recorded into command lists, which are later
//! executed on the GPU.

use std::collections::HashMap;

use super::{rhi_enums::*, rhi_structs::*};
use crate::shaderpack;
use crate::surface::Surface;
use cgmath::Vector2;
use std::rc::Rc;

/// Top-level trait for functions that don't belong to any specific device object.
pub trait GraphicsApi {
    /// Corresponding physical device.
    type Device: Device;

    /// Corresponding platform surface.
    type PlatformSurface;

    /// Gets a list of all available graphics adapters.
    fn get_adapters(&self) -> Vec<Self::Device>;

    /// Gets the surface this API was created with.
    fn get_surface(&self) -> Rc<dyn Surface<Self::PlatformSurface>>;
}

/// An implementation of the rendering API for a specific device.
///
/// This will probably be a physical GPU, but a software implementation of either Vulkan or Direct3D 12 is possible.
pub trait Device {
    /// Device's queue type.
    type Queue: Queue;

    /// Device's memory type.
    type Memory: Memory;

    /// Device's command allocator type.
    type CommandAllocator: CommandAllocator;

    /// Device's image type.
    type Image: Image;

    /// Device's renderpass type.
    type Renderpass: Renderpass;

    /// Device's framebuffer type.
    type Framebuffer: Framebuffer;

    /// Device's pipeline interface type.
    type PipelineInterface: PipelineInterface;

    /// Device's descriptor pool type.
    type DescriptorPool: DescriptorPool;

    /// Device's pipeline type.
    type Pipeline: Pipeline;

    /// Device's semaphore type.
    type Semaphore: Semaphore;

    /// Device's fence type.
    type Fence: Fence;

    /// Accesses all properties of the physical device.
    fn get_properties(&self) -> DeviceProperties;

    /// Checks if this physical device is suitable for Nova.
    ///
    /// Devices are suitable for Nova if they:
    /// - Have queues that support graphics, compute, transfer, and present operations.
    /// - Support tessellation and geometry shaders.
    ///
    /// Nova's supported APIs have very different ways to check what features and capabilities a
    /// physical device has, so this method encapsulates all that.
    ///
    /// Future work will probably come up with a way to score physical devices from most suitable to
    /// least suitable, but for now this is fine.
    fn can_be_used_by_nova(&self) -> bool;

    /// Gets the amount of free VRAM on this physical device.
    fn get_free_memory(&self) -> u64;

    /// Retrieves the Queue with the provided queue family index and queue index.
    ///
    /// The caller should verify that the device supports the requested queue index and queue
    /// family index.
    ///
    /// # Parameters
    ///
    /// * `queue_type` - The type of queue you want.
    /// * `queue_index` - The index of the queue to get from the selected queue family.
    fn get_queue(&self, queue_type: QueueType, queue_index: u32) -> Result<Self::Queue, QueueGettingError>;

    /// Allocates memory from the graphics API.
    ///
    /// This memory may be on the device or on the host, depending on its usage and allowed objects.
    ///
    /// # Parameters
    ///
    /// * `size` - The size, in bytes, of the memory you want to allocate.
    /// * `memory_usage` - The usage you want the memory to be usable for.
    /// * `allowed_objects` - The types of objects you want to allow from this memory. Enforcing
    /// this is up to the caller.
    fn allocate_memory(
        &self,
        size: u64,
        memory_usage: MemoryUsage,
        allowed_objects: ObjectType,
    ) -> Result<Self::Memory, AllocationError>;

    /// Creates a new CommandAllocator.
    ///
    /// # Parameters
    ///
    /// * `create_info` - Information about how you want the CommandAllocator created.
    fn create_command_allocator(
        &self,
        create_info: CommandAllocatorCreateInfo,
    ) -> Result<Self::CommandAllocator, MemoryError>;

    /// Creates a new renderpass from the provided shaderpack data.
    ///
    /// # Parameters
    ///
    /// * `data` - The shaderpack data to create the renderpass from.
    fn create_renderpass(&self, data: shaderpack::RenderPassCreationInfo) -> Result<Self::Renderpass, MemoryError>;

    /// Creates a new Framebuffer
    ///
    /// Framebuffers get their attachment layout from a renderpass. I do not know why Khronos didn't
    /// make a separate type for a framebuffer interface, yet here we are. Thus, this method takes in
    /// the renderpass to use an interface.
    ///
    /// # Parameters
    ///
    /// * `renderpass` - The Renderpass to get the framebuffer layout from.
    /// * `attachments` - The images to attach to the framebuffer, in attachment order.
    /// * `framebuffer_size` - The size of the framebuffer, in pixels.
    fn create_framebuffer(
        &self,
        renderpass: Self::Renderpass,
        attachments: Vec<Self::Image>,
        framebuffer_size: Vector2<f32>,
    ) -> Result<Self::Framebuffer, MemoryError>;

    /// Creates a PipelineInterface from the provided information.
    ///
    /// # Parameters
    ///
    /// * `bindings` - The bindings that the pipeline exposes.
    /// * `color_attachments` - All the color attachments that the pipeline writes to.
    /// * `depth_texture` - The depth texture that this pipeline writes to, if it writes to one.
    fn create_pipeline_interface(
        &self,
        bindings: &HashMap<String, ResourceBindingDescription>,
        color_attachments: &[shaderpack::TextureAttachmentInfo],
        depth_texture: &Option<shaderpack::TextureAttachmentInfo>,
    ) -> Result<Self::PipelineInterface, MemoryError>;

    /// Creates a DescriptorPool with the desired descriptors.
    ///
    /// # Parameters
    ///
    /// * `num_sampled_images` - The number of sampled image descriptors you'll make from the new pool.
    /// * `num_samplers` - The number of sampler descriptors you'll make from the pool.
    /// * `num_uniform_buffers` - The number of UBO/CBV or SSBO/UAV descriptors you'll make from the pool.
    fn create_descriptor_pool(
        &self,
        num_sampled_images: u32,
        num_samplers: u32,
        num_uniform_buffers: u32,
    ) -> Result<Self::DescriptorPool, DescriptorPoolCreationError>;

    /// Creates a Pipeline with the provided PipelineInterface and the given PipelineCreateInfo.
    ///
    /// # Parameters
    ///
    /// * `pipeline_interface` - The interface you want the new pipeline to have.
    /// * `data` - The data to create a pipeline from.
    fn create_pipeline(
        &self,
        pipeline_interface: Self::PipelineInterface,
        data: shaderpack::PipelineCreationInfo,
    ) -> Result<Self::Pipeline, PipelineCreationError>;

    /// Creates an Image from the specified ImageCreateInto.
    ///
    /// Images are created directly from the Device and not from a MemoryPool. In Nova, images are either render
    /// targets, which should have a dedicated allocation, or a virtual texture, which should also have a dedicated
    /// allocation because of its size. All the types of images that Nova deals with need a dedicated allocation, so
    /// there's no support for creating an image from a non-unique memory allocation
    ///
    /// # Parameters
    ///
    /// * `data` - The ImageData to create the image from.
    /// * `swapchain_size` - The size of the swapchain, in pixels. Used to resolve the size of swapchain-relative images
    fn create_image(
        &self,
        data: shaderpack::TextureCreateInfo,
        swapchain_size: &Vector2<u32>,
    ) -> Result<Self::Image, MemoryError>;

    /// Creates a new Semaphore.
    fn create_semaphore(&self, start_signalled: bool) -> Result<Self::Semaphore, MemoryError>;

    /// Creates the specified number of Semaphores.
    ///
    /// # Parameters
    ///
    /// * `count` - The number of semaphores to create.
    /// * `start_signalled` - True if all the semaphores should be created in a signalled state, false otherwise
    fn create_semaphores(&self, count: u32, start_signalled: bool) -> Result<Vec<Self::Semaphore>, MemoryError>;

    /// Creates a new fence.
    fn create_fence(&self, start_signalled: bool) -> Result<Self::Fence, MemoryError>;

    /// Creates the specified number of Fences.
    ///
    /// # Parameters
    ///
    /// * `count` - The number of fences to create.
    fn create_fences(&self, count: u32, start_signalled: bool) -> Result<Vec<Self::Fence>, MemoryError>;

    /// Waits for all the provided fences to be signalled.
    ///
    /// # Parameters
    ///
    /// * `fences` - All the fences to wait for.
    fn wait_for_fences(&self, fences: Vec<Self::Fence>);

    /// Resets all the provided fences to an unsignalled state.
    ///
    /// # Parameters
    ///
    /// * `fences` - The fences to reset.
    fn reset_fences(&self, fences: Vec<Self::Fence>);

    /// Executes the provided DescriptorSetWrites on this device.
    ///
    /// # Parameters
    ///
    /// * `updates` - The DescriptorSetWrites to execute.
    fn update_descriptor_sets(&self, updates: Vec<DescriptorSetWrite>);
}

/// Represents a queue of command lists to run.
pub trait Queue {
    /// The queue's command list type.
    type CommandList: CommandList;

    /// The queue's fence type.
    type Fence: Fence;

    /// The queue's semaphore type.
    type Semaphore: Semaphore;

    /// Submits a command list to this queue.
    ///
    /// # Parameters
    ///
    /// * `commands` - The CommandList to submit to this queue.
    /// * `fence_to_signal` - The Fence to signal after the CommandList has finished executing.
    /// * `wait_semaphores` The semaphores to wait for before executing the CommandList.
    /// * `signal_semaphores` - The semaphores to signal when the CommandList has finished executing.
    fn submit_commands(
        &self,
        commands: Self::CommandList,
        fence_to_signal: Self::Fence,
        wait_semaphores: Vec<Self::Semaphore>,
        signal_semaphores: Vec<Self::Semaphore>,
    );
}

/// A block of memory and an allocation strategy.
pub trait Memory {
    /// Memory's underlying buffer type.
    type Buffer: Buffer;

    /// Creates a buffer from this memory.
    ///
    /// It's the caller's responsibility to make sure that this memory is allowed to create buffers.
    ///
    /// # Parameters
    ///
    /// * `data` - The BufferData to create the new buffer from.
    fn create_buffer(&self, data: BufferCreateInfo) -> Result<Self::Buffer, MemoryError>;
}

/// A buffer or texture. Often interchangeable.
pub trait Resource {}

/// A data buffer.
pub trait Buffer {
    /// Maps this buffer so that you can write data directly to it
    ///
    /// This method will fail if the buffer is in device-local memory, or has otherwise been created in a heap with no
    /// CPU access
    fn map(&self) -> Result<*mut (), MappingError>;

    /// Unmaps this buffer
    ///
    /// This method doesn't do anything interesting if the buffer isn't CPU-addressable
    fn unmap(&self);
}

/// An raw image with no sampler.
pub trait Image {}

/// An image sampler.
pub trait Sampler {}

/// A pool of descriptors.
pub trait DescriptorPool {
    /// Descriptor pool's pipeline interface type.
    type PipelineInterface: PipelineInterface;

    /// Descriptor pool's descriptor set type.
    type DescriptorSet: DescriptorSet;

    /// Creates DescriptorSets from the provided PipelineInterface.
    ///
    /// # Parameters
    ///
    /// * `pipeline_interface` - The PipelineInterface to create the descriptors from.
    fn create_descriptor_sets(&self, pipeline_interface: Self::PipelineInterface) -> Vec<Self::DescriptorSet>;
}

/// FIXME(dethraid): docs
pub trait DescriptorSet {}

/// FIXME(dethraid): docs
pub trait Renderpass {}

/// FIXME(dethraid): docs
pub trait Framebuffer {}

/// A swapchain that Nova can render to
///
/// Contains all the framebuffers and images needed!
pub trait Swapchain<'a> {
    type Framebuffer: Framebuffer;
    type Image: Image;
    type Fence: Fence;

    /// Gets the index of the first available swapchain image
    ///
    /// If no swapchain images are available, this method will block until one is available
    fn acquire_next_image(&self) -> u32;

    /// Tells the presents the specified swapchain image to the screen. The swapchain image at the specified index
    /// is unusable until that index is returned from acquire_next_image
    fn present(&self, index: u32);

    /// Borrows the framebuffer that can render to the swapchain image at the specified index
    fn get_framebuffer(&self, index: u32) -> &'a Self::Framebuffer;

    /// Borrows the graphics API's representation of the swapchain image at the specified index
    fn get_image(&self, index: u32) -> &'a Self::Image;

    /// Gets the size, in pixels,  of the swapchain
    fn get_size(&self) -> Vector2<u32>;
}

/// FIXME(dethraid): docs
pub trait PipelineInterface {}

/// FIXME(dethraid): docs
pub trait Pipeline {}

/// FIXME(dethraid): docs
pub trait Semaphore {}

/// Represents a fence in an API-agnostic way
///
/// Fences are used for GPU -> CPU synchronization. Various functions take in fences to signal when an operation is
/// complete, then you can wait on the fence itself to ensure that the GPU is finished with some work before the CPU
/// moves ahead
///
/// Example use case: when you submit a command list to a queue you may pass in a fence. The GPU will signal the fence
/// when the command list has finished executing, so the CPU can wait on the fence to know when it can destroy the
/// resources used by that command list
pub trait Fence {
    /// Waits for this fence to become signalled
    fn wait_for_signal(&self);

    /// Resets this fence from a signalled to an unsignalled state
    fn reset(&self);
}

/// Allocator for command lists.
pub trait CommandAllocator {
    /// Command list type being allocated.
    type CommandList: CommandList;

    /// Allocate a single command list.
    ///
    /// # Parameters
    ///
    /// * `secondary_list` - If the list is a secondary one which can be used from other command lists
    fn create_command_list(&self, secondary_list: bool) -> Result<Self::CommandList, MemoryError>;
}

/// A CommandList is a sequence of commands which can be submitted to the GPU.
pub trait CommandList {
    /// CommandList's buffer type.
    type Buffer: Buffer;
    /// CommandList's sub command list type.
    type CommandList: CommandList;
    /// CommandList's renderpass type.
    type Renderpass: Renderpass;
    /// CommandList's framebuffer type.
    type Framebuffer: Framebuffer;
    /// CommandList's pipeline type.
    type Pipeline: Pipeline;
    /// CommandList's descriptor set type.
    type DescriptorSet: DescriptorSet;
    /// CommandList's pipeline interface type.
    type PipelineInterface: PipelineInterface;

    /// Records resource barriers which happen after all the stages in the `stages_before_barrier`
    /// bitmask, and before all the stages in the `stages_after_barrier` bitmask.
    ///
    /// # Parameters
    ///
    /// * `stages_before_barrier` - The pipeline barrier will take place after all the stages in this bitmask.
    /// * `stages_after_barrier` - The pipeline barrier will take place before all the stages in this bitmask.
    /// * `barriers` - The resource barriers to record.
    fn resource_barriers(
        &self,
        stages_before_barrier: PipelineStageFlags,
        stages_after_barrier: PipelineStageFlags,
        barriers: Vec<ResourceBarrier>,
    );

    /// Records a command to copy data from one buffer to another.
    ///
    /// # Parameters
    ///
    /// * `destination_buffer` - The buffer to write data to.
    /// * `destination_offset` - The number of bytes from the start of `destination_buffer` to write to.
    /// * `source_buffer` - The buffer to read data from.
    /// * `source_offset` - The number of bytes from the start of `source_buffer` to read data from.
    /// * `num_bytes` - The number of bytes to copy.
    fn copy_buffer(
        &self,
        destination_buffer: Self::Buffer,
        destination_offset: u64,
        source_buffer: Self::Buffer,
        source_offset: u64,
        num_bytes: u64,
    );

    /// Records a command to execute the provided command lists.
    ///
    /// # Parameters
    ///
    /// * `lists` - The command lists to execute
    fn execute_command_lists(&self, lists: Vec<Self::CommandList>);

    /// Records a command to begin a renderpass with a framebuffer.
    ///
    /// # Parameters
    ///
    /// * `renderpass` - The renderpass to begin
    /// * `framebuffer` - The framebuffer to begin the renderpass with
    fn begin_renderpass(&self, renderpass: Self::Renderpass, framebuffer: Self::Framebuffer);

    /// Records a command to end the current renderpass
    fn end_renderpass(&self);

    /// Binds a pipeline to the command list.
    ///
    /// # Parameters
    ///
    /// * `pipeline` - The pipeline to bind
    fn bind_pipeline(&self, pipeline: Self::Pipeline);

    /// Records a command to bind DescriptorSet to a PipelineInterface.
    ///
    /// # Parameters
    ///
    /// * `descriptor_sets` - The DescriptorSets to bind
    /// * `pipeline_interface` - The PipelineInterface to bind the descriptor sets to
    fn bind_descriptor_sets(
        &self,
        descriptor_sets: Vec<Self::DescriptorSet>,
        pipeline_interface: Self::PipelineInterface,
    );

    /// Records a command to bind vertex buffers.
    ///
    /// Vertex buffers are always bound sequentially starting at binding 0.
    ///
    /// # Parameters
    ///
    /// * `buffers` - The buffers to bind
    fn bind_vertex_buffers(&self, buffers: Vec<Self::Buffer>);

    /// Binds an index buffer.
    ///
    /// # Parameters
    ///
    /// * `buffer` - The buffer to bind as an index buffer
    fn bind_index_buffer(&self, buffer: Self::Buffer);

    /// Records a drawcall to grab `num_indices` indices from the currently bound index buffer and
    /// draw them `num_instances` times.
    ///
    /// # Parameters
    ///
    /// * `num_indices` - The number of indices to draw from the currently bound index buffer
    /// * `num_instances` - How many times to draw the mesh
    fn draw_indexed_mesh(&self, num_indices: u32, num_instances: u32);
}
