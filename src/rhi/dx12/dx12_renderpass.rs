use crate::rhi::Renderpass;

use winapi::um::d3d12::*;

/// Information on the beginning and ending access for a resource
pub struct Dx12RenderPassAccessInfo {
    pub beginning_access: D3D12_RENDER_PASS_BEGINNING_ACCESS,
    pub ending_access: D3D12_RENDER_PASS_ENDING_ACCESS,
}

/// DX12 renderpass yay
///
/// DX12 renderpasses are simpler than Vulkan renderpasses, because DX12 doesnt' care about mobile's tilled GPUs. This
/// struct basically holds the parameters for a DX12 renderpass
pub struct Dx12Renderpass {
    pub render_targets: Vec<Dx12RenderPassAccessInfo>,
    pub depth_stencil: Option<Dx12RenderPassAccessInfo>,
}

impl Dx12Renderpass {
    pub fn new(render_targets: Vec<Dx12RenderPassAccessInfo>, depth_stencil: Option<Dx12RenderPassAccessInfo>) -> Self {
        Dx12Renderpass {
            render_targets,
            depth_stencil,
        }
    }
}

impl Renderpass for Dx12Renderpass {}
