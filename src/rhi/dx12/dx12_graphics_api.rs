#![allow(unsafe_code)]

use winapi::{
    shared::{dxgi, dxgi1_2, dxgi1_3, dxgi1_4, winerror},
    Interface,
};

use log::error;

use crate::rhi::{Device, GraphicsApi};

use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::dx12::dx12_device::Dx12Device;
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
    type Device = Dx12Device;
    type PlatformSurface = Win32Surface;

    fn get_adapters(&self) -> Vec<Dx12Device> {
        let mut adapters: Vec<Dx12Device> = vec![];

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

            match Dx12Device::new(adapter2) {
                Some(device) => {
                    if device.can_be_used_by_nova() {
                        adapters.push(device);
                    }
                }
                None => (),
            }
        }

        adapters
    }

    fn get_surface(&self) -> Rc<dyn Surface<Win32Surface>> {
        unimplemented!()
    }
}
