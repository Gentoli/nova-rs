#![allow(unsafe_code)]

#[macro_use]
use log::*;

use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::{DescriptorType, QueueType, ResourceState};
use crate::{shaderpack, ErrorCode};
use spirv_cross::{hlsl, spirv};
use std::collections::HashMap;
use std::ffi::CStr;
use std::mem;
use std::ptr::null;
use winapi::shared::dxgiformat::*;
use winapi::shared::ntdef::{LANG_NEUTRAL, MAKELANGID, SUBLANG_DEFAULT};
use winapi::shared::winerror::{FAILED, HRESULT};
use winapi::um::d3d12::*;
use winapi::um::d3d12shader::*;
use winapi::um::d3dcommon::ID3DBlob;
use winapi::um::d3dcommon::*;
use winapi::um::d3dcompiler::*;
use winapi::um::winbase::{FormatMessageA, FORMAT_MESSAGE_FROM_SYSTEM};
use winapi::Interface;

impl From<HRESULT> for ErrorCode<HRESULT> {
    fn from(hr: i32) -> Self {
        let message = unsafe {
            let mut error_message_buffer: [char; 1024] = ['\0'; 1024];

            FormatMessageA(
                FORMAT_MESSAGE_FROM_SYSTEM,
                null(),
                hr as u32,
                MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as u32,
                *error_message_buffer,
                1024,
                null(),
            );

            unsafe { CStr::from_ptr(error_message_buffer as _) }
                .to_str()
                .unwrap()
                .to_string()
        };

        ErrorCode(hr, message)
    }
}

pub fn to_dx12_range_type(descriptor_type: &DescriptorType) -> D3D12_DESCRIPTOR_RANGE_TYPE {
    match descriptor_type {
        DescriptorType::CombinedImageSampler => D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
        DescriptorType::UniformBuffer => D3D12_DESCRIPTOR_RANGE_TYPE_CBV,
        DescriptorType::StorageBuffer => D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
    }
}

pub fn to_dx12_blend(blend_factor: &shaderpack::BlendFactor) -> D3D12_BLEND {
    match blend_factor {
        shaderpack::BlendFactor::One => D3D12_BLEND_ONE,
        shaderpack::BlendFactor::Zero => D3D12_BLEND_ZERO,
        shaderpack::BlendFactor::SrcColor => D3D12_BLEND_SRC_COLOR,
        shaderpack::BlendFactor::DstColor => D3D12_BLEND_DEST_COLOR,
        shaderpack::BlendFactor::OneMinusSrcColor => D3D12_BLEND_INV_SRC_COLOR,
        shaderpack::BlendFactor::OneMinusDstColor => D3D12_BLEND_INV_DEST_COLOR,
        shaderpack::BlendFactor::SrcAlpha => D3D12_BLEND_SRC_ALPHA,
        shaderpack::BlendFactor::DstAlpha => D3D12_BLEND_DEST_ALPHA,
        shaderpack::BlendFactor::OneMinusSrcAlpha => D3D12_BLEND_INV_SRC_ALPHA,
        shaderpack::BlendFactor::OneMinusDstAlpha => D3D12_BLEND_INV_DEST_ALPHA,
    }
}

pub fn to_dx12_compare_func(op: &shaderpack::CompareOp) -> D3D12_COMPARISON_FUNC {
    match op {
        shaderpack::CompareOp::Never => D3D12_COMPARISON_FUNC_NEVER,
        shaderpack::CompareOp::Less => D3D12_COMPARISON_FUNC_LESS,
        shaderpack::CompareOp::LessEqual => D3D12_COMPARISON_FUNC_LESS_EQUAL,
        shaderpack::CompareOp::Greater => D3D12_COMPARISON_FUNC_GREATER,
        shaderpack::CompareOp::GreaterEqual => D3D12_COMPARISON_FUNC_GREATER_EQUAL,
        shaderpack::CompareOp::Equal => D3D12_COMPARISON_FUNC_EQUAL,
        shaderpack::CompareOp::NotEqual => D3D12_COMPARISON_FUNC_NOT_EQUAL,
        shaderpack::CompareOp::Always => D3D12_COMPARISON_FUNC_NEVER,
    }
}

pub fn to_dx12_stencil_op(op: &shaderpack::StencilOp) -> D3D12_STENCIL_OP {
    match op {
        shaderpack::StencilOp::Keep => D3D12_STENCIL_OP_KEEP,
        shaderpack::StencilOp::Zero => D3D12_STENCIL_OP_ZERO,
        shaderpack::StencilOp::Replace => D3D12_STENCIL_OP_REPLACE,
        shaderpack::StencilOp::Incr => D3D12_STENCIL_OP_INCR,
        shaderpack::StencilOp::IncrWrap => D3D12_STENCIL_OP_INCR_SAT,
        shaderpack::StencilOp::Decr => D3D12_STENCIL_OP_DECR,
        shaderpack::StencilOp::DecrWrap => D3D12_STENCIL_OP_DECR_SAT,
        shaderpack::StencilOp::Invert => D3D12_STENCIL_OP_INVERT,
    }
}

