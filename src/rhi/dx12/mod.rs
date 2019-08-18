macro_rules! dx_call {
    ( $x:expr, $s:literal ) => {{
        if FAILED($x) {
            return Err(ErrorCode::CompilationError(String::from($s)));
        }
    }};
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
mod dx12_system_info;
mod dx12_utils;

pub use dx12_graphics_api::Dx12GraphicsApi;
