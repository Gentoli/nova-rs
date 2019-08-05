use crate::rhi::DescriptorType;

use crate::rhi::dx12::com::WeakPtr;
use crate::shaderpack;
use spirv_cross::{hlsl, spirv};
use std::collections::HashMap;
use winapi::um::d3d12::*;
use winapi::um::d3dcommon::ID3DBlob;

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
    options: spirv_cross::Options,
    tables: &HashMap<u32, Vec<D3D12_DESCRIPTOR_RANGE1>>,
) -> WeakPtr<ID3DBlob> {
    let ast = spirv::Ast::<hlsl::Target>::parse(shader.source);
}
