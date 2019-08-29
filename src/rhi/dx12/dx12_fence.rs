use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::Fence;
use winapi::um::d3d12::*;
use winapi::um::winnt::HANDLE;

pub struct Dx12Fence {
    pub fence: WeakPtr<ID3D12Fence>,
    pub event: HANDLE,
}

impl Fence for Dx12Fence {}
