//! We set out to make a completely new renderer for Minecraft aimed at giving
//! more control and vastly better tooling to shaderpack developers.
//! This is a rewrite of the old [Nova Renderer](https://github.com/NovaMods/nova-renderer) project
//! from C++ to Rust.

#![feature(async_await)]
#![feature(async_closure)]
#![feature(seek_convenience)]
#![feature(test)]
#![feature(type_alias_impl_trait)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::float_cmp)]
#![deny(nonstandard_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(unused)]

pub mod core;

pub mod debugging;
pub mod fs;
pub mod loading;
pub mod logging;
pub mod mesh;
pub mod rhi;
pub mod settings;
pub mod shaderpack;
pub mod surface;

struct ErrorCode<T>(T, String);

impl<T> From<spirv_cross::ErrorCode> for ErrorCode<T>
where
    T: Default,
{
    fn from(err: spirv_cross::ErrorCode) -> Self {
        match err {
            spirv_cross::ErrorCode::Unhandled => (T::default(), String::from("Unhandled error :(")),
            spirv_cross::ErrorCode::CompilationError(msg) => (T::default(), msg),
        }
    }
}