pub fn to_dx12_topology(topology: &shaderpack::PrimitiveTopology) -> D3D12_PRIMITIVE_TOPOLOGY_TYPE {
    match topology {
        shaderpack::PrimitiveTopology::Triangles => D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
        shaderpack::PrimitiveTopology::Lines => D3D12_PRIMITIVE_TOPOLOGY_TYPE_LINE,
    }
}

pub fn to_dx12_state(state: &ResourceState) -> D3D12_RESOURCE_STATES {
    match state {
        ResourceState::Undefined => D3D12_RESOURCE_STATE_COMMON,
        ResourceState::General => D3D12_RESOURCE_STATE_COMMON,
        ResourceState::ColorAttachment => D3D12_RESOURCE_STATE_RENDER_TARGET,
        ResourceState::DepthStencilAttachment => D3D12_RESOURCE_STATE_DEPTH_WRITE,
        ResourceState::DepthReadOnlyStencilAttachment => D3D12_RESOURCE_STATE_DEPTH_WRITE,
        ResourceState::DepthAttachmentStencilReadOnly => D3D12_RESOURCE_STATE_DEPTH_WRITE,
        ResourceState::DepthStencilReadOnlyAttachment => D3D12_RESOURCE_STATE_DEPTH_READ,
        ResourceState::PresentSource => D3D12_RESOURCE_STATE_PRESENT,
        ResourceState::NonFragmentShaderReadOnly => D3D12_RESOURCE_STATE_NON_PIXEL_SHADER_RESOURCE,
        ResourceState::FragmentShaderReadOnly => D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
        ResourceState::TransferSource => D3D12_RESOURCE_STATE_COPY_SOURCE,
        ResourceState::TransferDestination => D3D12_RESOURCE_STATE_COPY_DEST,
    }
}

pub fn to_command_list_type(queue_type: &QueueType) -> D3D12_COMMAND_LIST_TYPE {
    match queue_type {
        QueueType::Graphics => D3D12_COMMAND_LIST_TYPE_DIRECT,
        QueueType::Compute => D3D12_COMMAND_LIST_TYPE_COMPUTE,
        QueueType::Copy => D3D12_COMMAND_LIST_TYPE_COPY,
    }
}

pub fn to_dxgi_format(format: &shaderpack::PixelFormat) -> DXGI_FORMAT {
    match format {
        shaderpack::PixelFormat::RGBA8 => DXGI_FORMAT_R8G8B8A8_UNORM,
        shaderpack::PixelFormat::RGBA16F => DXGI_FORMAT_R16G16B16A16_FLOAT,
        shaderpack::PixelFormat::RGBA32F => DXGI_FORMAT_R32G32B32A32_FLOAT,
        shaderpack::PixelFormat::Depth => DXGI_FORMAT_D32_FLOAT,
        shaderpack::PixelFormat::DepthStencil => DXGI_FORMAT_D32_FLOAT_S8X24_UINT,
    }
}

pub fn compile_shader(
    shader: shaderpack::ShaderSource,
    target: &str,
    options: hlsl::CompilerOptions,
    tables: &mut HashMap<u32, Vec<D3D12_DESCRIPTOR_RANGE1>>,
) -> Result<WeakPtr<ID3DBlob>, ErrorCode<HRESULT>> {
    let shader_module = spirv::Module::from_words(&shader.source);
    let shader_compiler = spirv::Ast::<hlsl::Target>::parse(&shader_module);
    match shader_compiler
        .and_then(|ast| ast.get_shader_resources())
        .map_err(ErrorCode::<HRESULT>::from)
    {
        Ok(resources) => {
            let mut spirv_sampled_images = HashMap::<String, spirv::Resource>::new();
            for sampled_image in resources.sampled_images {
                spirv_sampled_images.insert(sampled_image.name, sampled_image);
            }

            let mut spirv_uniform_buffers = HashMap::<String, spirv::Resource>::new();
            for uniform_buffer in resources.uniform_buffers {
                spirv_uniform_buffers.insert(uniform_buffer.name, uniform_buffer);
            }

            // I know that shader_compiler is Ok because I'm in the branch of the match where it's Ok
            shader_compiler?
                .compile()
                .map_err(ErrorCode::<HRESULT>::from)
                .and_then(|shader_hlsl| {
                    // TODO: Write HLSL to a file to help debug
                    let shader_blob = WeakPtr::<ID3DBlob>::null();
                    let shader_error_blob = WeakPtr::<ID3DBlob>::null();

                    dx_call!(D3DCompile2(
                        shader_hlsl.as_ptr() as _,
                        shader_hlsl.len(),
                        shader.filename.to_str().unwrap().as_ptr() as _,
                        null as _,
                        D3D_COMPILE_STANDARD_FILE_INCLUDE,
                        "main".as_ptr() as _,
                        target.as_ptr() as _,
                        D3DCOMPILE_ENABLE_STRICTNESS | D3DCOMPILE_IEEE_STRICTNESS | D3DCOMPILE_OPTIMIZATION_LEVEL3,
                        0,
                        0,
                        null as _,
                        0,
                        shader_blob.GetBufferPointer() as _,
                        shader_error_blob.GetBufferPointer() as _,
                    ));

                    match extract_descriptor_info_from_blob(
                        tables,
                        &shader_compiler.unwrap(),
                        &mut spirv_sampled_images,
                        &mut spirv_uniform_buffers,
                        &shader_blob,
                    ) {
                        Ok(_) => Ok(shader_blob),
                        Err(e) => Err(e),
                    }
                })
        }
        Err(e) => Err(e),
    }
}

