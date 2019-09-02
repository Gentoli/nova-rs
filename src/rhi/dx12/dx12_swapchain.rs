use crate::rhi::dx12::dx12_fence::Dx12Fence;
use crate::rhi::dx12::dx12_framebuffer::Dx12Framebuffer;
use crate::rhi::dx12::dx12_image::Dx12Image;
use crate::rhi::Swapchain;
use cgmath::Vector2;

struct Dx12Swapchain {}

impl<'a> Swapchain<'a> for Dx12Swapchain {
    type Framebuffer = Dx12Framebuffer;
    type Image = Dx12Image;
    type Fence = Dx12Fence;

    fn acquire_next_image(&self) -> u32 {
        unimplemented!()
    }

    fn present(&self, index: u32) {
        unimplemented!()
    }

    fn get_framebuffer(&self, index: u32) -> &'a Dx12Framebuffer {
        unimplemented!()
    }

    fn get_image(&self, index: u32) -> &'a Dx12Image {
        unimplemented!()
    }

    fn get_size(&self) -> Vector2<u32> {
        unimplemented!()
    }
}
