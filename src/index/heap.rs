/// Heap that stores all the data in insert order fashion
///
///
///
const MAX_BLOCK_SIZE: usize = 8192;
const MAX_DATA_SIZE: usize = MAX_BLOCK_SIZE - size_of::<usize>() - size_of::<u16>();

// Encode it?
// There are MAX_DATA_SIZE possible offsets. Which we need 13 bits to encode (2^13 = 8192)
// We also need to encode length of each entry. Length can be 1, also 13 bits.
// We need 26 bits, so u32. So we will use 16 bits for each.
const OFFSET_MASK: u32 = (1 << 16) - 1;
const LENGTH_MASK: u32 = !OFFSET_MASK;

// [0..15] - offset
// [16..31] - length
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct BlockOffset(u32);

impl BlockOffset {
    fn new(start: u16, len: u16) -> Self {
        let mut value = 0u32;
        value |= start as u32;
        value |= (len as u32) << 16;

        Self(value)
    }

    fn get_start(&self) -> usize {
        (self.0 & OFFSET_MASK) as usize
    }

    fn get_length(&self) -> usize {
        ((self.0 & LENGTH_MASK) >> 16) as usize
    }
}

// TODO: Find some better name
pub(crate) struct BufferPool {
    blocks: Vec<Block>,
}

struct Block {
    block_id: usize,
    free_offset: u16,
    data: [u8; MAX_DATA_SIZE],
}

impl Block {
    fn init(id: usize) -> Block {
        Self {
            block_id: id,
            data: [0u8; MAX_DATA_SIZE],
            free_offset: 0,
        }
    }

    fn can_allocate(&self, size: usize) -> bool {
        self.free_offset as usize + size <= self.data.len()
    }

    fn allocate(&mut self, data: &[u8]) -> BlockOffset {
        let start = self.free_offset;
        self.data[start as usize..start as usize + data.len()].copy_from_slice(data);

        // This cast is safe as self.data.len() has to always be equal or less than MAX_DATA_SIZE
        self.free_offset += data.len() as u16;

        BlockOffset::new(start, data.len() as u16)
    }

    fn read(&self, offset: BlockOffset) -> Option<&[u8]> {
        let start = offset.get_start();
        let len = offset.get_length();

        if (start + len) > self.data.len() {
            return None;
        }

        let data = &self.data[start..start + len];
        Some(data)
    }
}

impl BufferPool {
    pub(crate) fn new() -> BufferPool {
        BufferPool { blocks: vec![] }
    }

    /// Allocates data on heap, returns (block_id, offset)
    pub(crate) fn allocate(&mut self, data: &[u8]) -> (usize, BlockOffset) {
        let block_id = self.get_free_block(data.len());

        let offset = self.blocks[block_id].allocate(data);

        (block_id, offset)
    }

    /// Looks up for block that has enough space to contain {data_size} number of bytes.
    /// If there is none, it allocates new one.
    // TODO: Track last free block
    fn get_free_block(&mut self, data_size: usize) -> usize {
        assert!(data_size <= MAX_DATA_SIZE, "data exceeds block capacity");

        for block in &self.blocks {
            if block.can_allocate(data_size) {
                return block.block_id;
            }
        }

        self.init_block()
    }

    fn init_block(&mut self) -> usize {
        let block_id = self.blocks.len();
        self.blocks.push(Block::init(block_id));
        block_id
    }

