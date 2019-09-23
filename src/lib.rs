//! We set out to make a completely new renderer for Minecraft aimed at giving
//! more control and vastly better tooling toshaderpack developers.
//! This is a rewrite of the old [Nova Renderer](https://github.com/NovaMods/nova-renderer) project
//! from C++ to Rust.

#![feature(async_await)]
#![feature(async_closure)]
#![feature(box_syntax)]
#![feature(seek_convenience)]
#![feature(test)]
#![feature(type_alias_impl_trait)]
#![allow(clippy::cognitive_complexity)]
#![allow(clippy::float_cmp)]
#![deny(nonstandard_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(missing_docs)]
#![warn(unused)]
// Rust warnings
#![warn(unused)]
#![deny(nonstandard_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_idioms)]
// Clippy warnings
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![allow(clippy::cognitive_complexity)] // This is dumb
#![allow(clippy::multiple_crate_versions)] // Dependencies are hard
// Clippy Restrictions
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::get_unwrap)]
#![warn(clippy::multiple_inherent_impl)]
#![warn(clippy::option_unwrap_used)]
#![warn(clippy::print_stdout)]
#![warn(clippy::result_unwrap_used)]
#![warn(clippy::unimplemented)]
#![warn(clippy::wildcard_enum_match_arm)]
#![warn(clippy::wrong_pub_self_convention)]

pub mod async_utils;
pub mod core;
pub mod debugging;
pub mod fs;
pub mod loading;
pub mod logging;
pub mod mesh;
pub mod renderer;
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
