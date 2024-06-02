use crate::game::block::Block;

pub struct ChunkData {
    blocks: Box<[Block]>,
    num_blocks: u32,
}
