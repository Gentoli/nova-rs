#![allow(unsafe_code)]

#[macro_use]
use log::*;

use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::dx12::dx12_command_allocator::Dx12CommandAllocator;
use crate::rhi::dx12::dx12_descriptor_pool::Dx12DescriptorPool;
use crate::rhi::dx12::dx12_fence::Dx12Fence;
use crate::rhi::dx12::dx12_framebuffer::Dx12Framebuffer;
use crate::rhi::dx12::dx12_image::Dx12Image;
use crate::rhi::dx12::dx12_memory::Dx12Memory;
use crate::rhi::dx12::dx12_pipeline::Dx12Pipeline;
use crate::rhi::dx12::dx12_pipeline_interface::Dx12PipelineInterface;
use crate::rhi::dx12::dx12_queue::Dx12Queue;
use crate::rhi::dx12::dx12_renderpass::Dx12RenderPassAccessInfo;
use crate::rhi::dx12::dx12_renderpass::Dx12Renderpass;
use crate::rhi::dx12::dx12_semaphore::Dx12Semaphore;
use crate::rhi::dx12::dx12_system_info::Dx12SystemInfo;
use crate::rhi::dx12::dx12_utils::{compile_shader, to_dx12_range_type, to_dx12_topology, to_dxgi_format};
use crate::rhi::dx12::get_uuid;
use crate::rhi::dx12::pso_utils::{
    get_input_descriptions, make_depth_stencil_state, make_rasterizer_desc, make_render_target_blend_desc,
};
use crate::rhi::{
    AllocationError, CommandAllocatorCreateInfo, DescriptorPoolCreationError, DescriptorSetWrite, DescriptorUpdateInfo,
    Device, DeviceCreationError, DeviceProperties, Fence, MemoryError, MemoryUsage, ObjectType, PipelineCreationError,
    QueueGettingError, QueueType, ResourceBindingDescription,
};
use crate::shaderpack;
use cgmath::Vector2;
use core::mem;
use spirv_cross::hlsl;
use std::collections::HashMap;
use std::ptr::null;
use winapi::shared::dxgi1_2::IDXGIAdapter2;
use winapi::shared::dxgiformat::DXGI_FORMAT_R8G8B8A8_SNORM;
use winapi::shared::dxgitype::DXGI_SAMPLE_DESC;
use winapi::shared::winerror::{E_OUTOFMEMORY, FAILED, SUCCEEDED};
use winapi::um::d3d12::*;
use winapi::um::d3dcommon::{ID3DBlob, D3D_FEATURE_LEVEL_11_0};
use winapi::um::synchapi::{CreateEventA, WaitForSingleObject};
use winapi::Interface;

