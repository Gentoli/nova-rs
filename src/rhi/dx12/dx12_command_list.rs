use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::dx12::dx12_device::Dx12Device;
use crate::rhi::{
    dx12::{
        dx12_buffer::Dx12Buffer, dx12_descriptor_set::Dx12DescriptorSet, dx12_framebuffer::Dx12Framebuffer,
        dx12_pipeline::Dx12Pipeline, dx12_pipeline_interface::Dx12PipelineInterface, dx12_renderpass::Dx12Renderpass,
    },
    rhi_enums::PipelineStageFlags,
    CommandList, ResourceBarrier,
};

use std::sync::Arc;
use winapi::shared::winerror::SUCCEEDED;
use winapi::um::d3d12::{
    ID3D12GraphicsCommandList, ID3D12GraphicsCommandList1, ID3D12GraphicsCommandList2, ID3D12GraphicsCommandList3,
    ID3D12GraphicsCommandList4, D3D12_RENDER_PASS_FLAG_NONE,
};

pub struct Dx12CommandList {
    list: WeakPtr<ID3D12GraphicsCommandList>,
    list1: WeakPtr<ID3D12GraphicsCommandList1>,
    list2: WeakPtr<ID3D12GraphicsCommandList2>,
    list3: WeakPtr<ID3D12GraphicsCommandList3>,
    list4: WeakPtr<ID3D12GraphicsCommandList4>,
}

impl Dx12CommandList {
    pub fn new(list: WeakPtr<ID3D12GraphicsCommandList>) -> Self {
        // We don't really care if any of these succeed because `WeakPtr::cast` gives you an empty WeakPtr if the cast
        // fails
        // Don't worry, we check if the WeakPtrs are valid before we try to use them
        let (list1, _) = unsafe { list.cast::<ID3D12GraphicsCommandList1>() };
        let (list2, _) = unsafe { list.cast::<ID3D12GraphicsCommandList2>() };
        let (list3, _) = unsafe { list.cast::<ID3D12GraphicsCommandList3>() };
        let (list4, _) = unsafe { list.cast::<ID3D12GraphicsCommandList4>() };

        Dx12CommandList {
            list,
            list1,
            list2,
            list3,
            list4,
        }
    }
}

impl CommandList for Dx12CommandList {
    type Buffer = Dx12Buffer;
    type CommandList = Dx12CommandList;
    type Renderpass = Dx12Renderpass;
    type Framebuffer = Dx12Framebuffer;
    type Pipeline = Dx12Pipeline;
    type DescriptorSet = Dx12DescriptorSet;
    type PipelineInterface = Dx12PipelineInterface;

    fn resource_barriers(
        &self,
        stages_before_barrier: PipelineStageFlags,
        stages_after_barrier: PipelineStageFlags,
        barriers: Vec<ResourceBarrier>,
    ) {
    }

    fn copy_buffer(
        &self,
        destination_buffer: Dx12Buffer,
        destination_offset: u64,
        source_buffer: Dx12Buffer,
        source_offset: u64,
        num_bytes: u64,
    ) {
        unimplemented!()
    }

    fn execute_command_lists(&self, lists: Vec<Dx12CommandList>) {
        unimplemented!()
    }

    fn begin_renderpass(&self, renderpass: Dx12Renderpass, framebuffer: Dx12Framebuffer) {
        if !self.list4.is_null() {
            self.list4.BeginRenderPass(
                renderpass.render_targets.len() as u32,
                renderpass.render_targets.as_ptr(),
                &renderpass.depth_stencil,
                D3D12_RENDER_PASS_FLAG_NONE,
            );
        }
    }

    fn end_renderpass(&self) {
        if !self.list4.is_null() {
            self.list4.EndRenderPass();
        }
    }

    fn bind_pipeline(&self, pipeline: Dx12Pipeline) {
        unimplemented!()
    }

    fn bind_descriptor_sets(&self, descriptor_sets: Vec<Dx12DescriptorSet>, pipeline_interface: Dx12PipelineInterface) {
        unimplemented!()
    }

    fn bind_vertex_buffers(&self, buffers: Vec<Dx12Buffer>) {
        unimplemented!()
    }

    fn bind_index_buffer(&self, buffer: Dx12Buffer) {
        unimplemented!()
    }

    fn draw_indexed_mesh(&self, num_indices: u32, num_instances: u32) {
        unimplemented!()
    }
}
