const MAX_NODE_SIZE: usize = 8096;

struct MemManager {
    blocks: Vec<Node>,
}

// TODO: Make it thread safe
impl MemManager {
    fn new() -> MemManager {
        MemManager { blocks: Vec::new() }
    }

    fn allocate_internal(&mut self) -> usize {
        let block_id = self.blocks.len();
        self.blocks.push(Node::Internal(Internal::new(block_id)));
        block_id
    }

    fn allocate_leaf(&mut self) -> usize {
        let block_id = self.blocks.len();
        self.blocks.push(Node::Leaf(Leaf::new(block_id)));
        block_id
    }
}

// May add internal if siblings pointers are benefitcial
enum Node {
    Internal(Internal),
    Leaf(Leaf),
}

/// Representation of internal node
struct Internal {
    block_id: usize,
    /// Pointers down the tree. This ought to be sorted.
    entries: Vec<InternalEntry>,
}

struct InternalEntry {
    /// Indexed data value
    key: Vec<u8>,

    /// Block id that potentially contains the data
    block_ptr: usize,
}

struct Leaf {
    leaf_id: usize,
    entries: Vec<LeafEntry>,
}

struct LeafEntry {
    key: Vec<u8>,
    tid: Tid,
}

/// Pointer to data page containing the actual data
struct Tid {
    block_id: usize,
    offset: u16,
}

impl Internal {
    fn new(block_id: usize) -> Internal {
        Internal {
            block_id,
            entries: vec![],
        }
    }
}

impl Leaf {
    fn new(leaf_id: usize) -> Leaf {
        Leaf {
            leaf_id,
            entries: vec![],
        }
    }
}
