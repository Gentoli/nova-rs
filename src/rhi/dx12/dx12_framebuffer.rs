use crate::rhi::Framebuffer;

pub struct Dx12Framebuffer {
    pub color_attachments: Vec<D3D12_CPU_DESCRIPTOR_HANDLE>,

    pub depth_attachment: Option<D3D12_CEPU_DESCRIPTOR_HANDLE>,
}

impl Framebuffer for Dx12Framebuffer {}
