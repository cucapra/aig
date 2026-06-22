use std::collections::HashMap;
use std::ops::Index;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u32);

const INVERSION_MASK: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0001;
const NODE_ID_MASK: u32 = 0b1111_1111_1111_1111_1111_1111_1111_1110;
const INPUT_NODE_MARKER: NodeId = NodeId(NODE_ID_MASK);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AigNode {
    left: NodeId,
    right: NodeId,
}

#[derive(Debug)]
pub struct AigGraph {
    graph: Vec<AigNode>,
    and_hash: HashMap<AigNode, NodeId>,
    outputs: Vec<NodeId>,
}

impl NodeId {
    pub const FALSE: NodeId = NodeId(0);
    pub const TRUE: NodeId = NodeId(1);

    pub fn is_inverted(self) -> bool {
        (self.0 & INVERSION_MASK) != 0
    }

    pub fn regular(self) -> Self {
        Self(self.0 & NODE_ID_MASK)
    }

    pub fn invert(self) -> Self {
        Self(self.0 ^ INVERSION_MASK)
    }

    pub fn is_const(self) -> bool {
        self.regular() == NodeId::FALSE
    }

    pub fn is_false(self) -> bool {
        self == NodeId::FALSE
    }

    pub fn is_true(self) -> bool {
        self == NodeId::TRUE
    }

    fn index(self) -> usize {
        self.to_index()
            .expect("Tried to index graph using a constant (true or false)")
    }

    fn to_index(self) -> Option<usize> {
        if self.is_const() {
            None
        } else {
            // NodeId(2) -> graph[0]
            // NodeId(4) -> graph[1]
            // NodeId(6) -> graph[2]
            Some(((self.regular().0 >> 1) - 1) as usize)
        }
    }

    fn from_index(index: usize) -> Self {
        // graph[0] -> NodeId(2)
        // graph[1] -> NodeId(4)
        // graph[2] -> NodeId(6)
        Self(((index as u32) + 1) << 1)
    }
}

impl AigNode {
    fn new(left: NodeId, right: NodeId) -> Self {
        Self { left, right }
    }

    fn new_input() -> Self {
        Self {
            left: INPUT_NODE_MARKER,
            right: INPUT_NODE_MARKER,
        }
    }

    pub fn left(&self) -> NodeId {
        self.left
    }

    pub fn right(&self) -> NodeId {
        self.right
    }

    pub fn is_input(&self) -> bool {
        self.left.regular() == INPUT_NODE_MARKER && self.right.regular() == INPUT_NODE_MARKER
    }
}

impl AigGraph {
    pub fn new() -> Self {
        Self {
            graph: Vec::new(),
            and_hash: HashMap::new(),
            outputs: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.graph.len()
    }

    pub fn is_empty(&self) -> bool {
        self.graph.is_empty()
    }

    pub fn const_false(&self) -> NodeId {
        NodeId::FALSE
    }

    pub fn const_true(&self) -> NodeId {
        NodeId::TRUE
    }

    pub fn add_input(&mut self) -> NodeId {
        let index = self.graph.len();
        let id = NodeId::from_index(index);

        self.graph.push(AigNode::new_input());

        id
    }

    pub fn add_and_raw(&mut self, left: NodeId, right: NodeId) -> NodeId {
        let index = self.graph.len();
        let id = NodeId::from_index(index);

        self.graph.push(AigNode::new(left, right));

        id
    }

    pub fn add_and_optimized(&mut self, left: NodeId, right: NodeId) -> NodeId {
        let false_id = NodeId::FALSE;
        let true_id = NodeId::TRUE;

        // x & false = false
        if left == false_id || right == false_id {
            return false_id;
        }

        // x & true = x
        if left == true_id {
            return right;
        }

        if right == true_id {
            return left;
        }

        // x & x = x
        if left == right {
            return left;
        }

        // x & !x = false
        if left == right.invert() {
            return false_id;
        }

        // AND is commutative, so canonicalize child order.
        let (left, right) = if right < left {
            (right, left)
        } else {
            (left, right)
        };

        let node = AigNode::new(left, right);

        if let Some(existing_id) = self.and_hash.get(&node) {
            return *existing_id;
        }

        let index = self.graph.len();
        let id = NodeId::from_index(index);

        self.graph.push(node);
        self.and_hash.insert(node, id);

        id
    }

    pub fn add_output(&mut self, output: NodeId) {
        self.outputs.push(output);
    }

    pub fn num_outputs(&self) -> usize {
        self.outputs.len()
    }

    pub fn output(&self, index: usize) -> Option<NodeId> {
        self.outputs.get(index).copied()
    }
}

impl Default for AigGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<NodeId> for AigGraph {
    type Output = AigNode;

    fn index(&self, id: NodeId) -> &Self::Output {
        &self.graph[id.index()]
    }
}
