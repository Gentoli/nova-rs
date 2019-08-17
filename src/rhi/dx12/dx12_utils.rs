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
use std::ptr::null;
use winapi::shared::winerror::FAILED;
use winapi::um::d3d12shader::*;
use winapi::um::d3dcompiler::*;

use std::ffi::CStr;
use std::mem;
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

                let hr = unsafe {
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
                };
                if FAILED(hr) {
                    return Err(ErrorCode::CompilationError(String::from(
                        "DirectX shader compiler error",
                    )));
                }

                let mut shader_reflector = WeakPtr::<ID3D12ShaderReflection>::null();
                let hr = unsafe {
                    D3DReflect(
                        shader_blob.GetBufferPointer(),
                        shader_blob.GetBufferSize(),
                        &ID3D12ShaderReflection::uuidof(),
                        shader_reflector.mut_void(),
                    )
                };
                if FAILED(hr) {
                    return Err(ErrorCode::CompilationError(String::from(
                        "Could not create D3D12ShaderReflector",
                    )));
                }

                let mut shader_desc = D3D12_SHADER_DESC {
                    ..unsafe { mem::zeroed() }
                };
                let hr = shader_reflector.GetDesc(&mut shader_desc);
                if FAILED(hr) {
                    return Err(ErrorCode::CompilationError(String::from(
                        "Could not get shader description",
                    )));
                }

                let shader_inputs = HashMap::<String, D3D12_SHADER_INPUT_BIND_DESC>::new();
                for i in 0..shader_desc.BoundResources {
                    let mut binding_desc = D3D12_SHADER_INPUT_BIND_DESC {
                        ..unsafe { mem::zeroed() }
                    };
                    let hr = shader_reflector.GetResourceBindingDesc(i, &mut binding_desc);
                    if FAILED(hr) {
                        return Err(ErrorCode::CompilationError(String::from(
                            "Could not get resource binding description",
                        )));
                    }

                    let (descriptor_type, spirv_resource, set) = match get_descriptor_info(
                        tables,
                        &shader_compiler.unwrap(),
                        &spirv_sampled_images,
                        &spirv_uniform_buffers,
                        &mut binding_desc,
                    ) {
                        Ok(tup) => tup,
                        Err(e) => return Err(e),
                    };
                }

                Err(ErrorCode::CompilationError(String::from(
                    "Could not create D3D12ShaderReflector",
                )))
            });
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

fn get_descriptor_info(
    tables: &mut HashMap<u32, Vec<D3D12_DESCRIPTOR_RANGE1>>,
    shader_compiler: &spirv::Ast<hlsl::Target>,
    mut spirv_sampled_images: &HashMap<String, spirv::Resource>,
    mut spirv_uniform_buffers: &HashMap<String, spirv::Resource>,
    mut binding_desc: &mut D3D12_SHADER_INPUT_BIND_DESC,
) -> Result<(u32, spirv::Resource, u32), ErrorCode> {
    // if binding_desc.Type

    match binding_desc.Type {
        D3D12_SIT_CBUFFER => {
            let name_str = unsafe { CStr::from_ptr(binding_desc.Name as _) }.to_str().unwrap();
            let descriptor_type = D3D12_DESCRIPTOR_RANGE_TYPE_CBV;
            let spirv_resource = spirv_uniform_buffers[name_str].clone();
            let set = match shader_compiler.get_decoration(spirv_resource.id, spirv::Decoration::DescriptorSet) {
                Ok(set) => set,
                Err(e) => {
                    return Err(ErrorCode::CompilationError(String::from(
                        "Could not get descriptor set decoration",
                    )));
                }
            };

            add_resource_to_descriptor_table(&descriptor_type, &binding_desc, &set, tables);

            Ok((descriptor_type, spirv_resource, set))
        }
        D3D_SIT_TEXTURE => {
            let name_str = unsafe { CStr::from_ptr(binding_desc.Name as _) }.to_str().unwrap();
            let descriptor_type = D3D12_DESCRIPTOR_RANGE_TYPE_SRV;
            let spirv_resource = spirv_sampled_images[name_str].clone();
            let set = match shader_compiler.get_decoration(spirv_resource.id, spirv::Decoration::DescriptorSet) {
                Ok(set) => set,
                Err(e) => {
                    return Err(ErrorCode::CompilationError(String::from(
                        "Could not get descriptor set decoration",
                    )));
                }
            };

            add_resource_to_descriptor_table(&D3D12_DESCRIPTOR_RANGE_TYPE_SAMPLER, &binding_desc, &set, &mut tables);

            Ok((descriptor_type, spirv_resource, set))
        }
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
