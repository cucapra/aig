pub type NodeId = u32;

pub const INVERSION_MASK: NodeId = 0b0000_0000_0000_0000_0000_0000_0000_0001;
pub const NODE_ID_MASK: NodeId = 0b1111_1111_1111_1111_1111_1111_1111_1110;

pub const INPUT_NODE_MARKER: NodeId = NODE_ID_MASK;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AigNode {
    left: NodeId,
    right: NodeId,
}

#[derive(Debug, Default)]
pub struct AigGraph {
    graph: Vec<AigNode>,
}

pub fn is_inverted(id: NodeId) -> bool {
    (id & INVERSION_MASK) != 0
}

pub fn regular(id: NodeId) -> NodeId {
    id & NODE_ID_MASK
}

pub fn invert(id: NodeId) -> NodeId {
    id ^ INVERSION_MASK
}

pub fn node_index(id: NodeId) -> usize {
    (regular(id) >> 1) as usize
}

pub fn make_node_id(index: usize) -> NodeId {
    (index as NodeId) << 1
}

impl AigNode {
    pub fn new(left: NodeId, right: NodeId) -> Self {
        Self { left, right }
    }

    pub fn new_input() -> Self {
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

    pub fn set_left(&mut self, left: NodeId) {
        self.left = left;
    }

    pub fn set_right(&mut self, right: NodeId) {
        self.right = right;
    }

    pub fn is_input(&self) -> bool {
        regular(self.left) == INPUT_NODE_MARKER && regular(self.right) == INPUT_NODE_MARKER
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
        let index: usize = self.graph.len();
        let id: NodeId = make_node_id(index);

        self.graph.push(AigNode::new_input());

        id
    }

    pub fn add_and(&mut self, left: NodeId, right: NodeId) -> NodeId {
        let index: usize = self.graph.len();
        let id: NodeId = make_node_id(index);

        self.graph.push(AigNode::new(left, right));

        id
    }

    pub fn get_node(&self, id: NodeId) -> Option<&AigNode> {
        self.graph.get(node_index(id))
    }

    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut AigNode> {
        self.graph.get_mut(node_index(id))
    }

    pub fn set_node(&mut self, id: NodeId, left: NodeId, right: NodeId) -> Result<(), String> {
        let node = self
            .graph
            .get_mut(node_index(id))
            .ok_or_else(|| format!("No node exists for NodeId {}", id))?;

        node.set_left(left);
        node.set_right(right);

        Ok(())
    }

    pub fn is_input(&self, id: NodeId) -> bool {
        self.get_node(id).is_some_and(AigNode::is_input)
    }

    pub fn is_and(&self, id: NodeId) -> bool {
        self.get_node(id).is_some_and(|node| !node.is_input())
    }

    pub fn left_child(&self, id: NodeId) -> Option<NodeId> {
        self.get_node(id).map(AigNode::left)
    }

    pub fn right_child(&self, id: NodeId) -> Option<NodeId> {
        self.get_node(id).map(AigNode::right)
    }
}