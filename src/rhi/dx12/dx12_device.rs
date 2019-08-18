#![allow(unsafe_code)]

use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::dx12::dx12_renderpass::Dx12RenderPassAccessInfo;
use crate::rhi::dx12::dx12_system_info::Dx12SystemInfo;
use crate::rhi::{DeviceCreationError, DeviceProperties};
use crate::{
    rhi::{
        dx12::{
            dx12_command_allocator::Dx12CommandAllocator, dx12_descriptor_pool::Dx12DescriptorPool,
            dx12_fence::Dx12Fence, dx12_framebuffer::Dx12Framebuffer, dx12_image::Dx12Image, dx12_memory::Dx12Memory,
            dx12_pipeline::Dx12Pipeline, dx12_pipeline_interface::Dx12PipelineInterface, dx12_queue::Dx12Queue,
            dx12_renderpass::Dx12Renderpass, dx12_semaphore::Dx12Semaphore,
        },
        AllocationError, CommandAllocatorCreateInfo, DescriptorPoolCreationError, DescriptorSetWrite, Device,
        MemoryError, MemoryUsage, ObjectType, PipelineCreationError, QueueGettingError, QueueType,
        ResourceBindingDescription,
    },
    shaderpack,
};
use cgmath::Vector2;
use core::mem;
use spirv_cross::hlsl;
use std::collections::HashMap;
use winapi::shared::dxgi1_2::IDXGIAdapter2;
use winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_SNORM;
use winapi::shared::winerror::{FAILED, SUCCEEDED};
use winapi::um::d3dcommon::{ID3DBlob, D3D_FEATURE_LEVEL_11_0};
use winapi::Interface;
use winapi::{shared::winerror, um::d3d12::*};

#[macro_use]
use log::*;
use crate::rhi::dx12::dx12_utils::{compile_shader, to_dx12_range_type};
use spirv_cross::ErrorCode;
use std::ptr::null;

