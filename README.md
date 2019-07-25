<h1 align="center">
  <img width=64px src="https://raw.githubusercontent.com/NovaMods/nova-rs/dc28cda0d5b534e89632602bac1bcddbda0a3c57/docs/images/nova-logo-128px-noborder.png">ova Renderer
</h1>

<p align="center">
  <a href="https://ci.appveyor.com/project/DethRaid/nova-rs/branch/master">
    <img alt="Windows Builds" src="https://ci.appveyor.com/api/projects/status/6jt834srdy3kjo7o/branch/master?svg=true">
  </a>

  <a href="https://travis-ci.org/NovaMods/nova-rs">
    <img alt="Linux Builds" src="https://travis-ci.org/NovaMods/nova-rs.svg?branch=master">
  </a>

  <a href="https://discord.gg/VGqtadw">
    <img alt="Discord Chat" src="https://img.shields.io/discord/193228267313037312.svg?color=7289DA&label=discord">
  </a>
</p>

## Purpose

We set out to make a completely new renderer for Minecraft aimed at giving more control and vastly better tooling to 
shaderpack developers. This is a rewrite of the old [Nova Renderer](https://github.com/NovaMods/nova-renderer) project 
from C++ to Rust.

### QuikFAQ

- Nova is a replacement for Minecraft's renderer built for shaderpack support and more shaderpack features. It is not:
  - Something to make Minecraft run faster
  - For older computers
  - A rewrite of the tick system
  - A rewrite of the audio system
  - Anything to do with the MC server
- [Why Rust?](docs/rust_faq.md)

## Development Status

Nova Renderer is a passion project by the developers and as such does not have any set deadlines or release dates.
We are still in early development of the Rust rewrite and things may change at any moment. That all being said, it
is still in active development.

## Developer Setup

[Contributing](docs/contributing.md).

You must have a Rust nightly-2019-07-19 toolchain setup. If you want to help develop the Nova Renderer you must also have
rustfmt installed for that toolchain so the automatically installed pre-commit hook works.

Please read the following to help get a feel for the project:

- [The Project Charter](docs/project_charter.md).
- [Git Rules](docs/git.md). **These must be followed for your PR to be accepted.**
- [Async Await Primer](docs/async_await.md). This project uses Rust's async/await feature throughout.

Please contact us on Discord if you want to help! We're very friendly :smile:
