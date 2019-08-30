use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::{Buffer, BufferCreateInfo};
use winapi::um::d3d12::*;

pub struct Dx12Buffer {
    pub size: u64,
    pub resource: WeakPtr<ID3D12Resource>,
}

impl Buffer for Dx12Buffer {
    fn write_data(&self, data: BufferCreateInfo, num_bytes: u64, offset: u64) {
        unimplemented!()
    }
}
