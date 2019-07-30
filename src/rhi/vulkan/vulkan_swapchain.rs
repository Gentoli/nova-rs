use ash::vk;

pub struct VulkanSwapchain {
    phys_device: vk::PhysicalDevice,
    surface_loader: ash::extensions::khr::Surface,
}

impl VulkanSwapchain {
    pub fn new(
        phys_device: vk::PhysicalDevice,
        surface_loader: ash::extensions::khr::Surface,
    ) -> ash::Result<VulkanSwapchain> {
        let mut swapchain = VulkanSwapchain {
            phys_device,
            surface_loader,
        };

        Ok(swapchain)
    }
}