const CPU_FENCE_SIGNALED: i32 = 16;
const GPU_FENCE_SIGNALED: i32 = 32;

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
                get_uuid(device),
                device.mut_void(),
            );
            if SUCCEEDED(hr) {
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
                .CreateCommandQueue(&queue_desc, get_uuid(queue), queue.mut_void())
        };
        if SUCCEEDED(hr) {
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
                    .CreateHeap(&heap_create_info, get_uuid(heap), heap.mut_void())
            };
            if SUCCEEDED(hr) {
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
            self.device
                .CreateCommandAllocator(command_allocator_type, get_uuid(allocator), allocator.mut_void())
        };
        if SUCCEEDED(hr) {
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
            self.device
                .CreateDescriptorHeap(&rtv_descriptor_heap_desc, get_uuid(heap), heap.mut_void())
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
                    get_uuid(root_sig),
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
            self.device
                .CreateDescriptorHeap(&sampler_heap_desc, get_uuid(sampler_heap), sampler_heap.mut_void())
        };
        if FAILED(hr) {
            match hr {
                E_OUTOFMEMORY => return Err(DescriptorPoolCreationError::OutOfHostMemory),
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
            self.device
                .CreateDescriptorHeap(&cbv_srv_uav_descriptor_heap, get_uuid(data_heap), data_heap.mut_void())
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

        // Shaders
        {
            match compile_shader(data.vertex_shader, "vs_5_1", spv_cross_options, &mut shader_inputs) {
                Ok(blob) => {
                    pso_desc.VS.BytecodeLength = blob.GetBufferSize();
                    pso_desc.VS.pShaderBytecode = blob.GetBufferPointer();
                }
                Err((err, msg)) => return Err(PipelineCreationError::InvalidShader(msg)),
            };

            if let Some(geo) = data.geometry_shader {
                match compile_shader(geo, "gs_5_1", spv_cross_options, &mut shader_inputs) {
                    Ok(blob) => {
                        pso_desc.GS.BytecodeLength = blob.GetBufferSize();
                        pso_desc.GS.pShaderBytecode = blob.GetBufferPointer();
                    }
                    Err((err, msg)) => return Err(PipelineCreationError::InvalidShader(msg)),
                };
            }

            if let Some(tesc) = data.tessellation_control_shader {
                match compile_shader(tesc, "hs_5_1", spv_cross_options, &mut shader_inputs) {
                    Ok(blob) => {
                        pso_desc.HS.BytecodeLength = blob.GetBufferSize();
                        pso_desc.HS.pShaderBytecode = blob.GetBufferPointer();
                    }
                    Err((err, msg)) => return Err(PipelineCreationError::InvalidShader(msg)),
                };
            }

            if let Some(tese) = data.tessellation_evaluation_shader {
                match compile_shader(tese, "ds_5_1", spv_cross_options, &mut shader_inputs) {
                    Ok(blob) => {
                        pso_desc.DS.BytecodeLength = blob.GetBufferSize();
                        pso_desc.DS.pShaderBytecode = blob.GetBufferPointer();
                    }
                    Err((err, msg)) => return Err(PipelineCreationError::InvalidShader(msg)),
                };
            }

            if let Some(frag) = data.fragment_shader {
                match compile_shader(frag, "ps_5_1", spv_cross_options, &mut shader_inputs) {
                    Ok(blob) => {
                        pso_desc.PS.BytecodeLength = blob.GetBufferSize();
                        pso_desc.PS.pShaderBytecode = blob.GetBufferPointer();
                    }
                    Err((err, msg)) => return Err(PipelineCreationError::InvalidShader(msg)),
                };
            }

            pso_desc.pRootSignature = pipeline_interface.root_sig.as_mut_ptr();
        }

        // Blending
        {
            pso_desc.BlendState.AlphaToCoverageEnable =
                data.states
                    .contains(&shaderpack::RasterizerState::EnableAlphaToCoverage) as i32;

            pso_desc.BlendState.IndependentBlendEnable = false as i32;

            pso_desc.BlendState.RenderTarget[0] = make_render_target_blend_desc(&data);

            pso_desc.SampleMask = 0xFFFF_FFFF;
        }

        // Rasterizer state
        {
            pso_desc.RasterizerState = make_rasterizer_desc(&data);
        }

        // Depth/Stencil state
        {
            pso_desc.DepthStencilState = make_depth_stencil_state(&data);
        }

        // Input Assembler state
        {
            // TODO: Get the pipeline inputs from the pipeline data
            let input_descs = get_input_descriptions();
            pso_desc.InputLayout.NumElements = input_descs.len() as u32;
            pso_desc.InputLayout.pInputElementDescs = input_descs.as_ptr();

            pso_desc.IBStripCutValue = D3D12_INDEX_BUFFER_STRIP_CUT_VALUE_DISABLED;
            pso_desc.PrimitiveTopologyType = to_dx12_topology(&data.primitive_mode);
        }

        // RTV and DSV formats
        {
            for i in 0..pipeline_interface.color_attachments.len() {
                let attachment_info = pipeline_interface.color_attachments.get(i).unwrap();
                pso_desc.RTVFormats[i] = to_dxgi_format(&attachment_info.pixel_format);
            }
            if let Some(depth_info) = pipeline_interface.depth_texture {
                pso_desc.DSVFormat = to_dxgi_format(&depth_info.pixel_format);
            }
        }

        // MSAA
        {
            if data.msaa_support != shaderpack::MSAASupport::None {
                pso_desc.SampleDesc.Count = 4;
                pso_desc.SampleDesc.Quality = 1;
            }
        }

        // Debug
        {
            // if debug
            pso_desc.Flags = D3D12_PIPELINE_STATE_FLAG_TOOL_DEBUG;
        }

        // PSO creation
        let mut pso = WeakPtr::<ID3D12PipelineState>::null();
        let hr = unsafe {
            self.device
                .CreateGraphicsPipelineState(&pso_desc, get_uuid(pso), pso.mut_void())
        };
        if FAILED(hr) {
            match hr {
                E_OUTOFMEMORY => return Err(PipelineCreationError::OutOfDeviceMemory),
                _ => return Err(PipelineCreationError::OutOfHostMemory),
            }
        }

        Ok(Dx12Pipeline {
            pso,
            root_sig: pipeline_interface.root_sig,
        })
    }

    fn create_image(
        &self,
        data: &shaderpack::TextureCreateInfo,
        swapchain_size: &Vector2<u32>,
    ) -> Result<Dx12Image, MemoryError> {
        let dimensions = match data.format.dimension_type {
            shaderpack::TextureDimensionType::ScreenRelative => Vector2::<u32>::new(),
            shaderpack::TextureDimensionType::Absolute => swapchain_size,
        };

        let mut texture_desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_DSV_DIMENSION_TEXTURE2D,
            Alignment: 0,
            Width: dimensions.x as u64,
            Height: dimensions.y as u32,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: to_dxgi_format(&data.format.pixel_format),
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 1 },
            Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
            Flags: D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET, /* TODO: Add a flag to TextureCreateInfo for if the
                                                             * texture will be an RTV, and only set this flag if that
                                                             * flag is true */
        };

        if data.format.pixel_format == shaderpack::PixelFormat::Depth
            || data.format.pixel_format == shaderpack::PixelFormat::DepthStencil
        {
            texture_desc.flags |= D3D12_RESOURCE_FLAG_ALLOW_DEPTH_STENCIL;
        }

        let mut image = WeakPtr::<ID3D12Resource>::null();
        let heap_props = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_DEFAULT,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_NOT_AVAILABLE,
            MemoryPoolPreference: D3D12_MEMORY_POOL_L1,
            CreationNodeMask: 0,
            VisibleNodeMask: 0,
        };

        let hr = unsafe {
            self.device.CreateCommittedResource(
                &heap_props,
                D3D12_HEAP_FLAG_NONE,
                &texture_desc,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                null,
                get_uuid(image),
                image.mut_void(),
            )
        };
        if FAILED(hr) {
            match hr {
                E_OUTOFMEMORY => return Err(MemoryError::OutOfDeviceMemory),
                _ => return Err(MemoryError::OutOfHostMemory),
            }
        }

        Ok(Dx12Image { image })
    }

    fn create_semaphore(&self, start_signalled: bool) -> Result<Dx12Semaphore, MemoryError> {
        let initial_fence_value = match start_signalled {
            true => CPU_FENCE_SIGNALED,
            false => 0,
        };

        let mut fence = WeakPtr::<ID3D12Fence>::null();
        let hr = unsafe {
            self.device.CreateFence(
                initial_fence_value,
                D3D12_FENCE_FLAG_NONE,
                get_uuid(fence),
                fence.mut_void(),
            )
        };
        if FAILED(hr) {
            match hr {
                E_OUTOFMEMORY => return Err(MemoryError::OutOfHostMemory),
                _ => return Err(MemoryError::OutOfHostMemory),
            }
        }

        Ok(Dx12Semaphore { fence })
    }

    fn create_semaphores(&self, count: u32, start_signalled: bool) -> Result<Vec<Dx12Semaphore>, MemoryError> {
        let mut vec = Vec::<Dx12Semaphore>::new();

        for i in 0..count {
            match self.create_semaphore(start_signalled) {
                Ok(fence) => vec.push(fence),
                Err(e) => return Err(e),
            }
        }

        Ok(vec)
    }

    fn create_fence(&self, start_signalled: bool) -> Result<Dx12Fence, MemoryError> {
        // I feel like a functional boi now
        self.create_semaphore(start_signalled).map(|semaphore| {
            let event = unsafe { CreateEventA(null(), false as i32, start_signalled as i32, null()) };
            semaphore.fence.SetEventOnCompletion(CPU_FENCE_SIGNALED, event);

            Dx12Fence {
                fence: semaphore.fence,
                event,
            }
        })
    }

    fn create_fences(&self, count: u32, start_signalled: bool) -> Result<Vec<Dx12Fence>, MemoryError> {
        let mut vec = Vec::<Dx12Fence>::new();

        for i in 0..count {
            match self.create_fence(start_signalled) {
                Ok(fence) => vec.push(fence),
                Err(e) => Err(e),
            }
        }

        Ok(vec)
    }

    fn wait_for_fences(&self, fences: Vec<Dx12Fence>) {
        for fence in fences {
            fence.wait_for_signal();
        }
    }

    fn reset_fences(&self, fences: Vec<Dx12Fence>) {
        for fence in fences {
            fence.reset();
        }
    }

    fn update_descriptor_sets(&self, updates: Vec<DescriptorSetWrite>) {
        for update in updates {
            let mut write_handle = D3D12_CPU_DESCRIPTOR_HANDLE {
                ..unsafe { mem::zeroed() }
            };

            let descriptor_heap_start_handle = unsafe { update.heap.GetGCPUDescriptorForHeapStart() };

            write_handle.ptr = descriptor_heap_start_handle.ptr + self.shader_resource_descriptor_size * update.binding;

            match update.update_info {
                DescriptorUpdateInfo::Image { image, format, sampler } => {
                    let mut srv_descriptor = D3D12_SHADER_RESOURCE_VIEW_DESC {
                        Format: to_dxgi_format(&format.pixel_format),
                        ViewDimension: D3D12_SRV_DIMENSION_TEXTURE2D, // TODO: Support more texture types
                        Shader4ComponentMapping: 0,
                        ..unsafe { mem::zeroed() }
                    };

                    srv_descriptor.Texture2D_mut() = D3D12_TEX2D_SRV {
                        MostDetailedMip: 0,
                        MipLevels: 1,
                        PlaneSlice: 0,
                        ResourceMinLODClamp: 0.0,
                    };

                    unsafe {
                        self.device
                            .CreateShaderResourceView(image.resource, &srv_descriptor, write_handle)
                    };
                }
            }
        }
    }
}
