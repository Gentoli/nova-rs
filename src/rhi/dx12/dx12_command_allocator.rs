use crate::rhi::{dx12::dx12_command_list::Dx12CommandList, CommandAllocator, MemoryError};

pub struct Dx12CommandAllocator {
    allocator: d3d12::CommandAllocator,
}

impl Dx12CommandAllocator {
    pub fn new(allocator: d3d12::CommandAllocator) -> Self {
        Dx12CommandAllocator { allocator }
    }
}

impl CommandAllocator for Dx12CommandAllocator {
    type CommandList = Dx12CommandList;

    fn create_command_list() -> Result<Dx12CommandList, MemoryError> {
        unimplemented!()
    }
}
