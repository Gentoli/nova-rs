#![allow(unsafe_code)]

pub mod barriers;
pub mod enum_conversions;
pub mod shaders;

#[macro_use]
use log::*;

use crate::rhi::{DescriptorType, QueueType, ResourceBarrier, ResourceState};
use crate::ErrorCode;
use spirv_cross::spirv;
use std::ffi::CStr;
use std::ptr::null;
use winapi::shared::ntdef::{LANG_NEUTRAL, MAKELANGID, SUBLANG_DEFAULT};
use winapi::shared::winerror::HRESULT;
use winapi::um::winbase::{FormatMessageA, FORMAT_MESSAGE_FROM_SYSTEM};

impl From<HRESULT> for ErrorCode<HRESULT> {
    fn from(hr: i32) -> Self {
        let message = unsafe {
            let mut error_message_buffer: [char; 1024] = ['\0'; 1024];

            FormatMessageA(
                FORMAT_MESSAGE_FROM_SYSTEM,
                null(),
                hr as u32,
                MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as u32,
                error_message_buffer as _,
                1024,
                null() as _,
            );

            unsafe { CStr::from_ptr(error_message_buffer as _) }
                .to_str()
                .unwrap()
                .to_string()
        };

        ErrorCode(hr, message)
    }
}
