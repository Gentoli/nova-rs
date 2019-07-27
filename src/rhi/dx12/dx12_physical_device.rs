use crate::rhi::{DeviceCreationError, PhysicalDevice, PhysicalDeviceProperties};

use super::dx12_device::Dx12Device;
use winapi::shared::{dxgi1_2, winerror};
use winapi::um::d3d12::*;

/// A physical device which supports DX12
pub struct Dx12PhysicalDevice {
    adapter: com::WeakPtr<dxgi1_2::IDXGIAdapter2>,
}

impl Dx12PhysicalDevice {
    pub fn new(adapter: com::WeakPtr<dxgi1_2::IDXGIAdapter2>) -> Self {
        Dx12PhysicalDevice { adapter }
    }
}

impl<'a> PhysicalDevice<'a> for Dx12PhysicalDevice {
    type Device = Dx12Device<'a>;

    fn get_properties(&self) -> PhysicalDeviceProperties {
        unimplemented!()
    }

    fn can_be_used_by_nova(&self) -> bool {
        // TODO: Something more in depth
        match self.create_logical_device() {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn create_logical_device(&'a self) -> Result<Dx12Device<'a>, DeviceCreationError> {
        unsafe {
            let mut device: ID3D12Device;
            let hr = D3D12CreateDevice(adapter, D3D_FEATURE_LEVEL_11_0, ID3D12Device::uuifof(), *device);
            if winerror::SUCCEEDED(hr) {
                Ok(Dx12Device::new(self, device))
            } else {
                Err(DeviceCreationError::Failed)
            }
        }
    }

    fn get_free_memory(&self) -> u64 {
        0
    }
}
