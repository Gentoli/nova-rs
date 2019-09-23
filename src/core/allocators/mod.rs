mod block_allocator;

use crate::rhi::MemoryAllocationInfo;

pub use block_allocator::BlockAllocationStrategy;

pub trait AllocationStrategy {
    fn allocate(&mut self, size: u64, info: &mut MemoryAllocationInfo) -> bool;

    fn describe_allocation(&self, info: MemoryAllocationInfo) -> String;
}

pub fn align(value: u64, alignment: u64) -> u64 {
    ((value + (alignment - 1)) & !(alignment - 1))
}
