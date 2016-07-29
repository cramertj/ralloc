use prelude::*;

trait Breaker {
    /// Allocate _fresh_ space.
    ///
    /// "Fresh" means that the space is allocated through the breaker.
    ///
    /// The returned pointer is guaranteed to be aligned to `align`.
    fn alloc_fresh(pool: &mut Vec<Block>, size: usize, align: usize) -> Block;
}

/// Canonicalize a BRK request.
///
/// Syscalls can be expensive, which is why we would rather accquire more memory than necessary,
/// than having many syscalls acquiring memory stubs. Memory stubs are small blocks of memory,
/// which are essentially useless until merge with another block.
///
/// To avoid many syscalls and accumulating memory stubs, we BRK a little more memory than
/// necessary. This function calculate the memory to be BRK'd based on the necessary memory.
///
/// The return value is always greater than or equals to the argument.
#[inline]
fn canonicalize_brk(min: usize) -> usize {
    /// The BRK multiplier.
    ///
    /// The factor determining the linear dependence between the minimum segment, and the acquired
    /// segment.
    const BRK_MULTIPLIER: usize = 2;
    /// The minimum size to be BRK'd.
    const BRK_MIN: usize = 1024;
    /// The maximal amount of _extra_ elements.
    const BRK_MAX_EXTRA: usize = 4 * 65536;

    let res = cmp::max(BRK_MIN, min.saturating_add(cmp::min(BRK_MULTIPLIER * min, BRK_MAX_EXTRA)));

    // Make some handy assertions.
    debug_assert!(res >= min, "Canonicalized BRK space is smaller than the one requested.");

    res
}

struct Sbrk;

impl Breaker for Sbrk {
    fn ocbtain(size: usize, align: usize) -> (Block, Block, Block) {
        // Logging.
        log!(self.pool;self.pool.len(), "Obtaining a block of size, {}, and alignment {}.", size, align);

        // Calculate the canonical size (extra space is allocated to limit the number of system calls).
        let brk_size = canonicalize_brk(size) + align;

        // Use SBRK to allocate extra data segment. The alignment is used as precursor for our
        // allocated block. This ensures that it is properly memory aligned to the requested value.
        let (alignment_block, rest) = Block::brk(brk_size).align(align).unwrap();

        // Split the block to leave the excessive space.
        let (res, excessive) = rest.split(size);

        // Make some assertions.
        debug_assert!(res.aligned_to(align), "Alignment failed.");
        debug_assert!(res.size() + alignment_block.size() + excessive.size() == brk_size, "BRK memory leak.");

        (alignment_block, res, excessive)
    }
}
