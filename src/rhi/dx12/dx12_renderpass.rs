use crate::rhi::Renderpass;

use winapi::um::d3d12;

/// DX12 renderpass yay
///
/// DX12 renderpasses are simpler than Vulkan renderpasses, because DX12 doesnt' care about mobile's tilled GPUs. This
/// struct basically holds the parameters for a DX12 renderpass
pub struct Dx12Renderpass {
    render_targets: d3d12::D3D12_RENDER_PASS_RENDER_TARGET_DESC,
    depth_stencil: d3d12::D3D12_RENDER_PASS_DEPTH_STENCIL_DESC,
}

impl Dx12Renderpass {
    pub fn new(
        render_targets: d3d12::D3D12_RENDER_PASS_RENDER_TARGET_DESC,
        depth_stencil: d3d12::D3D12_RENDER_PASS_DEPTH_STENCIL_DESC,
    ) -> Self {
        Dx12Renderpass {
            render_targets,
            depth_stencil,
        }
    }
}

impl Renderpass for Dx12Renderpass {}
