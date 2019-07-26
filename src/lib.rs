#![feature(async_await)]
#![feature(async_closure)]
#![feature(seek_convenience)]
#![feature(test)]
#![deny(nonstandard_style)]
#![deny(future_incompatible)]
#![deny(rust_2018_idioms)]
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
