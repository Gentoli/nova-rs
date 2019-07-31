use crate::surface;

use ash;
use ash::vk;
use std::rc::Rc;

pub struct VulkanSwapchain {
    phys_device: vk::PhysicalDevice,
    surface_loader: ash::extensions::khr::Surface,
    surface: Rc<dyn surface::Surface<vk::SwapchainKHR>>,
}

impl VulkanSwapchain {
    pub fn new(
        phys_device: vk::PhysicalDevice,
        surface_loader: ash::extensions::khr::Surface,
        surface: Rc<dyn surface::Surface<vk::SwapchainKHR>>,
    ) -> Result<VulkanSwapchain, ()> {
        let mut swapchain = VulkanSwapchain {
            phys_device,
            surface_loader,
            surface,
        };

        Ok(swapchain)
    }

    pub fn get_extend(&self) -> vk::Extent2D {
        let surface_size = self.surface.get_current_size();

        vk::Extent2D {
            width: surface_size.0,
            height: surface_size.1,
        }
    }
}
