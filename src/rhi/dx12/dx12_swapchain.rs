use crate::rhi::dx12::dx12_fence::Dx12Fence;
use crate::rhi::dx12::dx12_framebuffer::Dx12Framebuffer;
use crate::rhi::dx12::dx12_image::Dx12Image;
use crate::rhi::Swapchain;
use cgmath::Vector2;

struct Dx12Swapchain {}

impl<'a> Swapchain for Dx12Swapchain {
    type Framebuffer = Dx12Framebuffer;
    type Image = Dx12Image;
    type Fence = Dx12Fence;

    fn acquire_next_image() -> u32 {
        unimplemented!()
    }

    fn present(index: u32) {
        unimplemented!()
    }

    fn get_framebuffer(index: u32) -> &'a Self::Framebuffer {
        unimplemented!()
    }

    fn get_image(index: u32) -> &'a Self::Image {
        unimplemented!()
    }

    fn get_size() -> Vector2<u32> {
        unimplemented!()
    }
}
