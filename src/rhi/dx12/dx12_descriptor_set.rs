use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::DescriptorSet;
use winapi::um::d3d12::ID3D12DescriptorHeap;

pub struct Dx12DescriptorSet {}

impl DescriptorSet for Dx12DescriptorSet {}
