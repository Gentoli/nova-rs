use crate::rhi::Framebuffer;

use crate::rhi::dx12::com::WeakPtr;
use winapi::um::d3d12::ID3D12DescriptorHeap;
use winapi::um::d3d12::D3D12_CPU_DESCRIPTOR_HANDLE;

pub struct Dx12Framebuffer {
    pub color_attachments: Vec<D3D12_CPU_DESCRIPTOR_HANDLE>,

    pub depth_attachment: Option<D3D12_CPU_DESCRIPTOR_HANDLE>,

    pub descriptor_heap: WeakPtr<ID3D12DescriptorHeap>,
}

impl Framebuffer for Dx12Framebuffer {}
