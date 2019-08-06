#![allow(unsafe_code)]

use crate::rhi::shaderpack::*;
use crate::rhi::vulkan::vulkan_command_allocator::VulkanCommandAllocator;
use crate::rhi::vulkan::vulkan_descriptor_pool::VulkanDescriptorPool;
use crate::rhi::vulkan::vulkan_framebuffer::VulkanFramebuffer;
use crate::rhi::vulkan::vulkan_image::VulkanImage;
use crate::rhi::vulkan::vulkan_memory::VulkanMemory;
use crate::rhi::vulkan::vulkan_pipeline::VulkanPipeline;
use crate::rhi::vulkan::vulkan_pipeline_interface::VulkanPipelineInterface;
use crate::rhi::vulkan::vulkan_queue::VulkanQueue;
use crate::rhi::vulkan::vulkan_renderpass::VulkanRenderPass;
use crate::rhi::vulkan::vulkan_swapchain::VulkanSwapchain;
use crate::rhi::*;

use ash::extensions::ext::DebugReport;
use ash::extensions::khr::Swapchain;
use ash::version::{DeviceV1_0, InstanceV1_0};
use ash::vk;
use cgmath::Vector2;
use std::collections::HashMap;

#[cfg(all(unix, not(target_os = "android")))]
use ash::extensions::khr::XlibSurface;

#[cfg(windows)]
use ash::extensions::khr::Win32Surface;

#[derive(Clone, Copy, Debug)]
pub struct VulkanDeviceQueueFamilies {
    graphics_queue_family_index: u32,
    transfer_queue_family_index: u32,
    compute_queue_family_index: u32,
}

impl VulkanDeviceQueueFamilies {
    pub fn get(&self, queue_type: QueueType) -> u32 {
        match queue_type {
            QueueType::Graphics => self.graphics_queue_family_index,
            QueueType::Copy => self.transfer_queue_family_index,
            QueueType::Compute => self.compute_queue_family_index,
        }
    }
}

pub struct VulkanDevice {
    instance: ash::Instance,
    device: ash::Device,
    debug_utils: Option<ash::extensions::ext::DebugUtils>,

    queue_families: VulkanDeviceQueueFamilies,

    memory_properties: vk::PhysicalDeviceMemoryProperties,
    swapchain: VulkanSwapchain,

    allocated_memory: Vec<VulkanMemory>,
}

impl VulkanDevice {
    pub fn new(
        instance: ash::Instance,
        phys_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        debug_utils: Option<ash::extensions::ext::DebugUtils>,
        entry: ash::Entry,
    ) -> Result<VulkanDevice, DeviceCreationError> {
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
        let queue_family_props = unsafe { instance.get_physical_device_queue_family_properties(phys_device) };

        let mut graphics_queue_family_index = std::u32::MAX;
        let mut compute_queue_family_index = std::u32::MAX;
        let mut transfer_queue_family_index = std::u32::MAX;

        for (index, props) in queue_family_props.iter().enumerate() {
            if !unsafe { surface_loader.get_physical_device_surface_support(phys_device, index as u32, surface) } {
                continue;
            }

            if graphics_queue_family_index == std::u32::MAX && props.queue_flags & vk::QueueFlags::GRAPHICS != 0u32 {
                graphics_queue_family_index = index as u32
            }

            if compute_queue_family_index == std::u32::MAX && props.queue_flags & vk::QueueFlags::COMPUTE != 0u32 {
                compute_queue_family_index = index as u32
            }

            if transfer_queue_family_index == std::u32::MAX && props.queue_flags & vk::QueueFlags::TRANSFER != 0u32 {
                transfer_queue_family_index = index as u32
            }
        }

        let graphics_queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue_family_index)
            .queue_priorities(&[1.0f32])
            .build();

        let transfer_queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(transfer_queue_family_index)
            .queue_priorities(&[1.0f32])
            .build();
        let compute_queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(compute_queue_family_index)
            .queue_priorities(&[1.0f32])
            .build();

        let queue_create_infos = [
            graphics_queue_create_info,
            transfer_queue_create_info,
            compute_queue_create_info,
        ];

