use crate::rhi::Renderpass;

/// DX12 renderpass yay
///
/// DX12 renderpasses are simpler than Vulkan renderpasses, because DX12 doesnt' care about mobile's tilled GPUs. This
/// struct basically holds the parameters for a DX12 renderpass
pub struct Dx12Renderpass {
    // TODO: Give this struct members when `winapi-rs` support `ID3D12GraphicsCommandList4`
}

impl Renderpass for Dx12Renderpass {}
