#![allow(unsafe_code)]

use crate::rhi::shaderpack::*;
use crate::rhi::vulkan::vulkan_command_allocator::VulkanCommandAllocator;
use crate::rhi::vulkan::vulkan_memory::VulkanMemory;
use crate::rhi::vulkan::vulkan_queue::VulkanQueue;
use crate::rhi::vulkan::vulkan_renderpass::VulkanRenderPass;
use crate::rhi::vulkan::vulkan_swapchain::VulkanSwapchain;
use crate::rhi::*;

use crate::rhi::vulkan::vulkan_framebuffer::VulkanFramebuffer;
use crate::rhi::vulkan::vulkan_image::VulkanImage;
use crate::rhi::vulkan::vulkan_pipeline::VulkanPipeline;
use crate::rhi::vulkan::vulkan_pipeline_interface::VulkanPipelineInterface;
use ash::version::DeviceV1_0;
use ash::vk;
use cgmath::Vector2;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::process::exit;

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
        device: ash::Device,
        debug_utils: Option<ash::extensions::ext::DebugUtils>,
        graphics_queue_family_index: u32,
        transfer_queue_family_index: u32,
        compute_queue_family_index: u32,
        swapchain: VulkanSwapchain,
        memory_properties: vk::PhysicalDeviceMemoryProperties,
    ) -> Result<VulkanDevice, DeviceCreationError> {
        let mut device = VulkanDevice {
            instance,
            device,
            debug_utils,
            queue_families: VulkanDeviceQueueFamilies {
                graphics_queue_family_index,
                transfer_queue_family_index,
                compute_queue_family_index,
            },
            memory_properties,
            swapchain,

            allocated_memory: Vec::new(),
        };

        Ok(device)
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
    type DescriptorPool = ();
    type Pipeline = VulkanPipeline;
    type Semaphore = ();
    type Fence = ();

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
                .object_type(vk::ObjectType::IMAGE)
                .object_handle(pass.vk_renderpass as u64)
                .object_name(data.name.into())
                .build();

            match unsafe {
                self.debug_utils
                    .unwrap()
                    .debug_utils_set_object_name(self.device.handle(), &object_name)
            } {
                Err(err) => log::debug!("debug_utils_set_object_name failed: {:?}", err),
                Ok(_) => {}
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
        unimplemented!()
    }

    fn create_pipeline(
        &self,
        pipeline_interface: Self::PipelineInterface,
        data: PipelineCreationInfo,
    ) -> Result<Self::Pipeline, PipelineCreationError> {
        unimplemented!()
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
