#![allow(unsafe_code)]

use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::dx12::get_uuid;
use crate::rhi::{dx12::dx12_buffer::Dx12Buffer, BufferCreateInfo, BufferUsage, Memory, MemoryError};
use std::ptr::null;
use winapi::shared::dxgiformat::*;
use winapi::shared::dxgitype::*;
use winapi::shared::winerror::FAILED;
use winapi::um::d3d12::*;

pub struct Dx12Memory {
    device: WeakPtr<ID3D12Device>,

    heap: WeakPtr<ID3D12Heap>,
    size: u64,
}

impl Dx12Memory {
    pub fn new(device: WeakPtr<ID3D12Device>, heap: WeakPtr<ID3D12Heap>, size: u64) -> Self {
        Dx12Memory { device, heap, size }
    }
}

impl Memory for Dx12Memory {
    type Buffer = Dx12Buffer;

    fn create_buffer(&self, data: BufferCreateInfo) -> Result<Self::Buffer, MemoryError> {
        let states = match data.buffer_usage {
            BufferUsage::UniformBuffer => D3D12_RESOURCE_STATE_VERTEX_AND_CONSTANT_BUFFER,
            BufferUsage::IndexBuffer => D3D12_RESOURCE_STATE_INDEX_BUFFER,
            BufferUsage::VertexBuffer => D3D12_RESOURCE_STATE_VERTEX_AND_CONSTANT_BUFFER,
            BufferUsage::StagingBuffer => D3D12_RESOURCE_STATE_COPY_SOURCE,
        };

        let resource_desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
            Alignment: 0,
            Width: data.size as u64,
            Height: 1,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: DXGI_FORMAT_UNKNOWN,
            SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
            Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
            Flags: D3D12_RESOURCE_FLAG_NONE,
        };

        let mut buffer = WeakPtr::<ID3D12Resource>::null();
        let hr = unsafe {
            self.device.CreatePlacedResource(
                self.heap,
                data.allocation.allocation_info.offset,
                &resource_desc,
                states,
                null(),
                get_uuid(buffer),
                buffer.mut_void(),
            )
        };
        if FAILED(hr) {
            return Err(MemoryError::OutOfDeviceMemory);
        }

        Ok(Dx12Buffer {
            size: data.size as u64,
            resource: buffer,
        })
    }
}
