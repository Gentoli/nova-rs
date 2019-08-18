//! We set out to make a completely new renderer for Minecraft aimed at giving
//! more control and vastly better tooling toshaderpack developers.
//! This is a rewrite of the old [Nova Renderer](https://github.com/NovaMods/nova-renderer) project
//! from C++ to Rust.

// Rust features
#![feature(async_closure)]
#![feature(seek_convenience)]
#![feature(test)]
// Rust warnings
#![warn(unused)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_idioms)]
#![deny(unsafe_code)] // Most is safe, but the RHI needs unsafe
// Clippy warnings
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::restriction)]
// Annoying regular clippy warnings
#![allow(clippy::cast_possible_truncation)] // Annoying
#![allow(clippy::cast_possible_wrap)] // Annoying
#![allow(clippy::cast_precision_loss)] // Annoying
#![allow(clippy::cast_sign_loss)] // Annoying
#![allow(clippy::cognitive_complexity)] // This is dumb
#![allow(clippy::doc_markdown)] // Too picky
#![allow(clippy::module_name_repetitions)] // Causes name conflicts
#![allow(clippy::pub_enum_variant_names)] // Conventional names are redundant
// Annoying/irrelevant clippy Restrictions
#![allow(clippy::decimal_literal_representation)]
#![allow(clippy::else_if_without_else)]
#![allow(clippy::float_arithmetic)]
#![allow(clippy::float_cmp_const)]
#![allow(clippy::implicit_return)]
#![allow(clippy::integer_arithmetic)]
#![allow(clippy::integer_division)]
#![allow(clippy::mem_forget)] // Useful for FFI
#![allow(clippy::missing_docs_in_private_items)]
#![allow(clippy::missing_inline_in_public_items)]
#![allow(clippy::shadow_reuse)]
#![allow(clippy::shadow_same)]
#![allow(clippy::unimplemented)] // Annoying during early prototyping
#![allow(clippy::wildcard_enum_match_arm)]

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
