use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::dx12::dx12_system_info::Dx12SystemInfo;
use crate::{
    rhi::{
        dx12::{
            dx12_command_allocator::Dx12CommandAllocator, dx12_descriptor_pool::Dx12DescriptorPool,
            dx12_fence::Dx12Fence, dx12_framebuffer::Dx12Framebuffer, dx12_image::Dx12Image, dx12_memory::Dx12Memory,
            dx12_physical_device::Dx12PhysicalDevice, dx12_pipeline::Dx12Pipeline,
            dx12_pipeline_interface::Dx12PipelineInterface, dx12_queue::Dx12Queue, dx12_renderpass::Dx12Renderpass,
            dx12_semaphore::Dx12Semaphore,
        },
        AllocationError, CommandAllocatorCreateInfo, DescriptorPoolCreationError, DescriptorSetWrite, Device,
        MemoryError, MemoryUsage, ObjectType, PhysicalDevice, PipelineCreationError, QueueGettingError, QueueType,
        ResourceBindingDescription,
    },
    shaderpack,
};
use cgmath::Vector2;
use core::mem;
use std::collections::HashMap;
use std::rc::Rc;
use winapi::Interface;
use winapi::{shared::winerror, um::d3d12::*};

pub struct Dx12Device {
    /// Graphics adapter that we're using
    phys_device: Rc<Dx12PhysicalDevice>,

    /// D3D12 device that we're wrapping
    device: WeakPtr<ID3D12Device>,

    /// Increment size of an RTV descriptor
    rtv_descriptor_size: u32,

    /// Increment size of a CBV, UAV, or SRV descriptor
    shader_resource_descriptor_size: u32,

    /// Various information about the system we're running on
    system_info: Dx12SystemInfo,
}

impl Dx12Device {
    pub fn new(phys_device: Rc<Dx12PhysicalDevice>, device: WeakPtr<ID3D12Device>) -> Self {
        let rtv_descriptor_size = device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);
        let shader_resource_descriptor_size =
            device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV);

        Dx12Device {
            phys_device,
            device,
            rtv_descriptor_size,
            shader_resource_descriptor_size,
            system_info: Dx12SystemInfo { supported_version: 4 },
        }
    }

    pub fn get_api_version(&self) -> u32 {
        self.system_info.supported_version
    }
}

impl Device for Dx12Device {
    type Queue = Dx12Queue;
    type Memory = Dx12Memory;
    type CommandAllocator = Dx12CommandAllocator;
    type Image = Dx12Image;
    type Renderpass = Dx12Renderpass;
    type Framebuffer = Dx12Framebuffer;
    type PipelineInterface = Dx12PipelineInterface;
    type DescriptorPool = Dx12DescriptorPool;
    type Pipeline = Dx12Pipeline;
    type Semaphore = Dx12Semaphore;
    type Fence = Dx12Fence;

    fn get_queue(&self, queue_type: QueueType, queue_index: u32) -> Result<Dx12Queue, QueueGettingError> {
        let queue_type = match queue_type {
            QueueType::Graphics => D3D12_COMMAND_LIST_TYPE_DIRECT,
            QueueType::Compute => D3D12_COMMAND_LIST_TYPE_COMPUTE,
            QueueType::Copy => D3D12_COMMAND_LIST_TYPE_COPY,
        };

        let mut queue = WeakPtr::<ID3D12CommandQueue>::null();
        let queue_desc = D3D12_COMMAND_QUEUE_DESC {
            Type: queue_type,
            Priority: D3D12_COMMAND_QUEUE_PRIORITY_NORMAL as _,
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            NodeMask: 0,
        };

        let hr = unsafe {
            self.device
                .CreateCommandQueue(&queue_desc, ID3D12CommandQueue::uuidof(), queue.mut_void())
        };
        if winerror::SUCCEEDED(hr) {
            Ok(Dx12Queue::new(queue))
        } else {
            Err(QueueGettingError::OutOfMemory)
        }
    }

