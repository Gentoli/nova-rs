use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::Semaphore;
use winapi::um::d3d12::*;

pub struct Dx12Semaphore {
    pub fence: WeakPtr<ID3D12Fence>,
}

impl Semaphore for Dx12Semaphore {}
