use crate::rhi::*;

pub struct VulkanCommandAllocator;

impl CommandAllocator for VulkanCommandAllocator {
    type CommandList = ();

    fn create_command_list() -> Result<Self::CommandList, MemoryError> {
        unimplemented!()
    }
}