pub struct Dx12Device {
    /// Graphics adapter that we're using
    adapter: WeakPtr<IDXGIAdapter2>,

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
    pub fn new(adapter: WeakPtr<IDXGIAdapter2>) -> Option<Self> {
        let device_result = unsafe {
            let mut device = WeakPtr::<ID3D12Device>::null();
            // TODO: Figure out how to determine which SDK version the system we're running on supports
            let hr = D3D12CreateDevice(
                adapter.as_unknown() as *const _ as *mut _,
                D3D_FEATURE_LEVEL_11_0,
                &ID3D12Device::uuidof(),
                device.mut_void(),
            );
            if winerror::SUCCEEDED(hr) {
                Ok(device)
            } else {
                Err(DeviceCreationError::Failed)
            }
        };

        match device_result {
            Ok(device) => {
                let rtv_descriptor_size =
                    unsafe { device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV) };
                let shader_resource_descriptor_size =
                    unsafe { device.GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV) };

                Some(Dx12Device {
                    adapter,
                    device,
                    rtv_descriptor_size,
                    shader_resource_descriptor_size,
                    system_info: Dx12SystemInfo { supported_version: 4 },
                })
            }
            Err(_) => None,
        }
    }

    pub fn get_api_version(&self) -> u32 {
        self.system_info.supported_version
    }

    fn handle_shader_compile_error(name: &str, e: ErrorCode) -> Result<Dx12Pipeline, PipelineCreationError> {
        match e {
            ErrorCode::Unhandled => Err(PipelineCreationError::InvalidShader),
            ErrorCode::CompilationError(str) => {
                warn!("Could not compile shader for {} because {}", name, str);
                Err(PipelineCreationError::InvalidShader)
            }
        }
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

    fn get_properties(&self) -> DeviceProperties {
        unimplemented!()
    }

    fn can_be_used_by_nova(&self) -> bool {
        unimplemented!()
    }

    fn get_free_memory(&self) -> u64 {
        unimplemented!()
    }

    fn get_queue(&self, queue_type: QueueType, _queue_index: u32) -> Result<Dx12Queue, QueueGettingError> {
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
                .CreateCommandQueue(&queue_desc, &ID3D12CommandQueue::uuidof(), queue.mut_void())
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
        let free_memory = self.get_free_memory();
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
                    .CreateHeap(&heap_create_info, &ID3D12Heap::uuidof(), heap.mut_void())
            };
            if winerror::SUCCEEDED(hr) {
                Ok(Dx12Memory::new(heap, size))
            } else if memory_usage == MemoryUsage::StagingBuffer {
                Err(AllocationError::OutOfHostMemory)
            } else {
                Err(AllocationError::OutOfDeviceMemory)
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
                &ID3D12CommandAllocator::uuidof(),
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
        let mut render_target_descs = Vec::<Dx12RenderPassAccessInfo>::new();
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
                    Format: DXGI_FORMAT_R8G8B8A8_SNORM,
                    ..unsafe { mem::zeroed() }
                };

                *unsafe { clear_value.u.Color_mut() } = [0_f32, 0_f32, 0_f32, 0_f32];

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

            let render_target_desc = Dx12RenderPassAccessInfo {
                beginning_access,
                ending_access,
            };

            render_target_descs.push(render_target_desc);
        }

        let depth_stencil_desc = data.depth_texture.map(|depth_info| {
            let depth_beginning_access_type = match depth_info.clear {
                true => D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_CLEAR,
                false => D3D12_RENDER_PASS_BEGINNING_ACCESS_TYPE_PRESERVE,
            };

            let mut depth_beginning_access = D3D12_RENDER_PASS_BEGINNING_ACCESS {
                Type: depth_beginning_access_type,
                ..unsafe { mem::zeroed() }
            };

            let clear_value = D3D12_CLEAR_VALUE {
                Format: DXGI_FORMAT_R8G8B8A8_SNORM,
                ..unsafe { mem::zeroed() }
            };

            *unsafe { depth_beginning_access.u.Clear_mut() } = D3D12_RENDER_PASS_BEGINNING_ACCESS_CLEAR_PARAMETERS {
                ClearValue: clear_value,
            };

            let depth_ending_access = D3D12_RENDER_PASS_ENDING_ACCESS {
                Type: D3D12_RENDER_PASS_ENDING_ACCESS_TYPE_PRESERVE,
                ..unsafe { mem::zeroed() }
            };

            Dx12RenderPassAccessInfo {
                beginning_access: depth_beginning_access,
                ending_access: depth_ending_access,
            }
        });

        Ok(Dx12Renderpass::new(render_target_descs, depth_stencil_desc))
    }

    fn create_framebuffer(
        &self,
        renderpass: Dx12Renderpass,
        attachments: Vec<Dx12Image>,
        _: Vector2<f32>,
    ) -> Result<Dx12Framebuffer, MemoryError> {
        let num_rtv_descriptors = match renderpass.depth_stencil {
            Some(_) => renderpass.render_targets.len() + 1,
            None => renderpass.render_targets.len(),
        } as u32;

        let rtv_descriptor_heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
            NumDescriptors: num_rtv_descriptors,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
            NodeMask: 0,
        };

        let mut heap = WeakPtr::<ID3D12DescriptorHeap>::null();
        let hr = unsafe {
            self.device.CreateDescriptorHeap(
                &rtv_descriptor_heap_desc,
                &ID3D12DescriptorHeap::uuidof(),
                heap.mut_void(),
            )
        };
        if SUCCEEDED(hr) {
            let base_descriptor = unsafe { heap.GetCPUDescriptorHandleForHeapStart() };

            let mut color_attachment_descriptors = Vec::<D3D12_CPU_DESCRIPTOR_HANDLE>::new();
            for (i, _) in attachments.iter().enumerate() {
                color_attachment_descriptors.push(D3D12_CPU_DESCRIPTOR_HANDLE {
                    ptr: base_descriptor.ptr + i,
                });
            }

            let depth_attachment_descriptor = match renderpass.depth_stencil {
                Some(_) => Some(D3D12_CPU_DESCRIPTOR_HANDLE {
                    ptr: base_descriptor.ptr + color_attachment_descriptors.len() + 1,
                }),
                None => None,
            };

            Ok(Dx12Framebuffer {
                color_attachments: color_attachment_descriptors,
                depth_attachment: depth_attachment_descriptor,
                descriptor_heap: heap,
            })
        } else {
            Err(MemoryError::OutOfHostMemory)
        }
    }

    fn create_pipeline_interface(
        &self,
        bindings: &HashMap<String, ResourceBindingDescription>,
        color_attachments: &[shaderpack::TextureAttachmentInfo],
        depth_texture: &Option<shaderpack::TextureAttachmentInfo>,
    ) -> Result<Dx12PipelineInterface, MemoryError> {
        let mut table_layouts: HashMap<u32, Vec<ResourceBindingDescription>> =
            HashMap::<u32, Vec<ResourceBindingDescription>>::new();

        for binding in bindings.values() {
            match table_layouts.get_mut(&binding.set) {
                Some(table_bindings) => table_bindings.push(binding.clone()),
                None => {
                    table_layouts.insert(binding.set, vec![binding.clone()]);
                }
            }
        }

        let num_sets = table_layouts.len();

        let mut root_signature_params = Vec::<D3D12_ROOT_PARAMETER>::new();

        for set in 0..num_sets {
            let descriptor_layouts_opt = table_layouts.get(&(set as u32));
            if descriptor_layouts_opt.is_none() {
                warn!(
                    "No descriptors in set {}, but there should. Each pipeline _must_ use contiguous descriptor sets",
                    set
                );
                continue;
            }

            let mut param = D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
                ..unsafe { mem::zeroed() }
            };

            let descriptor_layouts = descriptor_layouts_opt.unwrap();

            let mut descriptor_ranges = Vec::<D3D12_DESCRIPTOR_RANGE>::new();
            for layout in descriptor_layouts {
                let descriptor_range = D3D12_DESCRIPTOR_RANGE {
                    RangeType: to_dx12_range_type(&layout.descriptor_type),
                    NumDescriptors: layout.count,
                    BaseShaderRegister: layout.binding,
                    RegisterSpace: 0,
                    OffsetInDescriptorsFromTableStart: D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
                };

                descriptor_ranges.push(descriptor_range);
            }

            let descriptor_table_param = D3D12_ROOT_DESCRIPTOR_TABLE {
                NumDescriptorRanges: descriptor_ranges.len() as u32,
                pDescriptorRanges: descriptor_ranges.as_ptr(),
            };

            *unsafe { param.u.DescriptorTable_mut() } = descriptor_table_param;

            root_signature_params.push(param);
        }

        let root_signature = D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: root_signature_params.len() as _,
            pParameters: root_signature_params.as_ptr(),
            NumStaticSamplers: 0,
            pStaticSamplers: null(),
            Flags: 0,
        };

        let mut root_sig_blob = WeakPtr::<ID3DBlob>::null();
        let mut root_sig_error_blob = WeakPtr::<ID3DBlob>::null();
        let hr = unsafe {
            D3D12SerializeRootSignature(
                &root_signature,
                D3D_ROOT_SIGNATURE_VERSION_1_0,
                root_sig_blob.mut_void() as *mut *mut _,
                root_sig_error_blob.mut_void() as *mut *mut _,
            )
        };

        if SUCCEEDED(hr) {
            let mut root_sig = WeakPtr::<ID3D12RootSignature>::null();
            let hr = unsafe {
                self.device.CreateRootSignature(
                    0,
                    root_sig_blob.GetBufferPointer(),
                    root_sig_blob.GetBufferSize(),
                    &ID3D12RootSignature::uuidof(),
                    root_sig.mut_void(),
                )
            };
            if SUCCEEDED(hr) {
                let pipeline_interface = Dx12PipelineInterface {
                    root_sig,
                    color_attachments: color_attachments.to_vec(),
                    depth_texture: depth_texture.clone(),
                };

                return Ok(pipeline_interface);
            }
        }

        Err(MemoryError::OutOfHostMemory)
    }

    fn create_descriptor_pool(
        &self,
        num_sampled_images: u32,
        num_samplers: u32,
        num_uniform_buffers: u32,
    ) -> Result<Dx12DescriptorPool, DescriptorPoolCreationError> {
        let sampler_heap_desc = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER,
            NumDescriptors: num_samplers,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
            NodeMask: 0,
        };

        let mut sampler_heap = WeakPtr::<ID3D12DescriptorHeap>::null();
        let hr = unsafe {
            self.device.CreateDescriptorHeap(
                &sampler_heap_desc,
                &ID3D12DescriptorHeap::uuidof(),
                sampler_heap.mut_void(),
            )
        };
        if FAILED(hr) {
            match hr {
                winerror::E_OUTOFMEMORY => return Err(DescriptorPoolCreationError::OutOfHostMemory),
                _ => return Err(DescriptorPoolCreationError::Fragmentation),
            }
        }

        let cbv_srv_uav_descriptor_heap = D3D12_DESCRIPTOR_HEAP_DESC {
            Type: D3D12_DESCRIPTOR_HEAP_TYPE_SAMPLER,
            NumDescriptors: num_sampled_images + num_uniform_buffers,
            Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
            NodeMask: 0,
        };

        let mut data_heap = WeakPtr::<ID3D12DescriptorHeap>::null();
        let hr = unsafe {
            self.device.CreateDescriptorHeap(
                &cbv_srv_uav_descriptor_heap,
                &ID3D12DescriptorHeap::uuidof(),
                data_heap.mut_void(),
            )
        };
        if FAILED(hr) {
            match hr {
                E_OUTOFMEMORY => return Err(DescriptorPoolCreationError::OutOfHostMemory),
                _ => return Err(DescriptorPoolCreationError::Fragmentation),
            }
        }

        Ok(Dx12DescriptorPool {
            sampler_heap,
            data_heap,
        })
    }

    fn create_pipeline(
        &self,
        pipeline_interface: Dx12PipelineInterface,
        data: shaderpack::PipelineCreationInfo,
    ) -> Result<Dx12Pipeline, PipelineCreationError> {
        let mut pso_desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
            ..unsafe { mem::zeroed() }
        };

        let mut shader_inputs = HashMap::<u32, Vec<D3D12_DESCRIPTOR_RANGE1>>::new();
        let spv_cross_options = hlsl::CompilerOptions {
            shader_model: hlsl::ShaderModel::V5_1, // TODO: Check if this is the DX12 needs
            point_size_compat: false,
            point_coord_compat: false,
            vertex: hlsl::CompilerVertexOptions {
                invert_y: true, // TODO: check if this is correct
                transform_clip_space: false,
            },
        };

        // Shader compilation
        {
            match compile_shader(data.vertex_shader, "vs_5_1", spv_cross_options, &mut shader_inputs) {
                Ok(blob) => {
                    pso_desc.VS.BytecodeLength = blob.GetBufferSize();
                    pso_desc.VS.pShaderBytecode = blob.GetBufferPointer();
                }
                Err(e) => return Dx12Device::handle_shader_compile_error(&data.name, e),
            };

            if let Some(geo) = data.geometry_shader {
                match compile_shader(geo, "gs_5_1", spv_cross_options, &mut shader_inputs) {
                    Ok(blob) => {
                        pso_desc.GS.BytecodeLength = blob.GetBufferSize();
                        pso_desc.GS.pShaderBytecode = blob.GetBufferPointer();
                    }
                    Err(e) => return Dx12Device::handle_shader_compile_error(&data.name, e),
                };
            }

            if let Some(tesc) = data.tessellation_control_shader {
                match compile_shader(tesc, "hs_5_1", spv_cross_options, &mut shader_inputs) {
                    Ok(blob) => {
                        pso_desc.HS.BytecodeLength = blob.GetBufferSize();
                        pso_desc.HS.pShaderBytecode = blob.GetBufferPointer();
                    }
                    Err(e) => return Dx12Device::handle_shader_compile_error(&data.name, e),
                };
            }

            if let Some(tese) = data.tessellation_evaluation_shader {
                match compile_shader(tese, "ds_5_1", spv_cross_options, &mut shader_inputs) {
                    Ok(blob) => {
                        pso_desc.DS.BytecodeLength = blob.GetBufferSize();
                        pso_desc.DS.pShaderBytecode = blob.GetBufferPointer();
                    }
                    Err(e) => return Dx12Device::handle_shader_compile_error(&data.name, e),
                };
            }

            if let Some(frag) = data.fragment_shader {
                match compile_shader(frag, "ps_5_1", spv_cross_options, &mut shader_inputs) {
                    Ok(blob) => {
                        pso_desc.PS.BytecodeLength = blob.GetBufferSize();
                        pso_desc.PS.pShaderBytecode = blob.GetBufferPointer();
                    }
                    Err(e) => return Dx12Device::handle_shader_compile_error(&data.name, e),
                };
            }
        }

        Err(PipelineCreationError::InvalidShader)
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
