//! We set out to make a completely new renderer for Minecraft aimed at giving
//! more control and vastly better tooling toshaderpack developers.
//! This is a rewrite of the old [Nova Renderer](https://github.com/NovaMods/nova-renderer) project
//! from C++ to Rust.

#![feature(async_await)]
#![feature(async_closure)]
#![feature(seek_convenience)]
#![feature(test)]
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
pub mod rhi;
pub mod settings;
pub mod shaderpack;
pub mod surface;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
