use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::Image;
use winapi::um::d3d12::*;

pub struct Dx12Image {
    pub image: WeakPtr<ID3D12Resource>,
}

impl Image for Dx12Image {}
