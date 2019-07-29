use crate::rhi::Renderpass;

use winapi::um::d3d12::*;

/// DX12 renderpass yay
///
/// DX12 renderpasses are simpler than Vulkan renderpasses, because DX12 doesnt' care about mobile's tilled GPUs. This
/// struct basically holds the parameters for a DX12 renderpass
pub struct Dx12Renderpass {
    pub render_targets: Vec<D3D12_RENDER_PASS_RENDER_TARGET_DESC>,
    pub depth_stencil: D3D12_RENDER_PASS_DEPTH_STENCIL_DESC,
}

impl Dx12Renderpass {
    pub fn new(
        render_targets: Vec<D3D12_RENDER_PASS_RENDER_TARGET_DESC>,
        depth_stencil: D3D12_RENDER_PASS_DEPTH_STENCIL_DESC,
    ) -> Self {
        Dx12Renderpass {
            render_targets,
            depth_stencil,
        }
    }
}

impl Renderpass for Dx12Renderpass {}
