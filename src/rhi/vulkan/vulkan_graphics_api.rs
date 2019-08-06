#![allow(unsafe_code)]
use crate::rhi::vulkan::vulkan_device;
use crate::rhi::vulkan::vulkan_device::VulkanDevice;
use crate::rhi::*;
use crate::surface::{Surface, SurfaceError};

use ash::extensions::ext::DebugReport;
use ash::version::{EntryV1_0, InstanceV1_0};
use ash::vk;
use log::debug;
use std::ffi;
use std::mem;
use std::os::raw;
use std::rc::Rc;

unsafe extern "system" fn vulkan_debug_callback(
    _: vk::DebugReportFlagsEXT,
    _: vk::DebugReportObjectTypeEXT,
    _: u64,
    _: usize,
    _: i32,
    _: *const raw::c_char,
    p_message: *const raw::c_char,
    _: *mut raw::c_void,
) -> u32 {
    debug!("{:?}", ffi::CStr::from_ptr(p_message));
    vk::FALSE
}

#[derive(Debug)]
pub enum VulkanGraphicsApiCreationError {
    VkFailedResult(vk::Result),
    LoadingError(Vec<String>),
    SurfaceError(SurfaceError),
}

/// TODO(janrupf): docs
pub struct VulkanGraphicsApi {
    instance: ash::Instance,
    debug_callback: Option<vk::DebugReportCallbackEXT>,
    entry: ash::Entry,
    surface: Rc<dyn Surface<vk::SurfaceKHR>>,
    surface_object: vk::SurfaceKHR,
    debug_utils: Option<ash::extensions::ext::DebugUtils>,
}

impl VulkanGraphicsApi {
    pub fn get_layer_names() -> Vec<*const u8> {
        (if cfg!(debug_assertions) {
            [ffi::CString::new("VK_LAYER_LUNARG_standard_validation").unwrap()]
        } else {
            []
        })
        .iter()
        .map(|n| n.as_ptr())
        .collect()
    }

    // TODO: This currently uses Vector3<f32> as a vertex, maybe this needs to be adjusted
    pub fn get_vertex_input_binding_description() -> Vec<vk::VertexInputBindingDescription> {
        (0..=6)
            .map(|v| vk::VertexInputBindingDescription {
                binding: v,
                stride: mem::size_of::<cgmath::Vector3<f32>>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            })
            .collect()
    }

    pub fn get_vertex_input_attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 3,
                binding: 0,
                format: vk::Format::R16G16_UNORM,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 4,
                binding: 0,
                format: vk::Format::R8G8_UNORM,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 5,
                binding: 0,
                format: vk::Format::R32_UINT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 6,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 0,
            },
        ]
    }

    pub fn new(
        application_name: String,
        application_version: (u32, u32, u32),
        mut surface: Rc<dyn Surface<vk::SurfaceKHR>>,
    ) -> Result<VulkanGraphicsApi, VulkanGraphicsApiCreationError> {
        let layer_names_raw = VulkanGraphicsApi::get_layer_names().as_slice();

        let extension_names_raw = vulkan_device::get_needed_extensions();

        let application_info = vk::ApplicationInfo::builder()
            .application_name(&application_name.into())
            .application_version(ash::vk_make_version!(
                application_version.0,
                application_version.1,
                application_version.2
            ))
            .engine_name(ffi::CString::new("Nova Renderer").as_c_str())
            .engine_version(ash::vk_make_version!(0, 1, 0))
            .api_version(ash::vk_make_version!(1, 1, 0))
            .build();

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(&layer_names_raw)
            .enabled_extension_names(&extension_names_raw)
            .build();

        let entry = match ash::Entry::new() {
            Err(error) => {
                return Err(VulkanGraphicsApiCreationError::LoadingError(
                    [error.unwrap().0].to_vec(),
                ));
            }
            Ok(v) => v,
        };

        let instance = match unsafe { entry.create_instance(&create_info, None) } {
            Err(error) => {
                return match error {
                    ash::InstanceError::LoadError(errors) => Err(VulkanGraphicsApiCreationError::LoadingError(
                        errors.iter().map(|raw| String::from(raw)).collect(),
                    )),
                    ash::InstanceError::VkError(result) => Err(VulkanGraphicsApiCreationError::VkFailedResult(result)),
                };
            }
            Ok(v) => v,
        };

        let debug_callback = if cfg!(debug_assertions) {
            let debug_info = vk::DebugReportCallbackCreateInfoEXT::builder()
                .flags(
                    vk::DebugReportFlagsEXT::ERROR
                        | vk::DebugReportFlagsEXT::WARNING
                        | vk::DebugReportFlagsEXT::PERFORMANCE_WARNING
                        | vk::DebugReportFlagsEXT::INFORMATION
                        | vk::DebugReportFlagsEXT::DEBUG,
                )
                .pfn_callback(Some(vulkan_debug_callback));

            let debug_report_loader = DebugReport::new(&entry, &instance);
            match unsafe { debug_report_loader.create_debug_report_callback(&debug_info, None) } {
                Err(error) => return Err(VulkanGraphicsApiCreationError::VkFailedResult(error)),
                Ok(v) => Some(v),
            }
        } else {
            None
        };

        let surface_object = match surface.platform_object() {
            Err(error) => return Err(VulkanGraphicsApiCreationError::SurfaceError(error)),
            Ok(v) => v,
        };

        let debug_utils = if cfg!(debug_assertions) {
            Some(ash::extensions::ext::DebugUtils::new(&entry, &instance))
        } else {
            None
        };

        Ok(VulkanGraphicsApi {
            instance,
            debug_callback,
            entry,
            surface,
            surface_object,
            debug_utils,
        })
    }
}

impl GraphicsApi for VulkanGraphicsApi {
    type Device = VulkanDevice;
    type PlatformSurface = vk::SurfaceKHR;

    fn get_adapters(&self) -> Vec<VulkanDevice> {
        let devices = unsafe { self.instance.enumerate_physical_devices() };
        if devices.is_err() {
            // TODO: The current trait doesn't allow us to return an error, what to do?
            return Vec::new();
        }

        devices
            .unwrap()
            .iter()
            .map(|d| {
                VulkanDevice::new(
                    self.instance,
                    *d,
                    self.surface.clone(),
                    self.debug_utils,
                    self.entry.clone(),
                )
            })
            .filter(|d| d.can_be_used_by_nova())
            .collect()
    }

    fn get_surface(&self) -> Rc<dyn Surface<Self::PlatformSurface>> {
        self.surface.clone()
    }
}
