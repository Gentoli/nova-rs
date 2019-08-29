use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::Semaphore;

pub struct Dx12Semaphore {
    pub fence: WeakPtr<ID3D12Fence>,
}

impl Semaphore for Dx12Semaphore {}