        let physical_device_features = vk::PhysicalDeviceFeatures::builder()
            .geometry_shader(true)
            .tessellation_shader(true)
            .sampler_anisotropy(true)
            .build();

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&physical_device_features)
            .enabled_extension_names(&[Swapchain::name()])
            .enabled_layer_names(VulkanGraphicsApi::get_layer_names().as_slice())
            .build();

        let swapchain = match VulkanSwapchain::new(phys_device, surface_loader.clone(), surface) {
            Err(_) => return Err(DeviceCreationError::Failed),
            Ok(v) => v,
        };

        match unsafe { instance.create_device(phys_device, &device_create_info, None) } {
            Err(_) => Err(DeviceCreationError::Failed),
            Ok(device) => Ok(VulkanDevice {
                instance,
                device,
                debug_utils,
                queue_families: VulkanDeviceQueueFamilies {
                    graphics_queue_family_index,
                    transfer_queue_family_index,
                    compute_queue_family_index,
                },
                memory_properties: unsafe { instance.get_physical_device_memory_properties(phys_device) },
                swapchain,

                allocated_memory: Vec::new(),
            }),
        }
    }

    fn supports_needed_extensions(&self) -> bool {
        let available_extensions =
            match unsafe { self.instance.enumerate_device_extension_properties(self.phys_device) } {
                Ok(extensions) => extensions,
                Err(_) => Vec::new(),
            };

        let mut needed_extensions = get_needed_extensions();

        for ext in available_extensions {
            needed_extensions.remove(ext.extension_name);
        }

        needed_extensions.is_empty()
    }

    fn get_manufacturer(&self, properties: &vk::PhysicalDeviceProperties) -> PhysicalDeviceManufacturer {
        match properties.vendor_id {
            // see http://vulkan.gpuinfo.org/
            // and https://www.reddit.com/r/vulkan/comments/4ta9nj/is_there_a_comprehensive_list_of_the_names_and/
            //  (someone find a better link here than reddit)
            0x1002 => PhysicalDeviceManufacturer::AMD,
            0x10DE => PhysicalDeviceManufacturer::Nvidia,
            0x8086 => PhysicalDeviceManufacturer::Intel,
            _ => PhysicalDeviceManufacturer::Other,
        }
    }

    fn find_memory_by_flags(&self, memory_flags: vk::MemoryPropertyFlags, exact: bool) -> Option<u32> {
        self.memory_properties
            .memory_types
            .iter()
            .find(|t| {
                if exact {
                    t.property_flags == memory_flags
                } else {
                    t.property_flags & memory_flags != 0
                }
            })
            .map(|t| t.heap_index)
    }

    fn nova_pixel_format_to_vulkan_format(pixel_format: shaderpack::PixelFormat) -> vk::Format {
        match pixel_format {
            shaderpack::PixelFormat::RGBA8 => vk::Format::R8G8B8A8_SNORM,
            shaderpack::PixelFormat::RGBA16F => vk::Format::R16G16B16A16_SFLOAT,
            shaderpack::PixelFormat::RGBA32F => vk::Format::R32G32B32A32_SFLOAT,
            shaderpack::PixelFormat::Depth => vk::Format::D32_SFLOAT,
            shaderpack::PixelFormat::DepthStencil => vk::Format::D24_UNORM_S8_UINT,
            /* _ => vk::Format::R10X6G10X6_UNORM_2PACK16, */
            /* FIXME: last arm found in Nova's C++ code, but unreachable here? */
        }
    }

    fn nova_descriptor_type_to_vulkan_type(descriptor_type: DescriptorType) -> vk::DescriptorType {
        match descriptor_type {
            DescriptorType::CombinedImageSampler => vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            DescriptorType::StorageBuffer => vk::DescriptorType::STORAGE_BUFFER,
            DescriptorType::UniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
        }
    }

    #[inline]
    fn nova_stage_flags_to_vulkan_flags(stage_flags: ShaderStageFlags) -> vk::ShaderStageFlags {
        // currently nova's ShaderStageFlags match their vulkan counterparts
        stage_flags as vk::ShaderStageFlags
    }

    fn nova_compare_op_to_vulkan_op(compare_op: CompareOp) -> vk::CompareOp {
        match compare_op {
            CompareOp::Never => vk::CompareOp::NEVER,
            CompareOp::Less => vk::CompareOp::LESS,
            CompareOp::LessEqual => vk::CompareOp::LESS_OR_EQUAL,
            CompareOp::Greater => vk::CompareOp::GREATER,
            CompareOp::GreaterEqual => vk::CompareOp::GREATER_OR_EQUAL,
            CompareOp::Equal => vk::CompareOp::EQUAL,
            CompareOp::NotEqual => vk::CompareOp::NOT_EQUAL,
            CompareOp::Always => vk::CompareOp::ALWAYS,
        }
    }

    fn nova_stencil_op_to_vulkan_op(stencil_op: StencilOp) -> vk::StencilOp {
        match stencil_op {
            StencilOp::Keep => vk::StencilOp::KEEP,
            StencilOp::Zero => vk::StencilOp::ZERO,
            StencilOp::Replace => vk::StencilOp::REPLACE,
            StencilOp::Incr => vk::StencilOp::INCREMENT_AND_CLAMP,
            StencilOp::IncrWrap => vk::StencilOp::INCREMENT_AND_WRAP,
            StencilOp::Decr => vk::StencilOp::DECREMENT_AND_CLAMP,
            StencilOp::DecrWrap => vk::StencilOp::DECREMENT_AND_WRAP,
            StencilOp::Invert => vk::StencilOp::INVERT,
        }
    }

    fn nova_blend_factor_to_vulkan_factor(blend_factor: BlendFactor) -> vk::BlendFactor {
        match blend_factor {
            BlendFactor::One => vk::BlendFactor::ONE,
            BlendFactor::Zero => vk::BlendFactor::ZERO,
            BlendFactor::SrcColor => vk::BlendFactor::SRC_COLOR,
            BlendFactor::DstColor => vk::BlendFactor::DST_COLOR,
            BlendFactor::OneMinusSrcColor => vk::BlendFactor::ONE_MINUS_SRC_COLOR,
            BlendFactor::OneMinusDstColor => vk::BlendFactor::ONE_MINUS_DST_COLOR,
            BlendFactor::SrcAlpha => vk::BlendFactor::SRC_ALPHA,
            BlendFactor::DstAlpha => vk::BlendFactor::DST_ALPHA,
            BlendFactor::OneMinusSrcAlpha => vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
            BlendFactor::OneMinusDstAlpha => vk::BlendFactor::ONE_MINUS_DST_ALPHA,
        }
    }

    fn create_descriptor_set_layouts(
        &self,
        all_bindings: &HashMap<String, ResourceBindingDescription>,
    ) -> Result<Vec<vk::DescriptorSetLayout>, MemoryError> {
        let mut bindings_by_set = Vec::new();
        for i in 0..all_bindings.len() {
            bindings_by_set.push(Vec::new());
        }

        for (name, binding) in all_bindings {
            if binding.set >= all_bindings.len() {
                log::error!("Found skipped descriptor set, Nova can't handle that properly");
                continue;
            }

            let descriptor_binding = vk::DescriptorSetLayoutBinding::builder()
                .binding(binding.binding)
                .descriptor_type(VulkanDevice::nova_descriptor_type_to_vulkan_type(
                    binding.descriptor_type,
                ))
                .descriptor_count(binding.count)
                .stage_flags(VulkanDevice::nova_stage_flags_to_vulkan_flags(binding.stages))
                .build();

            bindings_by_set.get_mut(binding.set).unwrap().push(descriptor_binding);
        }

        let mut layouts = Vec::new();
        for bindings in bindings_by_set {
            let create_info = vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(bindings.as_slice())
                .build();

            layouts.push(
                match unsafe { self.device.create_descriptor_set_layout(&create_info, None) } {
                    Err(result) => {
                        return match result {
                            vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(MemoryError::OutOfHostMemory),
                            vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(MemoryError::OutOfDeviceMemory),
                            _ => panic!("Invalid vk result returned: {:?}", result),
                        };
                    }
                    Ok(v) => v,
                },
            );
        }

        Ok(layouts)
    }

    fn create_shader_module(&self, source: &Vec<u32>) -> Result<vk::ShaderModule, vk::Result> {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(source.as_slice()).build();
        unsafe { self.device.create_shader_module(&create_info, None) }
    }

    pub fn get_queue_families(&self) -> VulkanDeviceQueueFamilies {
        self.queue_families
    }
}

