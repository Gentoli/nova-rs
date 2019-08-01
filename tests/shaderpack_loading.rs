use futures::executor::block_on;
use nova_rs::shaderpack;
use nova_rs::shaderpack::{load_nova_shaderpack, ShaderpackLoadingFailure};
use std::path::PathBuf;

#[test]
fn default_nova_shaderpack() -> Result<(), ShaderpackLoadingFailure> {
    let parsed = block_on(load_nova_shaderpack(PathBuf::from(
        "tests/data/shaderpacks/nova/DefaultShaderpack",
    )))?;

    Ok(())
}
