use std::collections::HashMap;
use std::ops::Index;

mod graphviz;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u32);

const INVERSION_MASK: u32 = 0b0000_0000_0000_0000_0000_0000_0000_0001;
const NODE_ID_MASK: u32 = 0b1111_1111_1111_1111_1111_1111_1111_1110;

/// Reserved marker used inside AigNode to represent "this node is an input/latch marker".
/// This should never be a real graph node ID.
const INPUT_NODE_MARKER: NodeId = NodeId(NODE_ID_MASK);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AigNode {
    left: NodeId,
    right: NodeId,
}

#[derive(Debug)]
pub struct AigGraph {
    nodes: Vec<AigNode>,
    inputs: Vec<NodeId>,
    latches: Vec<NodeId>,
    outputs: Vec<NodeId>,
}

#[derive(Debug)]
pub struct AigBuilder {
    graph: AigGraph,
    and_hash: HashMap<AigNode, NodeId>,
}

impl NodeId {
    pub const FALSE: NodeId = NodeId(0);
    pub const TRUE: NodeId = NodeId(1);

    /// inversion is specified by the LSB of a NodeId being `1`.
    pub fn is_inverted(self) -> bool {
        (self.0 & INVERSION_MASK) != 0
    }

    /// Since the LSB defines inversion, the
    /// rest of the NodeId (i.e., the "regular" part) is its unique identifier
    pub fn regular(self) -> Self {
        Self(self.0 & NODE_ID_MASK)
    }

    /// To invert a Node, simply flip its LSB
    pub fn invert(self) -> Self {
        Self(self.0 ^ INVERSION_MASK)
    }

    /// True or False is represented by a NodeId having a "regular" value of all zeros
    pub fn is_const(self) -> bool {
        self.regular() == NodeId::FALSE
    }

    /// Marker values are reserved for classifying input and latch nodes.
    pub fn is_marker(self) -> bool {
        self.regular() == INPUT_NODE_MARKER
    }

    /// Constant False = regular value of all zeros and a LSB of 0
    pub fn is_false(self) -> bool {
        self == NodeId::FALSE
    }

    /// Constant True = regular value of all zeros and a LSB of 1
    /// (i.e., True is the inversion of False!)
    pub fn is_true(self) -> bool {
        self == NodeId::TRUE
    }

    /// converts a NodeID to an index in our internal represntation of an AIG
    /// see `impl TryFrom<NodeId> for usize` the specific logic of this, and
    /// `impl From<usize> for NodeId` for converting the other way (i.e. graph index to NodeID)
    fn index(self) -> usize {
        usize::try_from(self).expect("NodeId does not correspond to a graph index")
    }
}

/// Conversion from `NodeId` to graph index.
///
/// Our special markers are reserved for identifying inputs and latches, so we do
/// not allow `NodeId`s with those values to be treated as ordinary graph node
/// IDs. Otherwise, we might falsely identify an AND node as being an input or a
/// latch.
///
/// Constants are also not stored in the graph, so `NodeId::FALSE` and
/// `NodeId::TRUE` cannot be converted into graph indices.
///
/// The main idea is that the graph vector is indexed from 0:
///
/// graph[0]
/// graph[1]
/// graph[2]
/// ...
///
///
/// But `NodeId`s do not start at 0. We reserve:
///
///
/// NodeId(0) = false
/// NodeId(1) = true / inverted false
///
///
/// So the first actual graph node has to start after the constants. That gives
/// us:
///
///
/// graph[0] -> NodeId(2)
/// graph[1] -> NodeId(4)
/// graph[2] -> NodeId(6)
///
/// Notice that all real, non-inverted graph nodes are even. This is intentional:
/// the least significant bit is reserved as the inversion bit.
///
/// For example:
///
/// NodeId(6) = regular node
/// NodeId(7) = inverted version of NodeId(6)
///
///
/// In binary, that looks like:
///
///
/// 6 = 0b110
/// 7 = 0b111
///
///
/// So flipping the last bit toggles whether the edge is inverted, while the rest
/// of the bits (the "regular" bits) still identify the same underlying graph node.
///
/// That means when we want the graph index, we first ignore the inversion bit by
/// using the regular node ID. Then we undo the encoding:
///
///
/// graph[0] -> NodeId(2)
/// graph[1] -> NodeId(4)
/// graph[2] -> NodeId(6)
///
///
/// Dividing by 2 gives:
///
///
/// NodeId(2) / 2 = 1
/// NodeId(4) / 2 = 2
/// NodeId(6) / 2 = 3
///
///
/// But graph indices start at 0, not 1, so we subtract 1:
///
/// NodeId(2) / 2 - 1 = 0
/// NodeId(4) / 2 - 1 = 1
/// NodeId(6) / 2 - 1 = 2
///
/// Therefore:
///
/// graph_index = (regular_node_id / 2) - 1
impl TryFrom<NodeId> for usize {
    type Error = &'static str;

