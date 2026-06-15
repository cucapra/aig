pub type NodeId = u32;

pub const INVERSION_MASK: NodeId = 0b0000_0000_0000_0000_0000_0000_0000_0001;
pub const NODE_ID_MASK: NodeId = 0b1111_1111_1111_1111_1111_1111_1111_1110;
pub const PRIMARY_INPUT_ID: NodeId = 0b1111_1111_1111_1111_1111_1111_1111_1110;

pub struct AigNode {
    pub left: NodeId,
    pub right: NodeId,
}

pub struct AigGraph {
    pub graph: Vec<AigNode>,
}

pub fn is_input(n: NodeId) -> bool {
    (n & NODE_ID_MASK) == PRIMARY_INPUT_ID
}

pub fn is_inverted(n: NodeId) -> bool {
    (n & INVERSION_MASK) != 0
}
