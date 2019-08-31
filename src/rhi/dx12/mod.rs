use crate::rhi::dx12::com::WeakPtr;
use winapi::shared::guiddef::REFGUID;

fn get_uuid<T>(_: WeakPtr<T>) -> REFGUID {
    &T::uuidof()
}

pub mod dx12_graphics_api;

mod com;
mod dx12_buffer;
mod dx12_command_allocator;
mod dx12_command_list;
mod dx12_descriptor_pool;
mod dx12_descriptor_set;
mod dx12_device;
mod dx12_fence;
mod dx12_framebuffer;
mod dx12_image;
mod dx12_memory;
mod dx12_pipeline;
mod dx12_pipeline_interface;
mod dx12_queue;
mod dx12_renderpass;
mod dx12_semaphore;
mod dx12_swapchain;
mod dx12_system_info;
mod dx12_utils;
mod pso_utils;

pub use dx12_graphics_api::Dx12GraphicsApi;
