use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

const INVERSION_MASK: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0001;
const NODE_ID_MASK: u32 = 0b1111_1111_1111_1111_1111_1111_1111_1110;

const INPUT_NODE_MARKER: NodeId = NodeId(NODE_ID_MASK);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AigNode {
    left: NodeId,
    right: NodeId,
}

#[derive(Debug)]
pub struct AigGraph {
    graph: Vec<AigNode>,
}

impl NodeId {
    pub fn is_inverted(self) -> bool {
        (self.0 & INVERSION_MASK) != 0
    }

    pub fn regular(self) -> Self {
        Self(self.0 & NODE_ID_MASK)
    }

    pub fn invert(self) -> Self {
        Self(self.0 ^ INVERSION_MASK)
    }

    fn index(self) -> usize {
        (self.regular().0 >> 1) as usize
    }

    fn from_index(index: usize) -> Self {
        Self((index as u32) << 1)
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

    pub fn is_and(&self) -> bool {
        !self.is_input()
    }
}

impl AigGraph {
    pub fn new() -> Self {
        Self { graph: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.graph.len()
    }

    pub fn is_empty(&self) -> bool {
        self.graph.is_empty()
    }

    pub fn add_input(&mut self) -> NodeId {
        let index = self.graph.len();
        let id = NodeId::from_index(index);

        self.graph.push(AigNode::new_input());

        id
    }

    pub fn add_and(&mut self, left: NodeId, right: NodeId) -> NodeId {
        let index = self.graph.len();
        let id = NodeId::from_index(index);

        self.graph.push(AigNode::new(left, right));

        id
    }
}

impl Index<NodeId> for AigGraph {
    type Output = AigNode;

    fn index(&self, id: NodeId) -> &Self::Output {
        &self.graph[id.index()]
    }
}

impl IndexMut<NodeId> for AigGraph {
    fn index_mut(&mut self, id: NodeId) -> &mut Self::Output {
        &mut self.graph[id.index()]
    }
}