fn extract_descriptor_info_from_blob(
    tables: &mut HashMap<u32, Vec<D3D12_DESCRIPTOR_RANGE1>>,
    shader_compiler: &spirv::Ast<hlsl::Target>,
    spirv_sampled_images: &mut HashMap<String, spirv::Resource>,
    spirv_uniform_buffers: &mut HashMap<String, spirv::Resource>,
    shader_blob: &WeakPtr<ID3DBlob>,
) -> Result<bool, ErrorCode<HRESULT>> {
    let mut shader_reflector = WeakPtr::<ID3D12ShaderReflection>::null();
    dx_call!(D3DReflect(
        shader_blob.GetBufferPointer(),
        shader_blob.GetBufferSize(),
        &ID3D12ShaderReflection::uuidof(),
        shader_reflector.mut_void(),
    ));

    let mut shader_desc = D3D12_SHADER_DESC {
        ..unsafe { mem::zeroed() }
    };
    dx_call!(shader_reflector.GetDesc(&mut shader_desc));

    let shader_inputs = HashMap::<String, D3D12_SHADER_INPUT_BIND_DESC>::new();
    for i in 0..shader_desc.BoundResources {
        let mut binding_desc = D3D12_SHADER_INPUT_BIND_DESC {
            ..unsafe { mem::zeroed() }
        };
        dx_call!(shader_reflector.GetResourceBindingDesc(i, &mut binding_desc));

        if binding_desc.Type == D3D_SIT_CBUFFER {}

        save_descriptor_info(
            tables,
            shader_compiler,
            &spirv_sampled_images,
            &spirv_uniform_buffers,
            &mut binding_desc,
        );
    }

    Ok(true)
}

fn save_descriptor_info(
    tables: &mut HashMap<u32, Vec<D3D12_DESCRIPTOR_RANGE1>>,
    shader_compiler: &spirv::Ast<hlsl::Target>,
    mut spirv_sampled_images: &HashMap<String, spirv::Resource>,
    mut spirv_uniform_buffers: &HashMap<String, spirv::Resource>,
    mut binding_desc: &D3D12_SHADER_INPUT_BIND_DESC,
) {
    let mut set: u32;
    let mut descriptor_type: u32;
    let mut spirv_resource: spirv::Resource;

    if binding_desc.Type == D3D_SIT_CBUFFER {
        let name_str = unsafe { CStr::from_ptr(binding_desc.Name as _) }.to_str().unwrap();

        descriptor_type = D3D12_DESCRIPTOR_RANGE_TYPE_CBV;
        spirv_resource = spirv_uniform_buffers[name_str].clone();
        set = shader_compiler
            .get_decoration(spirv_resource.id, spirv::Decoration::DescriptorSet)
            .unwrap();

        add_resource_to_descriptor_table(&descriptor_type, &binding_desc, &set, tables);
    } else if binding_desc.Type == D3D_SIT_TEXTURE {
        let name_str = unsafe { CStr::from_ptr(binding_desc.Name as _) }.to_str().unwrap();

        descriptor_type = D3D12_DESCRIPTOR_RANGE_TYPE_SRV;
        spirv_resource = spirv_sampled_images[name_str].clone();
        set = shader_compiler
            .get_decoration(spirv_resource.id, spirv::Decoration::DescriptorSet)
            .unwrap();

        add_resource_to_descriptor_table(&descriptor_type, &binding_desc, &set, tables);
        add_resource_to_descriptor_table(&D3D12_DESCRIPTOR_RANGE_TYPE_SAMPLER, &binding_desc, &set, tables);
    }
}

fn add_resource_to_descriptor_table(
    descriptor_type: &u32,
    binding_desc: &D3D12_SHADER_INPUT_BIND_DESC,
    set: &u32,
    tables: &mut HashMap<u32, Vec<D3D12_DESCRIPTOR_RANGE1>>,
) {
    let range = D3D12_DESCRIPTOR_RANGE1 {
        RangeType: descriptor_type.clone(),
        NumDescriptors: 1,
        BaseShaderRegister: binding_desc.BindPoint,
        RegisterSpace: binding_desc.Space,
        Flags: 0,
        OffsetInDescriptorsFromTableStart: D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
    };

    tables[set].push(range);
}