    fn allocate_memory(
        &self,
        size: u64,
        memory_usage: MemoryUsage,
        allowed_objects: ObjectType,
    ) -> Result<Dx12Memory, AllocationError> {
        let heap_properties = match memory_usage {
            MemoryUsage::DeviceOnly => D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_DEFAULT,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_NOT_AVAILABLE,
                MemoryPoolPreference: D3D12_MEMORY_POOL_L1,
                CreationNodeMask: 0,
                VisibleNodeMask: 0,
            },
            MemoryUsage::LowFrequencyUpload => D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_UPLOAD,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_WRITE_COMBINE,
                MemoryPoolPreference: D3D12_MEMORY_POOL_L1,
                CreationNodeMask: 0,
                VisibleNodeMask: 0,
            },
            MemoryUsage::StagingBuffer => D3D12_HEAP_PROPERTIES {
                Type: D3D12_HEAP_TYPE_UPLOAD,
                CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_WRITE_COMBINE,
                MemoryPoolPreference: D3D12_MEMORY_POOL_L0,
                CreationNodeMask: 0,
                VisibleNodeMask: 0,
            },
        };

        let heap_flags = match allowed_objects {
            ObjectType::Buffer => D3D12_HEAP_FLAG_ALLOW_ONLY_BUFFERS,
            ObjectType::Texture => D3D12_HEAP_FLAG_ALLOW_ONLY_NON_RT_DS_TEXTURES,
            ObjectType::Attachment => D3D12_HEAP_FLAG_ALLOW_ONLY_RT_DS_TEXTURES,
            ObjectType::SwapchainSurface => D3D12_HEAP_FLAG_ALLOW_ONLY_RT_DS_TEXTURES | D3D12_HEAP_FLAG_ALLOW_DISPLAY,
            ObjectType::Any => D3D12_HEAP_FLAG_ALLOW_ALL_BUFFERS_AND_TEXTURES,
        };

        // Ensure we have enough free memory for the requested allocation
        let free_memory = self.phys_device.get_free_memory();
        if free_memory < size {
            if memory_usage == MemoryUsage::StagingBuffer {
                Err(AllocationError::OutOfHostMemory)
            } else {
                Err(AllocationError::OutOfDeviceMemory)
            }
        } else {
            let mut heap = WeakPtr::<ID3D12Heap>::null();
            let heap_create_info = D3D12_HEAP_DESC {
                SizeInBytes: size,
                Properties: heap_properties,
                Alignment: 64,
                Flags: heap_flags,
            };

            let hr = unsafe {
                self.device
                    .CreateHeap(&heap_create_info, ID3D12Heap::uuidof(), heap.mut_void())
            };
            if winerror::SUCCEEDED(hr) {
                Ok(Dx12Memory::new(heap, size))
            } else {
                if memory_usage == MemoryUsage::StagingBuffer {
                    Err(AllocationError::OutOfHostMemory)
                } else {
                    Err(AllocationError::OutOfDeviceMemory)
                }
            }
        }
    }

    fn create_command_allocator(
        &self,
        create_info: CommandAllocatorCreateInfo,
    ) -> Result<Dx12CommandAllocator, MemoryError> {
        let command_allocator_type = match create_info.command_list_type {
            QueueType::Graphics => D3D12_COMMAND_LIST_TYPE_DIRECT,
            QueueType::Compute => D3D12_COMMAND_LIST_TYPE_COMPUTE,
            QueueType::Copy => D3D12_COMMAND_LIST_TYPE_COPY,
        };

        let mut allocator = WeakPtr::<ID3D12CommandAllocator>::null();
        let hr = unsafe {
            self.device.CreateCommandAllocator(
                command_allocator_type,
                ID3D12CommandAllocator::uuidof(),
                allocator.mut_void(),
            )
        };
        if winerror::SUCCEEDED(hr) {
            Ok(Dx12CommandAllocator::new(allocator))
        } else {
            Err(MemoryError::OutOfHostMemory)
        }
    }

    fn create_renderpass(&self, data: shaderpack::RenderPassCreationInfo) -> Result<Dx12Renderpass, MemoryError> {
        let mut render_target_descs = Vec::<D3D12_RENDER_PASS_RENDER_TARGET_DESC>::new();
        for attachment_info in data.texture_outputs {
            let (beginning_access_type, ending_access_type) = match attachment_info.name.as_ref() {
                "Backbuffer" => (
                    D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_CLEAR,
                    D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_PRESERVE,
                ),
                _ => match attachment_info.clear {
                    true => (
                        D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_CLEAR,
                        D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_PRESERVE,
                    ),
                    false => (
                        D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_PRESERVE,
                        D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_PRESERVE,
                    ),
                },
            };

            let mut beginning_access = D3D12_RENDER_PASS_BEGINNING_ACCESS {
                Type: beginning_access_type,
                ..unsafe { mem::zeroed() }
            };

            if beginning_access_type == D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_CLEAR {
                let mut clear_value = D3D12_CLEAR_VALUE {
                    Format: 0,
                    ..unsafe { mem::zeroed() }
                };

                *unsafe { clear_value.u.Color_mut() } = [0.into(), 0.into(), 0.into(), 0.into()];

                *unsafe { beginning_access.u.Clear_mut() } = D3D12_RENDER_PASS_BEGINNING_ACCESS_CLEAR_PARAMETERS {
                    ClearValue: clear_value,
                };
            }

            let mut ending_access = D3D12_RENDER_PASS_ENDING_ACCESS {
                Type: ending_access_type,
                ..unsafe { mem::zeroed() }
            };

            // TODO: Handle D3D12_RENDER_PASS_ENDING_ACCESS_TYPE_RESOLVE when we actually support MSAA in a meaningful
            // capacity

            let render_target_desc = D3D12_RENDER_PASS_RENDER_TARGET_DESC {
                cpuDescriptor: D3D12_CPU_DESCRIPTOR_HANDLE {},
                BeginningAccess: beginning_access,
                EndingAccess: ending_access,
            };

            render_target_descs.push(render_target_desc);
        }

        let depth_stencil_desc = data
            .depth_texture
            .map(|depth_info| D3D12_RENDER_PASS_DEPTH_STENCIL_DESC {
                cpuDescriptor: D3D12_CPU_DESCRIPTOR_HANDLE {},
                DepthBeginningAccess: D3D12_RENDER_PASS_BEGINNING_ACCESS {},
                StencilBeginningAccess: D3D12_RENDER_PASS_BEGINNING_ACCESS {},
                DepthEndingAccess: D3D12_RENDER_PASS_ENDING_ACCESS {},
                StencilEndingAccess: D3D12_RENDER_PASS_ENDING_ACCESS {},
            });

        Ok(Dx12Renderpass::new(render_target_descs, depth_stencil_desc))
    }

    fn create_framebuffer(
        &self,
        renderpass: Dx12Renderpass,
        attachments: Vec<Dx12Image>,
        framebuffer_size: Vector2<f32>,
    ) -> Result<Dx12Framebuffer, MemoryError> {
        unimplemented!()
    }

    fn create_pipeline_interface(
        &self,
        bindings: &HashMap<String, ResourceBindingDescription>,
        color_attachments: &[shaderpack::TextureAttachmentInfo],
        depth_texture: &Option<shaderpack::TextureAttachmentInfo>,
    ) -> Result<Dx12PipelineInterface, MemoryError> {
        unimplemented!()
    }

    fn create_descriptor_pool(
        &self,
        num_sampled_images: u32,
        num_samplers: u32,
        num_uniform_buffers: u32,
    ) -> Result<Vec<Dx12DescriptorPool>, DescriptorPoolCreationError> {
        unimplemented!()
    }

    fn create_pipeline(
        &self,
        pipeline_interface: Dx12PipelineInterface,
        data: shaderpack::PipelineCreationInfo,
    ) -> Result<Dx12Pipeline, PipelineCreationError> {
        unimplemented!()
    }

    fn create_image(&self, data: shaderpack::TextureCreateInfo) -> Result<Dx12Image, MemoryError> {
        unimplemented!()
    }

    fn create_semaphore(&self) -> Result<Dx12Semaphore, MemoryError> {
        unimplemented!()
    }

    fn create_semaphores(&self, count: u32) -> Result<Vec<Dx12Semaphore>, MemoryError> {
        unimplemented!()
    }

    fn create_fence(&self) -> Result<Dx12Fence, MemoryError> {
        unimplemented!()
    }

    fn create_fences(&self, count: u32) -> Result<Vec<Dx12Fence>, MemoryError> {
        unimplemented!()
    }

    fn wait_for_fences(&self, fences: Vec<Dx12Fence>) {
        unimplemented!()
    }

    fn reset_fences(&self, fences: Vec<Dx12Fence>) {
        unimplemented!()
    }

    fn update_descriptor_sets(&self, updates: Vec<DescriptorSetWrite>) {
        unimplemented!()
    }
}
