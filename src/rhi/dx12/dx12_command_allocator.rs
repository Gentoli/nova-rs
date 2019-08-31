#![allow(unsafe_code)]

use super::com::WeakPtr;
use crate::rhi::dx12::dx12_utils::to_command_list_type;
use crate::rhi::dx12::get_uuid;
use crate::rhi::{dx12::dx12_command_list::Dx12CommandList, CommandAllocator, MemoryError, QueueType};
use std::ptr::null;
use winapi::shared::winerror::FAILED;
use winapi::um::d3d12::*;

pub struct Dx12CommandAllocator {
    device: WeakPtr<ID3D12Device>,

    /// Actual command allocator
    allocator: WeakPtr<ID3D12CommandAllocator>,

    /// The queue type that this command allocator can allocate commands for
    queue_type: QueueType,
}

impl Dx12CommandAllocator {
    pub fn new(
        device: WeakPtr<ID3D12Device>,
        allocator: WeakPtr<ID3D12CommandAllocator>,
        queue_type: QueueType,
    ) -> Self {
        Dx12CommandAllocator {
            device,
            allocator,
            queue_type,
        }
    }
}

impl CommandAllocator for Dx12CommandAllocator {
    type CommandList = Dx12CommandList;

    fn create_command_list(&self, secondary_list: bool) -> Result<Dx12CommandList, MemoryError> {
        let mut command_list = WeakPtr::<ID3D12CommandList>::null();

        let command_list_type = match secondary_list {
            true => D3D12_COMMAND_LIST_TYPE_BUNDLE,
            false => to_command_list_type(&self.queue_type),
        };

        let hr = unsafe {
            self.device.CreateCommandList(
                0,
                command_list_type,
                self.command_allocator,
                null(),
                get_uuid(command_list),
                command_list.mut_void(),
            )
        };

        if FAILED(hr) {
            return Err(MemoryError::OutOfHostMemory);
        }

        let (list, hr) = unsafe { command_list.cast::<ID3D12GraphicsCommandList>() };
        if FAILED(hr) {
            Err(MemoryError::OutOfHostMemory)
        } else {
            Ok(Dx12CommandList::new(list))
        }
    }
}
