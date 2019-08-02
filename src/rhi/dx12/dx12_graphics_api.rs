#![allow(unsafe_code)]

use winapi::{
    shared::{dxgi, dxgi1_2, dxgi1_3, dxgi1_4, winerror},
    Interface,
};

use log::error;

use crate::rhi::{GraphicsApi, PhysicalDevice};

use super::dx12_physical_device::Dx12PhysicalDevice;
use crate::rhi::dx12::com::WeakPtr;
use crate::surface::{Surface, Win32Surface};
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Dx12GraphicsApi {
    factory: WeakPtr<dxgi1_4::IDXGIFactory4>,
}

impl Dx12GraphicsApi {
    fn new() -> Self {
        let factory_flags = dxgi1_3::DXGI_CREATE_FACTORY_DEBUG;

        let mut factory = WeakPtr::<dxgi1_4::IDXGIFactory4>::null();
        let hr = unsafe {
            dxgi1_3::CreateDXGIFactory2(factory_flags, &dxgi1_4::IDXGIFactory4::uuidof(), factory.mut_void())
        };

        if !winerror::SUCCEEDED(hr) {
            error!("Failed to create DXGI Factory: {:?}", hr);
        }

        Dx12GraphicsApi { factory }
    }
}

impl GraphicsApi for Dx12GraphicsApi {
    type PhysicalDevice = Dx12PhysicalDevice;
    type PlatformSurface = Win32Surface;

    fn get_adapters(&self) -> Vec<Dx12PhysicalDevice> {
        let mut adapters: Vec<Dx12PhysicalDevice> = vec![];

        let mut cur_adapter = 0;
        loop {
            let mut adapter = WeakPtr::<dxgi::IDXGIAdapter1>::null();
            let hr = unsafe {
                self.factory
                    .EnumAdapters1(cur_adapter, adapter.mut_void() as *mut *mut _)
            };
            if hr == winerror::DXGI_ERROR_NOT_FOUND {
                break;
            }

            cur_adapter += 1;

            let (adapter2, hr) = unsafe { adapter.cast::<dxgi1_2::IDXGIAdapter2>() };
            if !winerror::SUCCEEDED(hr) {
                // We need IDXGIAdapter2 features, but this physical device doesn't have them
                continue;
            }

            let phys_device = Dx12PhysicalDevice::new(adapter2);

            if phys_device.can_be_used_by_nova() {
                adapters.push(phys_device);
            }
        }

        adapters
    }

    fn get_surface(&self) -> Rc<dyn Surface<Win32Surface>> {
        unimplemented!()
    }
}
