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
use winapi::um::d3d12::{ID3D12ShaderReflection, D3D12_SHADER_DESC, D3D12_SHADER_INPUT_BIND_DESC};
use winapi::um::d3dcompiler::{
    D3DCompile2, D3DReflect, D3DCOMPILE_ENABLE_STRICTNESS, D3DCOMPILE_IEEE_STRICTNESS, D3DCOMPILE_OPTIMIZATION_LEVEL3,
    D3D_COMPILE_STANDARD_FILE_INCLUDE,
};

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
    tables: &HashMap<u32, Vec<D3D12_DESCRIPTOR_RANGE1>>,
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
                        shader.filename.into_os_string().as_ptr() as _,
                        null,
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

                let mut shader_desc = D3D12_SHADER_DESC::new();
                let hr = shader_reflector.GetDesc(&shader_desc);
                if FAILED(hr) {
                    return Err(ErrorCode::CompilationError(String::from(
                        "Could not get shader description",
                    )));
                }

                let shader_inputs = HashMap::<String, D3D12_SHADER_INPUT_BIND_DESC>::new();
                for bound_resource in shader_desc.BoundResources {}

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
