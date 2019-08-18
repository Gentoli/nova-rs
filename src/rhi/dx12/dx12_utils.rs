#![allow(unsafe_code)]

use crate::rhi::DescriptorType;

use crate::rhi::dx12::com::WeakPtr;
use crate::shaderpack;
use std::collections::HashMap;
use winapi::um::d3d12::*;
use winapi::um::d3dcommon::ID3DBlob;
#[macro_use]
use log::*;
use spirv_cross::{hlsl, spirv, ErrorCode};
use std::ffi::CStr;
use std::mem;
use std::ptr::null;
use winapi::shared::winerror::FAILED;
use winapi::um::d3d12shader::*;
use winapi::um::d3dcommon::*;
use winapi::um::d3dcompiler::*;
use winapi::Interface;

pub fn to_dx12_range_type(descriptor_type: &DescriptorType) -> D3D12_DESCRIPTOR_RANGE_TYPE {
    match descriptor_type {
        DescriptorType::CombinedImageSampler => D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
        DescriptorType::UniformBuffer => D3D12_DESCRIPTOR_RANGE_TYPE_CBV,
        DescriptorType::StorageBuffer => D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
    }
}

pub fn compile_shader(
    shader: shaderpack::ShaderSource,
    target: String,
    options: hlsl::CompilerOptions,
    tables: &mut HashMap<u32, Vec<D3D12_DESCRIPTOR_RANGE1>>,
) -> Result<WeakPtr<ID3DBlob>, spirv_cross::ErrorCode> {
    let shader_module = spirv::Module::from_words(&shader.source);
    let shader_compiler = spirv::Ast::<hlsl::Target>::parse(&shader_module);
    match shader_compiler.and_then(|ast| ast.get_shader_resources()) {
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
            shader_compiler?.compile().and_then(|shader_hlsl| {
                // TODO: Write HLSL to a file to help debug
                let shader_blob = WeakPtr::<ID3DBlob>::null();
                let shader_error_blob = WeakPtr::<ID3DBlob>::null();

                let hr = dx_call!(
                    unsafe {
                        D3DCompile2(
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
                        )
                    },
                    "DirectX shader compiler error"
                );

                extract_descriptor_info_from_blob(
                    tables,
                    &shader_compiler.unwrap(),
                    &mut spirv_sampled_images,
                    &mut spirv_uniform_buffers,
                    &shader_blob,
                )
            })
        }
        Err(e) => match e {
            spirv_cross::ErrorCode::Unhandled => {
                warn!("Unhandled error when compiling shader {:?}", shader.filename.to_str())
            }
            spirv_cross::ErrorCode::CompilationError(err) => warn!(
                "Compilation error {} when compiling shader {:?}",
                err,
                shader.filename.to_str()
            ),
        },
    };

    let blob = WeakPtr::<ID3DBlob>::null();

    Ok(blob)
}

fn extract_descriptor_info_from_blob(
    tables: &mut HashMap<u32, Vec<D3D12_DESCRIPTOR_RANGE1>>,
    shader_compiler: &spirv::Ast<hlsl::Target>,
    spirv_sampled_images: &mut _,
    spirv_uniform_buffers: &mut _,
    shader_blob: &WeakPtr<ID3DBlob>,
) {
    let mut shader_reflector = WeakPtr::<ID3D12ShaderReflection>::null();
    dx_call!(
        unsafe {
            D3DReflect(
                shader_blob.GetBufferPointer(),
                shader_blob.GetBufferSize(),
                &ID3D12ShaderReflection::uuidof(),
                shader_reflector.mut_void(),
            )
        },
        "Could not create D3D12ShaderReflector"
    );

    let mut shader_desc = D3D12_SHADER_DESC {
        ..unsafe { mem::zeroed() }
    };
    dx_call!(
        shader_reflector.GetDesc(&mut shader_desc),
        "Could not get shader description"
    );

    let shader_inputs = HashMap::<String, D3D12_SHADER_INPUT_BIND_DESC>::new();
    for i in 0..shader_desc.BoundResources {
        let mut binding_desc = D3D12_SHADER_INPUT_BIND_DESC {
            ..unsafe { mem::zeroed() }
        };
        dx_call!(
            shader_reflector.GetResourceBindingDesc(i, &mut binding_desc),
            "Could not get resource binding description"
        );

        if binding_desc.Type == D3D_SIT_CBUFFER {}

        save_descriptor_info(
            tables,
            shader_compiler,
            &spirv_sampled_images,
            &spirv_uniform_buffers,
            &mut binding_desc,
        );
    }
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
