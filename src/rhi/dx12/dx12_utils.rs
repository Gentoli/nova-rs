use crate::rhi::DescriptorType;

use winapi::um::d3d12::*;

pub fn to_dx12_range_type(descriptor_type: DescriptorType) -> D3D12_DESCRIPTOR_RANGE_TYPE {
    match descriptor_type {
        DescriptorType::CombinedImageSampler => D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
        DescriptorType::UniformBuffer => D3D12_DESCRIPTOR_RANGE_TYPE_CBV,
        DescriptorType::StorageBuffer => D3D12_DESCRIPTOR_RANGE_TYPE_UAV,
    }
}