    fn try_from(id: NodeId) -> Result<Self, Self::Error> {
        let regular_id = id.regular();

        if regular_id.is_const() {
            return Err("constants are not stored in the graph");
        }

        if regular_id.is_marker() {
            return Err("input/latch marker is not stored in the graph");
        }

        Ok(((regular_id.0 >> 1) - 1) as usize)
    }
}

/// Graph index -> NodeId
///
/// graph[0] -> NodeId(2)
/// graph[1] -> NodeId(4)
/// graph[2] -> NodeId(6)
impl From<usize> for NodeId {
    fn from(index: usize) -> Self {
        // We reserve NODE_ID_MASK / INPUT_NODE_MARKER as a special marker,
        // so the largest real graph NodeId must be smaller than that.
        const MAX_GRAPH_INDEX: usize = (u32::MAX as usize / 2) - 2;

        assert!(
            index <= MAX_GRAPH_INDEX,
            "graph index {index} does not fit in NodeId"
        );

        Self(((index + 1) * 2) as u32)
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

    fn new_latch(latch_input: NodeId) -> Self {
        Self {
            left: INPUT_NODE_MARKER,
            right: latch_input,
        }
    }

    pub fn left(&self) -> NodeId {
        self.left
    }

    pub fn right(&self) -> NodeId {
        self.right
    }

    pub fn is_input(&self) -> bool {
        self.left.is_marker() && self.right.is_marker()
    }

    pub fn is_latch(&self) -> bool {
        self.left.is_marker() && !self.right.is_marker()
    }

    pub fn is_and(&self) -> bool {
        !self.left.is_marker()
    }

    pub fn set_latch_input(&mut self, latch_input: NodeId) {
        assert!(
            self.is_latch(),
            "Tried to set the input of a non-latch node"
        );

        self.right = latch_input;
    }
}

impl AigGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            inputs: Vec::new(),
            latches: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn node(&mut self, id: NodeId) -> &mut AigNode {
        &mut self.nodes[id.index()]
    }
}

impl AigBuilder {
    pub fn new() -> Self {
        Self {
            graph: AigGraph::new(),
            and_hash: HashMap::new(),
        }
    }

    pub fn build(self) -> AigGraph {
        self.graph
    }

    pub fn node(&mut self, id: NodeId) -> &mut AigNode {
        self.graph.node(id)
    }

    pub fn add_input(&mut self) -> NodeId {
        let index = self.graph.nodes.len();
        let id = NodeId::from(index);

        self.graph.nodes.push(AigNode::new_input());
        self.graph.inputs.push(id);

        id
    }

    pub fn add_latch(&mut self, latch_input: NodeId) -> NodeId {
        let index = self.graph.nodes.len();
        let id = NodeId::from(index);

        self.graph.nodes.push(AigNode::new_latch(latch_input));
        self.graph.latches.push(id);

        id
    }

    pub fn add_and_raw(&mut self, left: NodeId, right: NodeId) -> NodeId {
        let index = self.graph.nodes.len();
        let id = NodeId::from(index);

        self.graph.nodes.push(AigNode::new(left, right));

        id
    }

    pub fn add_and_optimized(&mut self, left: NodeId, right: NodeId) -> NodeId {
        // x & false = false
        if left.is_false() || right.is_false() {
            return NodeId::FALSE;
        }

        // x & true = x
        if left.is_true() {
            return right;
        }

        if right.is_true() {
            return left;
        }

        // x & x = x
        if left == right {
            return left;
        }

        // x & !x = false
        if left == right.invert() {
            return NodeId::FALSE;
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

        let index = self.graph.nodes.len();
        let id = NodeId::from(index);

        self.graph.nodes.push(node);
        self.and_hash.insert(node, id);

        id
    }

    pub fn add_output(&mut self, output: NodeId) {
        self.graph.outputs.push(output);
    }
}

impl Default for AigGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<NodeId> for AigGraph {
    type Output = AigNode;

    fn index(&self, id: NodeId) -> &Self::Output {
        &self.nodes[id.index()]
    }
}
