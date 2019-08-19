use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::Pipeline;
use winapi::um::d3d12::*;

pub struct Dx12Pipeline {
    pub pso: WeakPtr<ID3D12PipelineState>,
    pub root_sig: WeakPtr<ID3D12RootSignature>,
}

impl Pipeline for Dx12Pipeline {}