    pub(crate) fn read(&self, block_id: usize, offset: BlockOffset) -> Option<&[u8]> {
        if block_id >= self.blocks.len() {
            return None;
        }

        self.blocks[block_id].read(offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_size_is_correct() {
        use std::mem::size_of;
        assert_eq!(size_of::<Block>(), MAX_BLOCK_SIZE);
    }

    #[test]
    fn test_new_heap_is_empty() {
        let heap = BufferPool::new();
        assert_eq!(heap.blocks.len(), 0);
    }

    #[test]
    fn test_allocate_creates_first_block() {
        let mut heap = BufferPool::new();
        let data = b"hello";

        let (block_id, offset) = heap.allocate(data);

        assert_eq!(block_id, 0);
        assert_eq!(heap.blocks.len(), 1);

        let res = heap.read(block_id, offset).unwrap();
        assert_eq!(data.as_slice(), res);
    }

    #[test]
    fn test_allocate_multiple_items_same_block() {
        let mut heap = BufferPool::new();

        let (block_id1, offset1) = heap.allocate(b"first");
        let (block_id2, offset2) = heap.allocate(b"second");

        assert_eq!(block_id1, 0);
        assert_eq!(block_id2, 0); // Same block
        assert_eq!(heap.blocks.len(), 1);

        assert_eq!(b"first".as_slice(), heap.read(block_id1, offset1).unwrap());
        assert_eq!(b"second".as_slice(), heap.read(block_id2, offset2).unwrap());
    }

    #[test]
    fn test_allocate_data_written_correctly() {
        let mut heap = BufferPool::new();
        let data = b"test data";

        let (block_id, offset) = heap.allocate(data);

        let stored = heap.read(block_id, offset).unwrap();
        assert_eq!(stored, data);
    }

    #[test]
    fn test_allocate_creates_new_block_when_full() {
        let mut heap = BufferPool::new();

        // Fill up first block with data close to MAX_DATA_SIZE
        let large_data = vec![0u8; MAX_DATA_SIZE - 10];
        heap.allocate(&large_data);

        // This should go to a new block
        let data = vec![1u8; 20];
        let (block_id, offset) = heap.allocate(&data);

        assert_eq!(block_id, 1);
        assert_eq!(heap.blocks.len(), 2);
    }

    #[test]
    fn test_block_can_allocate_exact_size() {
        let block = Block::init(0);
        assert!(block.can_allocate(MAX_DATA_SIZE));
    }

    #[test]
    fn test_block_cannot_allocate_oversized() {
        let block = Block::init(0);
        assert!(!block.can_allocate(MAX_DATA_SIZE + 1));
    }

    #[test]
    fn test_block_can_allocate_after_partial_use() {
        let mut block = Block::init(0);
        block.allocate(b"hello");

        assert!(block.can_allocate(MAX_DATA_SIZE - 5));
        assert!(!block.can_allocate(MAX_DATA_SIZE - 4));
    }

    #[test]
    fn test_block_init_has_correct_id() {
        let block = Block::init(42);
        assert_eq!(block.block_id, 42);
        assert_eq!(block.free_offset, 0);
    }

    #[test]
    fn test_get_free_block_initializes_first_block() {
        let mut heap = BufferPool::new();
        let block_id = heap.get_free_block(100);

        assert_eq!(block_id, 0);
        assert_eq!(heap.blocks.len(), 1);
    }

    #[test]
    fn test_get_free_block_reuses_existing_block() {
        let mut heap = BufferPool::new();
        heap.allocate(b"small");

        let block_id = heap.get_free_block(100);
        assert_eq!(block_id, 0); // Reuses first block
        assert_eq!(heap.blocks.len(), 1);
    }

    #[test]
    fn test_multiple_blocks_have_sequential_ids() {
        let mut heap = BufferPool::new();

        // Force creation of multiple blocks
        for i in 0..3 {
            let large_data = vec![i; MAX_DATA_SIZE];
            heap.allocate(&large_data);
        }

        assert_eq!(heap.blocks.len(), 3);
        assert_eq!(heap.blocks[0].block_id, 0);
        assert_eq!(heap.blocks[1].block_id, 1);
        assert_eq!(heap.blocks[2].block_id, 2);
    }

    #[test]
    fn test_allocate_empty_data() {
        let mut heap = BufferPool::new();
        let (block_id, offset) = heap.allocate(b"");

        assert_eq!(block_id, 0);
    }

    #[test]
    fn test_allocate_single_byte() {
        let mut heap = BufferPool::new();
        let (block_id, offset) = heap.allocate(b"x");

        assert_eq!(block_id, 0);
        assert_eq!(heap.blocks[0].data[0], b'x');
    }
}