impl Device for VulkanDevice {
    type Queue = VulkanQueue;
    type Memory = VulkanMemory;
    type CommandAllocator = VulkanCommandAllocator;
    type Image = VulkanImage;
    type Renderpass = VulkanRenderPass;
    type Framebuffer = VulkanFramebuffer;
    type PipelineInterface = VulkanPipelineInterface;
    type DescriptorPool = VulkanDescriptorPool;
    type Pipeline = VulkanPipeline;
    type Semaphore = ();
    type Fence = ();

    fn get_properties(&self) -> DeviceProperties {
        let properties: vk::PhysicalDeviceProperties =
            unsafe { self.instance.get_physical_device_properties(self.phys_device) };
        DeviceProperties {
            manufacturer: self.get_manufacturer(&properties),
            device_id: properties.device_id,
            device_name: String::from(properties.device_name),
            device_type: match properties.device_type {
                vk::PhysicalDeviceType::INTEGRATED_GPU => PhysicalDeviceType::Integrated,
                vk::PhysicalDeviceType::DISCRETE_GPU => PhysicalDeviceType::Discreet,
                vk::PhysicalDeviceType::VIRTUAL_GPU => PhysicalDeviceType::Virtual,
                vk::PhysicalDeviceType::CPU => PhysicalDeviceType::CPU,
                vk::PhysicalDeviceType::OTHER => PhysicalDeviceType::Other,
            },
            max_color_attachments: properties.limits.max_color_attachments,
        }
    }

    fn can_be_used_by_nova(&self) -> bool {
        if !self.supports_needed_extensions() {
            false
        }

        self.graphics_queue_family_index != std::usize::MAX
            && self.transfer_queue_family_index != std::usize::MAX
            && self.compute_queue_family_index != std::usize::MAX
    }

