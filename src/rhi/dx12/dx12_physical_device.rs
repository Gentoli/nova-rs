use crate::rhi::{DeviceCreationError, PhysicalDevice, PhysicalDeviceProperties};

use super::dx12_device::Dx12Device;
use crate::rhi::dx12::com::WeakPtr;
use winapi::shared::{dxgi1_2, winerror};
use winapi::um::d3d12::*;
use winapi::um::d3dcommon::*;

/// A physical device which supports DX12
pub struct Dx12PhysicalDevice {
    adapter: WeakPtr<dxgi1_2::IDXGIAdapter2>,
}

impl Dx12PhysicalDevice {
    pub fn new(adapter: WeakPtr<dxgi1_2::IDXGIAdapter2>) -> Self {
        Dx12PhysicalDevice { adapter }
    }
}

impl PhysicalDevice for Dx12PhysicalDevice {
    type Device = Dx12Device;

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

    fn create_logical_device(&self) -> Result<Dx12Device, DeviceCreationError> {
        unsafe {
            let mut device = WeakPtr::<ID3D12Device>::null();
            // TODO: Figure out how to determine which SDK version the system we're running on supports
            let hr = D3D12CreateDevice(
                self.adapter.as_unknown(),
                D3D_FEATURE_LEVEL_11_0,
                ID3D12Device::uuifof(),
                device.mut_void(),
            );
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
