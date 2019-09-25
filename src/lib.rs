//! We set out to make a completely new renderer for Minecraft aimed at giving
//! more control and vastly better tooling toshaderpack developers.
//! This is a rewrite of the old [Nova Renderer](https://github.com/NovaMods/nova-renderer) project
//! from C++ to Rust.

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

pub mod async_utils;
pub mod core;
pub mod debugging;
pub mod fs;
pub mod loading;
pub mod logging;
pub mod rhi;
pub mod settings;
pub mod shaderpack;
pub mod surface;
