use super::com::WeakPtr;
use crate::rhi::{dx12::dx12_command_list::Dx12CommandList, CommandAllocator, MemoryError};
use winapi::um::d3d12::*;

pub struct Dx12CommandAllocator {
    allocator: WeakPtr<ID3D12CommandAllocator>,
}

impl Dx12CommandAllocator {
    pub fn new(allocator: WeakPtr<ID3D12CommandAllocator>) -> Self {
        Dx12CommandAllocator { allocator }
    }
}

impl CommandAllocator for Dx12CommandAllocator {
    type CommandList = Dx12CommandList;

    fn create_command_list(&self, secondary_list: bool) -> Result<Dx12CommandList, MemoryError> {
        unimplemented!()
    }
}
