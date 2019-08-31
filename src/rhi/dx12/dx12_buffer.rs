#![allow(unsafe_code)]

use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::{Buffer, BufferCreateInfo, MappingError};
use core::ptr;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::d3d12::*;

pub struct Dx12Buffer {
    pub size: u64,
    pub resource: WeakPtr<ID3D12Resource>,
}

impl Buffer for Dx12Buffer {
    fn map(&self) -> Result<*mut (), MappingError> {
        let mapped_range = D3D12_RANGE {
            Begin: 0 as usize,
            End: self.size as usize,
        };

        let mut mapped_buffer = ptr::null_mut();

        unsafe {
            let hr = self.resource.Map(0, &mapped_range, &mapped_buffer);
            if SUCCEEDED(hr) {
                Ok(mapped_buffer)
            } else {
                Err(MappingError::MappingFailed)
            }
        }
    }

    fn unmap(&self) {
        let mapped_range = D3D12_RANGE {
            Begin: 0 as usize,
            End: self.size as usize,
        };

        unsafe { self.resource.Unmap(0, &mapped_range) };
    }
}
