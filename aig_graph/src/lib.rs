type NodeId = u32;

const INVERSION_MASK: NodeId = 0b0000_0000_0000_0000_0000_0000_0000_0001;
const NODE_ID_MASK: NodeId = 0b1111_1111_1111_1111_1111_1111_1111_1110;
const PRIMARY_INPUT_ID: NodeId = 0b1111_1111_1111_1111_1111_1111_1111_1110;

struct AigNode {
    left: NodeId,
    right: NodeId,
}

struct AigGraph {
    graph: Vec<AigNode>,
}

fn is_input(n: NodeId) -> bool {
    (n & NODE_ID_MASK) == PRIMARY_INPUT_ID
}

fn is_inverted(n: NodeId) -> bool {
    (n & INVERSION_MASK) != 0
}

