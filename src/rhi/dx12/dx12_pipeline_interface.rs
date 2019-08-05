use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::PipelineInterface;
use crate::shaderpack;
use winapi::um::d3d12::*;

pub struct Dx12PipelineInterface {
    pub root_sig: WeakPtr<ID3D12RootSignature>,
    color_attachments: Vec<shaderpack::TextureAttachmentInfo>,
    depth_texture: Option<shaderpack::TextureAttachmentInfo>,
}

impl PipelineInterface for Dx12PipelineInterface {}
