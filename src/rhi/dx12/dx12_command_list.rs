#![allow(unsafe_code)]

use crate::rhi::dx12::com::WeakPtr;
use crate::rhi::{
    dx12::{
        dx12_buffer::Dx12Buffer, dx12_descriptor_set::Dx12DescriptorSet, dx12_framebuffer::Dx12Framebuffer,
        dx12_pipeline::Dx12Pipeline, dx12_pipeline_interface::Dx12PipelineInterface, dx12_renderpass::Dx12Renderpass,
    },
    rhi_enums::PipelineStageFlags,
    CommandList, ResourceBarrier,
};

use std::ptr;
use winapi::um::d3d12::*;

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
            let mut render_target_descs = Vec::<D3D12_RENDER_PASS_RENDER_TARGET_DESC>::new();
            render_target_descs.reserve(renderpass.render_targets.len());
            for (i, render_target_info) in renderpass.render_targets.iter().enumerate() {
                let new_desc = D3D12_RENDER_PASS_RENDER_TARGET_DESC {
                    cpuDescriptor: framebuffer.color_attachments[i],
                    BeginningAccess: render_target_info.beginning_access,
                    EndingAccess: render_target_info.ending_access,
                };

                render_target_descs.push(new_desc);
            }

            match renderpass.depth_stencil {
                Some(ds_info) => {
                    let depth_stencil_desc = D3D12_RENDER_PASS_DEPTH_STENCIL_DESC {
                        cpuDescriptor: framebuffer.depth_attachment.unwrap(),
                        DepthBeginningAccess: ds_info.beginning_access,
                        StencilBeginningAccess: ds_info.beginning_access,
                        DepthEndingAccess: ds_info.ending_access,
                        StencilEndingAccess: ds_info.ending_access,
                    };

                    self.list4.BeginRenderPass(
                        render_target_descs.len() as u32,
                        render_target_descs.as_ptr(),
                        &depth_stencil_desc,
                        D3D12_RENDER_PASS_FLAG_NONE,
                    );
                }
                None => {
                    self.list4.BeginRenderPass(
                        render_target_descs.len() as u32,
                        render_target_descs.as_ptr(),
                        ptr::null(),
                        D3D12_RENDER_PASS_FLAG_NONE,
                    );
                }
            }
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
