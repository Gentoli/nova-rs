use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::{dx12::dx12_buffer::Dx12Buffer, BufferCreateInfo, Memory, MemoryError};
use winapi::shared::ntdef::UNICODE_STRING_MAX_BYTES;
use winapi::um::d3d12::*;

pub struct Dx12Memory {}

impl Dx12Memory {
    pub fn new(heap: WeakPtr<ID3D12Heap>, size: u64) -> Self {
        unimplemented!()
    }
}

impl Memory for Dx12Memory {
    type Buffer = Dx12Buffer;

    fn create_buffer(&self, data: BufferCreateInfo) -> Result<Self::Buffer, MemoryError> {
        unimplemented!()
    }
}
