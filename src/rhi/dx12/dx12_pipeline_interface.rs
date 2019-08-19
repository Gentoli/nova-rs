use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::PipelineInterface;
use crate::shaderpack;
use winapi::um::d3d12::*;

pub struct Dx12PipelineInterface {
    pub root_sig: WeakPtr<ID3D12RootSignature>,
    pub color_attachments: Vec<shaderpack::TextureAttachmentInfo>,
    pub depth_texture: Option<shaderpack::TextureAttachmentInfo>,
}

impl PipelineInterface for Dx12PipelineInterface {}