    fn get_free_memory(&self) -> u64 {
        // TODO: This just return all available memory, vulkan does not provide a way to query free memory
        //       on windows this could be done using DXGI (also works with vulkan according to stackoverflow),
        //       for linux a way has yet to be found
        let properties: vk::PhysicalDeviceMemoryProperties =
            unsafe { self.instance.get_physical_device_memory_properties(self.phys_device) };
        properties
            .memory_heaps
            .iter()
            .filter(|h| h.flags & vk::MemoryHeapFlags::DEVICE_LOCAL)
            .map(|h| h.size)
            .sum()
    }

    fn get_queue(&self, queue_type: QueueType, queue_index: u32) -> Result<Self::Queue, QueueGettingError> {
        if queue_index > 0 {
            // We only support queue index 0 at the moment
            Err(QueueGettingError::IndexOutOfRange)
        } else {
            let queue = unsafe {
                self.device
                    .get_device_queue(self.queue_families.get(queue_type), queue_index)
            };
            Ok(VulkanQueue { queue })
        }
    }

    fn allocate_memory(
        &self,
        size: u64,
        memory_usage: MemoryUsage,
        allowed_objects: ObjectType,
    ) -> Result<Self::Memory, AllocationError> {
        let memory_type_index = match memory_usage {
            MemoryUsage::DeviceOnly => {
                let i = self.find_memory_by_flags(vk::MemoryPropertyFlags::DEVICE_LOCAL, true);
                if i.is_none() {
                    self.find_memory_by_flags(vk::MemoryPropertyFlags::DEVICE_LOCAL, false)
                } else {
                    i
                }
            }
            MemoryUsage::LowFrequencyUpload => {
                let i = self.find_memory_by_flags(
                    vk::MemoryPropertyFlags::DEVICE_LOCAL | vk::MemoryPropertyFlags::HOST_VISIBLE,
                    false,
                );
                if i.is_none() {
                    self.find_memory_by_flags(vk::MemoryPropertyFlags::HOST_CACHED, false)
                } else {
                    i
                }
            }
            MemoryUsage::StagingBuffer => self.find_memory_by_flags(
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_CACHED,
                false,
            ),
        };

        let memory_type_index = match memory_type_index {
            None => return Err(AllocationError::NoSuitableMemoryFound),
            Some(i) => i,
        };

        let alloc_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(size)
            .memory_type_index(memory_type_index)
            .build();

        let allocated = {
            let allocated = unsafe { self.device.allocate_memory(&alloc_info, None) };
            match allocated {
                Err(result) => {
                    return match result {
                        vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(AllocationError::OutOfHostMemory),
                        vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(AllocationError::OutOfDeviceMemory),
                        vk::Result::ERROR_TOO_MANY_OBJECTS => Err(AllocationError::TooManyObjects),
                        vk::Result::ERROR_INVALID_EXTERNAL_HANDLE => Err(AllocationError::InvalidExternalHandle),
                        result => unreachable!("Invalid vk result returned: {:?}", result),
                    };
                }
                Ok(v) => v,
            }
        };

        match memory_usage {
            MemoryUsage::LowFrequencyUpload | MemoryUsage::StagingBuffer => {
                let mapped = unsafe {
                    self.device
                        .map_memory(allocated, 0, vk::WHOLE_SIZE, 0u32 as vk::MemoryMapFlags)
                };

                match mapped {
                    Err(result) => match result {
                        vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(AllocationError::OutOfDeviceMemory),
                        vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(AllocationError::OutOfHostMemory),
                        vk::Result::ERROR_MEMORY_MAP_FAILED => Err(AllocationError::MappingFailed),
                        result => unreachable!("Invalid vk result returned: {:?}", result),
                    },
                    Ok(mem) => Ok(VulkanMemory {
                        device: self.device.clone(),
                        memory: allocated,
                    }),
                }
            }
            _ => Ok(VulkanMemory {
                device: self.device.clone(),
                memory: allocated,
            }),
        }
    }

    fn create_command_allocator(
        &self,
        create_info: CommandAllocatorCreateInfo,
    ) -> Result<Self::CommandAllocator, MemoryError> {
        VulkanCommandAllocator::new(create_info, &self, self.instance.clone(), self.device.clone())
    }

