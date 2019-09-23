use crate::core::allocators::{align, AllocationStrategy};
use crate::rhi::MemoryAllocationInfo;
use std::rc::Rc;
use std::sync::Arc;

/// Simple block allocator
///
/// A block allocator keeps an internal list of free regions of memory, also called "blocks". When you allocate with a
/// block allocator, the allocator finds the smallest block that your allocation will fit into, splits it into the part
/// where your allocation will reside and the remaining free space, and returns the allocator
///
/// Block allocators support freeing allocations
///
/// This block allocator doesn't compact allocations, because that requires moving everything around which entails that
/// I support that in the first place. We'll need some more indirection to get that working well
///
/// This block allocator must be externally synchronized
pub struct BlockAllocationStrategy {
    head: Box<Block>,

    memory_size: u64,
    alignment: u64,

    allocated: u64,

    next_block_id: u64,
}

impl BlockAllocationStrategy {
    /// Creates anew block allocator which allocates out of `total_size` bytes, ensuring that every allocation is
    /// aligned to `alignment`
    ///
    /// # Parameters
    ///
    /// * `total_size` - The total number of bytes that this block allocator can allocate from
    /// * `alignment` - Alignment, in bytes, of all allocations from this block allocator. If this is zero or 1,
    /// allocations should be considered unaligned
    pub fn new(total_size: u64, alignment: u64) -> Self {
        let mut block = BlockAllocationStrategy {
            head: Default::default(),
            memory_size: total_size,
            alignment,
            allocated: 0,
            next_block_id: 0,
        };

        block.head = box block.make_new_block(0, total_size);

        block
    }

    /// Create an empty block with the provided offset and size
    ///
    /// # Parameters
    ///
    /// * `offset` - Offset from the start of the managed space to where this Block resides
    /// * `size` - How many bytes this Block speaks for
    fn make_new_block(&mut self, offset: u64, size: u64) -> Block {
        let block = Block {
            id: self.next_block_id,
            size,
            offset,
            previous: Default::default(),
            next: Default::default(),
            free: true,
        };

        self.next_block_id += 1;

        block
    }
}

impl AllocationStrategy for BlockAllocationStrategy {
    fn allocate(&mut self, size: u64, info: &mut MemoryAllocationInfo) -> bool {
        let size = align(size, self.alignment);

        let free_size = self.memory_size - self.allocated;
        if free_size < size {
            return false;
        }

        let mut best_fit: Option<Box<Block>> = None;
        let mut current = &self.head.clone();
        while !current.is_empty() {
            if !current.free || current.size < size {
                return false;
            }

            if let Some(best_fit_block) = &best_fit {
                if best_fit_block.size > current.size {
                    // TODO: How the fuck do I box
                    // best_fit = Some(best_fit_block.);
                }
            }
        }

        unimplemented!();
    }

    fn describe_allocation(&self, info: &MemoryAllocationInfo) -> String {
        unimplemented!();
    }
}

struct Block {
    pub id: u64,
    pub size: u64,
    pub offset: u64,
    pub previous: Rc<Block>,
    pub next: Rc<Block>,
    pub free: bool,
}
