use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::{
    dx12::{dx12_descriptor_set::Dx12DescriptorSet, dx12_pipeline_interface::Dx12PipelineInterface},
    DescriptorPool,
};
use winapi::um::d3d12::*;

pub struct Dx12DescriptorPool {
    pub sampler_heap: WeakPtr<ID3D12DescriptorHeap>,
    pub data_heap: WeakPtr<ID3D12DescriptorHeap>,
}

impl DescriptorPool for Dx12DescriptorPool {
    type PipelineInterface = Dx12PipelineInterface;
    type DescriptorSet = Dx12DescriptorSet;

    fn create_descriptor_sets(&self, pipeline_interface: Dx12PipelineInterface) -> Vec<Dx12DescriptorSet> {
        unimplemented!()
    }
}