    fn create_renderpass(&self, data: RenderPassCreationInfo) -> Result<Self::Renderpass, MemoryError> {
        let mut attachments = Vec::new();
        let mut attachment_references = Vec::new();

        let mut framebuffer_width = 0u32;
        let mut framebuffer_height = 0u32;

        let mut writes_to_backbuffer = false;
        for attachment in data.texture_outputs {
            match attachment.name.as_ref() {
                "Backbuffer" => {
                    writes_to_backbuffer = true;

                    let attachment_description = vk::AttachmentDescription::builder()
                        .format(self.swapchain.get_format())
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .load_op(vk::AttachmentLoadOp::CLEAR)
                        .store_op(vk::AttachmentStoreOp::STORE)
                        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                        .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .build();

                    attachments.push(attachment_description);

                    let attachment_reference = vk::AttachmentReference::builder()
                        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .attachment((attachments.len() - 1) as u32)
                        .build();

                    attachment_references.push(attachment_reference);
                    (framebuffer_width, framebuffer_height) = self.swapchain.get_size();

                    break;
                }
                _ => {
                    let attachment_description = vk::AttachmentDescription::builder()
                        .format(self.nova_pixel_format_to_vulkan_format(attachment.pixel_format))
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .load_op(match attachment.clear {
                            true => vk::AttachmentLoadOp::CLEAR,
                            false => vk::AttachmentLoadOp::LOAD,
                        })
                        .store_op(vk::AttachmentStoreOp::STORE)
                        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                        .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .build();

                    attachments.push(attachment_description);

                    let attachment_reference = vk::AttachmentReference::builder()
                        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .attachment((attachments.len() - 1) as u32)
                        .build();

                    attachment_references.push(attachment_reference);
                }
            }
        }

        let depth_attachment_reference = data.depth_texture.map(|texture| {
            let attachment_description = vk::AttachmentDescription::builder()
                .format(self.nova_pixel_format_to_vulkan_format(texture.pixel_format))
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(match texture.clear {
                    true => vk::AttachmentLoadOp::CLEAR,
                    false => vk::AttachmentLoadOp::LOAD,
                })
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build();

            attachments.push(attachment_description);

            vk::AttachmentReference::builder()
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .attachment((attachments.len() - 1) as u32)
                .build()
        });

        if framebuffer_width == 0 {
            panic!(
                "Framebuffer width for pass {} is 0. This is illegal! Make sure there is at least one attachment for this \
                 render pass, and ensure that all attachments used by this have a non-zero width",
                data.name
            )
        } else if framebuffer_height == 0 {
            panic!(
                "Framebuffer height for pass {} is 0. This is illegal! Make sure there is at least one attachment for this \
                 render pass, and ensure that all attachments used by this have a non-zero width",
                data.name
            )
        } else if writes_to_backbuffer && data.texture_outputs.len() > 1 {
            panic!(
                "Pass {} writes to the backbuffer and other textures. Passes that write to the backbuffer are not allowed to \
                 write to any other textures.",
                data.name
            )
        }

        let subpass_description = (match depth_attachment_reference {
            None => vk::SubpassDescription::builder(),
            Some(attachment) => vk::SubpassDescription::builder().depth_stencil_attachment(&attachment),
        })
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(attachment_references.as_slice())
        .build();

        let image_available_dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();

        let create_info = vk::RenderPassCreateInfo::builder()
            .subpasses(&[subpass_description])
            .dependencies(&[image_available_dependency])
            .attachments(attachments.as_slice())
            .build();

        let pass = VulkanRenderPass {
            vk_renderpass: match unsafe { self.device.create_render_pass(&create_info, None) } {
                Err(result) => {
                    return match result {
                        vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(MemoryError::OutOfHostMemory),
                        vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(MemoryError::OutOfDeviceMemory),
                        _ => panic!("Invalid result returned"),
                    };
                }
                Ok(v) => v,
            },
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: framebuffer_width,
                    height: framebuffer_height,
                },
            },
        };

        if cfg!(debug_assertions) {
            let object_name = vk::DebugUtilsObjectNameInfoEXT::builder()
                .object_type(vk::ObjectType::RENDER_PASS)
                .object_handle(pass.vk_renderpass as u64)
                .object_name(data.name.into())
                .build();

            if let Err(result) = unsafe {
                self.debug_utils
                    .unwrap()
                    .debug_utils_set_object_name(self.device.handle(), &object_name)
            } {
                log::debug!("debug_utils_set_object_name failed: {:?}", result)
            }
        }

        Ok(pass)
    }

    fn create_framebuffer(
        &self,
        renderpass: Self::Renderpass,
        attachments: Vec<Self::Image>,
        framebuffer_size: Vector2<f32>,
    ) -> Result<Self::Framebuffer, MemoryError> {
        let attachment_vies: Vec<vk::ImageView> = attachments.iter().map(|i| i.vk_image_view).collect();

        let create_info = vk::FramebufferCreateInfo::builder()
            .render_pass(renderpass.vk_renderpass)
            .attachments(attachment_vies.as_slice())
            .width(framebuffer_size.x as u32)
            .height(framebuffer_size.y as u32)
            .layers(1)
            .build();

        let vk_framebuffer = match unsafe { self.device.create_framebuffer(&create_info, None) } {
            Err(result) => {
                return match result {
                    vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(MemoryError::OutOfHostMemory),
                    vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(MemoryError::OutOfDeviceMemory),
                    _ => panic!("Invalid vk result returned: {:?}", result),
                };
            }
            Ok(v) => v,
        };

        Ok(VulkanFramebuffer {
            vk_framebuffer,
            width: framebuffer_size.x as u32,
            height: framebuffer_size.y as u32,
        })
    }

    fn create_pipeline_interface(
        &self,
        bindings: &HashMap<String, ResourceBindingDescription>,
        color_attachments: &Vec<TextureAttachmentInfo>,
        depth_texture: &Option<TextureAttachmentInfo>,
    ) -> Result<Self::PipelineInterface, MemoryError> {
        let layouts_by_set = self.create_descriptor_set_layouts(bindings)?;

        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(layouts_by_set.as_slice())
            .build();

        let pipeline_layout = match unsafe { self.device.create_pipeline_layout(&create_info, None) } {
            Err(result) => {
                return match result {
                    vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(MemoryError::OutOfHostMemory),
                    vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(MemoryError::OutOfDeviceMemory),
                    _ => panic!("Invalid vk result returned: {:?}", result),
                };
            }
            Ok(v) => v,
        };

        let mut attachment_descriptions = Vec::new();
        let mut attachment_references = Vec::new();

        let mut writes_to_backbuffer = false;
        for attachment in color_attachments {
            match attachment.name.as_ref() {
                "Backbuffer" => {
                    writes_to_backbuffer = true;

                    let attachment_description = vk::AttachmentDescription::builder()
                        .format(self.swapchain.get_format())
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .load_op(vk::AttachmentLoadOp::CLEAR)
                        .store_op(vk::AttachmentStoreOp::STORE)
                        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                        .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .build();

                    attachment_descriptions.push(attachment_description);

                    let attachment_reference = vk::AttachmentReference::builder()
                        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .attachment((attachment_descriptions.len() - 1) as u32)
                        .build();

                    attachment_references.push(attachment_reference);

                    break;
                }
                _ => {
                    let attachment_description = vk::AttachmentDescription::builder()
                        .format(VulkanDevice::nova_pixel_format_to_vulkan_format(
                            attachment.pixel_format,
                        ))
                        .samples(vk::SampleCountFlags::TYPE_1)
                        .load_op(match attachment.clear {
                            true => vk::AttachmentLoadOp::CLEAR,
                            false => vk::AttachmentLoadOp::LOAD,
                        })
                        .store_op(vk::AttachmentStoreOp::STORE)
                        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                        .initial_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .build();

                    attachment_descriptions.push(attachment_description);

                    let attachment_reference = vk::AttachmentReference::builder()
                        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .attachment((attachment_descriptions.len() - 1) as u32)
                        .build();

                    attachment_references.push(attachment_reference);
                }
            }
        }

        let depth_attachment_reference = depth_texture.map(|texture| {
            let attachment_description = vk::AttachmentDescription::builder()
                .format(self.nova_pixel_format_to_vulkan_format(texture.pixel_format))
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(match texture.clear {
                    true => vk::AttachmentLoadOp::CLEAR,
                    false => vk::AttachmentLoadOp::LOAD,
                })
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build();

            attachment_descriptions.push(attachment_description);

            vk::AttachmentReference::builder()
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .attachment((attachment_descriptions.len() - 1) as u32)
                .build()
        });

        let subpass_description = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(attachment_references.as_slice())
            .build();

        let image_available_dependency = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(0 as vk::AccessFlags)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
            .build();

        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .subpasses(&[subpass_description])
            .dependencies(&[image_available_dependency])
            .attachments(attachment_descriptions.as_slice())
            .build();

        Ok(VulkanPipelineInterface {
            vk_pipeline_layout: pipeline_layout,
            bindings: bindings.clone(),
            layouts_by_set,
        })
    }

    fn create_descriptor_pool(
        &self,
        num_sampled_images: u32,
        num_samplers: u32,
        num_uniform_buffers: u32,
    ) -> Result<Vec<Self::DescriptorPool>, DescriptorPoolCreationError> {
        let create_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(num_sampled_images + num_samplers + num_uniform_buffers)
            .pool_sizes(&[
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::SAMPLED_IMAGE,
                    descriptor_count: num_sampled_images,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::SAMPLER,
                    descriptor_count: num_samplers,
                },
                vk::DescriptorPoolSize {
                    ty: vk::DescriptorType::UNIFORM_BUFFER,
                    descriptor_count: num_uniform_buffers,
                },
            ])
            .build();

        Ok(vec![VulkanDescriptorPool {
            vk_descriptor_pool: match unsafe { self.device.create_descriptor_pool(&create_info, None) } {
                Err(result) => {
                    return match result {
                        vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(DescriptorPoolCreationError::OutOfHostMemory),
                        vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(DescriptorPoolCreationError::OutOfDeviceMemory),
                        vk::Result::ERROR_FRAGMENTATION_EXT => Err(DescriptorPoolCreationError::Fragmentation),
                        _ => panic!("Invalid vk result returned: {:?}", result),
                    };
                }
                Ok(v) => v,
            },
        }])
    }

    fn create_pipeline(
        &self,
        pipeline_interface: Self::PipelineInterface,
        data: PipelineCreationInfo,
    ) -> Result<Self::Pipeline, PipelineCreationError> {
        let mut shader_modules = HashMap::new();

        let insert_shader_module =
            |stage: vk::ShaderStageFlags, source: &Vec<u32>| -> Result<(), PipelineCreationError> {
                shader_modules.insert(
                    stage,
                    match self.create_shader_module(source) {
                        Err(result) => {
                            return match result {
                                vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(PipelineCreationError::OutOfHostMemory),
                                vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(PipelineCreationError::OutOfDeviceMemory),
                                vk::Result::ERROR_INVALID_SHADER_NV => Err(PipelineCreationError::InvalidShader),
                                _ => panic!("Invalid vk result returned: {:?}", result),
                            };
                        }
                        Ok(v) => v,
                    },
                );

                Ok(())
            };

        insert_shader_module(vk::ShaderStageFlags::VERTEX, &data.vertex_shader.source)?;

        if let Some(geometry_shader) = data.geometry_shader {
            insert_shader_module(vk::ShaderStageFlags::GEOMETRY, &geometry_shader.source)?;
        }

        if let Some(tessellation_control_shader) = data.tessellation_control_shader {
            insert_shader_module(
                vk::ShaderStageFlags::TESSELLATION_CONTROL,
                &tessellation_control_shader.source,
            )?;
        }

        if let Some(tessellation_evaluation_shader) = data.tessellation_evaluation_shader {
            insert_shader_module(
                vk::ShaderStageFlags::TESSELLATION_EVALUATION,
                &tessellation_evaluation_shader.source,
            )?;
        }

        if let Some(fragment_shader) = data.fragment_shader {
            insert_shader_module(vk::ShaderStageFlags::FRAGMENT, &fragment_shader.source)?;
        }

        let shader_stages: Vec<vk::PipelineShaderStageCreateInfo> = shader_modules
            .iter()
            .map(|(stage, module)| {
                vk::PipelineShaderStageCreateInfo::builder()
                    .stage(*stage)
                    .module(*module)
                    .name("main".into())
            })
            .collect();

        let vertex_input_state_create_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(VulkanGraphicsApi::get_vertex_input_binding_description().as_slice())
            .vertex_attribute_descriptions(VulkanGraphicsApi::get_vertex_input_attribute_descriptions().as_slice())
            .build();

        let input_assembly_create_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .primitive_restart_enable(false)
            .topology(match data.primitive_mode {
                PrimitiveTopology::Triangles => vk::PrimitiveTopology::TRIANGLE_LIST,
                PrimitiveTopology::Lines => vk::PrimitiveTopology::LINE_LIST,
            })
            .build();

        let swapchain_extend = self.swapchain.get_extend();

        let viewport = vk::Viewport::builder()
            .x(0f32)
            .y(0f32)
            .width(swapchain_extend.width as f32)
            .height(swapchain_extend.height as f32)
            .min_depth(0f32)
            .max_depth(1f32)
            .build();

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: swapchain_extend,
        };

        let viewport_state_create_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&[viewport])
            .scissors(&[scissor])
            .build();

        let cull_mode = if data.states.contains(&RasterizerState::InvertCulling)
            && !data.states.contains(&RasterizerState::DisableCulling)
        {
            vk::CullModeFlags::FRONT
        } else if !data.states.contains(&RasterizerState::InvertCulling)
            && data.states.contains(&RasterizerState::DisableCulling)
        {
            vk::CullModeFlags::NONE
        } else {
            panic!("Shaderpack data contains both disable and invert culling");
        };

        let rasterizer_create_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1f32)
            .cull_mode(cull_mode)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .depth_clamp_enable(false)
            .depth_bias_constant_factor(data.depth_bias)
            .depth_bias_slope_factor(data.slope_scaled_depth_bias)
            .build();

        let multisample_create_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .min_sample_shading(1f32)
            .alpha_to_coverage_enable(data.states.contains(&RasterizerState::EnableAlphaToCoverage))
            .alpha_to_one_enable(false)
            .build();

        let depth_stencil_create_info = {
            let depth_stencil_create_info = vk::PipelineDepthStencilStateCreateInfo::builder()
                .depth_test_enable(!data.states.contains(&RasterizerState::DisableDepthTest))
                .depth_write_enable(!data.states.contains(&RasterizerState::DisableDepthWrite))
                .depth_compare_op(VulkanDevice::nova_compare_op_to_vulkan_op(data.depth_func))
                .depth_bounds_test_enable(false)
                .stencil_test_enable(data.states.contains(&RasterizerState::EnableStencilTest));

            let depth_stencil_create_info = match data.front_face {
                Some(front_face) => depth_stencil_create_info.front(
                    vk::StencilOpState::builder()
                        .fail_op(VulkanDevice::nova_stencil_op_to_vulkan_op(front_face.fail_op))
                        .pass_op(VulkanDevice::nova_stencil_op_to_vulkan_op(front_face.pass_op))
                        .depth_fail_op(VulkanDevice::nova_stencil_op_to_vulkan_op(front_face.depth_fail_op))
                        .compare_op(VulkanDevice::nova_compare_op_to_vulkan_op(front_face.compare_op))
                        .compare_mask(front_face.compare_mask)
                        .write_mask(front_face.write_mask)
                        .build(),
                ),
                None => depth_stencil_create_info,
            };

            let depth_stencil_create_info = match data.back_face {
                Some(back_face) => depth_stencil_create_info.back(
                    vk::StencilOpState::builder()
                        .fail_op(VulkanDevice::nova_stencil_op_to_vulkan_op(back_face.fail_op))
                        .pass_op(VulkanDevice::nova_stencil_op_to_vulkan_op(back_face.pass_op))
                        .depth_fail_op(VulkanDevice::nova_stencil_op_to_vulkan_op(back_face.depth_fail_op))
                        .compare_op(VulkanDevice::nova_compare_op_to_vulkan_op(back_face.compare_op))
                        .compare_mask(back_face.compare_mask)
                        .write_mask(back_face.write_mask)
                        .build(),
                ),
                None => depth_stencil_create_info,
            };

            depth_stencil_create_info.build()
        };

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(true)
            .src_color_blend_factor(VulkanDevice::nova_blend_factor_to_vulkan_factor(data.src_blend_factor))
            .dst_color_blend_factor(VulkanDevice::nova_blend_factor_to_vulkan_factor(data.dst_blend_factor))
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(VulkanDevice::nova_blend_factor_to_vulkan_factor(data.alpha_src))
            .dst_alpha_blend_factor(VulkanDevice::nova_blend_factor_to_vulkan_factor(data.alpha_dst))
            .alpha_blend_op(vk::BlendOp::ADD)
            .build();

        let color_blend_create_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY) // TODO: Is this even required when `logic_op_enable = false`?
            .attachments(&[color_blend_attachment])
            .blend_constants([0f32, 0f32, 0f32, 0f32])
            .build();

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(shader_stages.as_slice())
            .vertex_input_state(&vertex_input_state_create_info)
            .input_assembly_state(&input_assembly_create_info)
            .viewport_state(&viewport_state_create_info)
            .rasterization_state(&rasterizer_create_info)
            .multisample_state(&multisample_create_info)
            .depth_stencil_state(&depth_stencil_create_info)
            .color_blend_state(&color_blend_create_info)
            .layout(pipeline_interface.vk_pipeline_layout)
            .render_pass(pipeline_interface.ren)
            .subpass(0)
            .base_pipeline_index(-1)
            .build();

        let pipeline = match unsafe {
            self.device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_create_info], None)
        } {
            Err((_, result)) => {
                return match result {
                    vk::Result::ERROR_OUT_OF_HOST_MEMORY => Err(PipelineCreationError::OutOfHostMemory),
                    vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => Err(PipelineCreationError::OutOfDeviceMemory),
                    vk::Result::ERROR_INVALID_SHADER_NV => Err(PipelineCreationError::InvalidShader),
                    _ => panic!("Invalid vk result returned: {:?}", result),
                };
            }
            Ok(mut v) => v.remove(0),
        };

        if cfg!(debug_assertions) {
            let object_name = vk::DebugUtilsObjectNameInfoEXT::builder()
                .object_handle(pipeline as u64)
                .object_type(vk::ObjectType::PIPELINE)
                .object_name(data.name.clone().into())
                .build();

            if let Err(result) = unsafe {
                self.debug_utils
                    .unwrap()
                    .debug_utils_set_object_name(self.device.handle(), &object_name)
            } {
                log::debug!("debug_utils_set_object_name failed: {:?}", result);
            }
        }

        Ok(VulkanPipeline { vk_pipeline: pipeline })
    }

    fn create_image(&self, data: TextureCreateInfo) -> Result<Self::Image, MemoryError> {
        unimplemented!()
    }

    fn create_semaphore(&self) -> Result<Self::Semaphore, MemoryError> {
        unimplemented!()
    }

    fn create_semaphores(&self, count: u32) -> Result<Vec<Self::Semaphore>, MemoryError> {
        unimplemented!()
    }

    fn create_fence(&self) -> Result<Self::Fence, MemoryError> {
        unimplemented!()
    }

    fn create_fences(&self, count: u32) -> Result<Vec<Self::Fence>, MemoryError> {
        unimplemented!()
    }

    fn wait_for_fences(&self, fences: Vec<Self::Fence>) {
        unimplemented!()
    }

    fn reset_fences(&self, fences: Vec<Self::Fence>) {
        unimplemented!()
    }

    fn update_descriptor_sets(&self, updates: Vec<DescriptorSetWrite>) {
        unimplemented!()
    }
}

#[cfg(all(unix, not(target_os = "android")))]
pub fn get_needed_extensions() -> Vec<*const u8> {
    vec![
        Swapchain::name().as_ptr(),
        XlibSurface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}

#[cfg(windows)]
pub fn get_needed_extensions() -> Vec<*const u8> {
    vec![
        Swapchain::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugReport::name().as_ptr(),
    ]
}
