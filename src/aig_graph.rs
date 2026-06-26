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
    inputs: Vec<NodeId>,
    latches: Vec<NodeId>,
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

    // Given a NodeId, find the Vec index of the underlying real node.
    fn to_index(self) -> Option<usize> {
        if self.is_const() {
            None // constants aren't stored in the graph
        } else {
            // NodeId(2) -> graph[0]
            // NodeId(4) -> graph[1]
            // NodeId(6) -> graph[2]
            Some(((self.regular().0 >> 1) - 1) as usize)
        }
    }

    // Given a Vec index, produce the regular non-inverted NodeId.
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

    fn new_latch(latch_id: NodeId) -> Self {
        Self {
            left: INPUT_NODE_MARKER,
            right: latch_id,
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

    pub fn is_latch(&self) -> bool {
        self.left.regular() == INPUT_NODE_MARKER && self.right.regular() != INPUT_NODE_MARKER
    }

    pub fn is_and(&self) -> bool {
        self.left.regular() != INPUT_NODE_MARKER
    }
}

impl AigGraph {
    pub fn new() -> Self {
        Self {
            graph: Vec::new(),
            and_hash: HashMap::new(),
            inputs: Vec::new(),
            latches: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn to_dot(&self) -> String {
        let mut dot: String = String::new();

        dot.push_str(
            "digraph AIG {\n\
        \trankdir=BT;\n\
        \tordering=out;\n\
        \tnode [fontname=\"Helvetica\"];\n\
        \tedge [fontname=\"Helvetica\"];\n\n\
        \tconst_false [label=\"false\", shape=box];\n\
        \tconst_true [label=\"true\", shape=box];\n\n",
        );

        for (input_index, input_id) in self.inputs.iter().enumerate() {
            let node_name = Self::dot_name(input_id.regular());
            let input_label = Self::input_label(input_index);

            dot.push_str(&format!(
                "\t{} [label=\"{}\", shape=box];\n",
                node_name, input_label
            ));
        }

        dot.push_str("\n");

        for (latch_index, latch_id) in self.latches.iter().enumerate() {
            let node_name = Self::dot_name(latch_id.regular());

            dot.push_str(&format!(
                "\t{} [label=\"l{}\", shape=box, style=rounded];\n",
                node_name, latch_index
            ));
        }

        dot.push_str("\n");

        for (index, node) in self.graph.iter().enumerate() {
            if node.is_and() {
                let node_id = NodeId::from_index(index);
                let node_name = Self::dot_name(node_id);

                dot.push_str(&format!("\t{} [label=\"AND\", shape=circle];\n", node_name));
            }
        }

        dot.push_str("\n");

        for (index, node) in self.graph.iter().enumerate() {
            if node.is_and() {
                let parent_id = NodeId::from_index(index);
                let parent_name = Self::dot_name(parent_id);

                Self::write_dot_edge(&mut dot, node.left(), &parent_name, "left");
                Self::write_dot_edge(&mut dot, node.right(), &parent_name, "right");
            }
        }

        dot.push_str("\n");

        for latch_id in &self.latches {
            let latch = &self[*latch_id];
            let latch_name = Self::dot_name(latch_id.regular());

            Self::write_dot_edge(&mut dot, latch.right(), &latch_name, "next");
        }

        dot.push_str("\n");

        for (index, output) in self.outputs.iter().enumerate() {
            let output_name = format!("out{}", index);

            dot.push_str(&format!(
                "\t{} [label=\"out{}\", shape=box];\n",
                output_name, index
            ));

            Self::write_dot_edge(&mut dot, *output, &output_name, "");
        }

        dot.push_str("}\n");

        dot
    }

    fn write_dot_edge(dot: &mut String, child: NodeId, parent_name: &str, edge_label: &str) {
        let child_name = Self::dot_name(child.regular());

        if edge_label.is_empty() {
            if child.is_inverted() {
                dot.push_str(&format!(
                    "\t{} -> {} [style=dashed];\n",
                    child_name, parent_name
                ));
            } else {
                dot.push_str(&format!("\t{} -> {};\n", child_name, parent_name));
            }
        } else if child.is_inverted() {
            dot.push_str(&format!(
                "\t{} -> {} [label=\"{}\", style=dashed];\n",
                child_name, parent_name, edge_label
            ));
        } else {
            dot.push_str(&format!(
                "\t{} -> {} [label=\"{}\"];\n",
                child_name, parent_name, edge_label
            ));
        }
    }

    fn dot_name(id: NodeId) -> String {
        if id.is_false() {
            String::from("const_false")
        } else if id.is_true() {
            String::from("const_true")
        } else {
            format!("n{}", id.index())
        }
    }

    // to represent inputs as letters instead of numbers
    fn input_label(index: usize) -> String {
        if index < 26 {
            ((b'a' + index as u8) as char).to_string()
        } else {
            let prefix = Self::input_label((index / 26) - 1);
            let suffix = (b'a' + (index % 26) as u8) as char;

            format!("{}{}", prefix, suffix)
        }
    }

    pub fn add_input(&mut self) -> NodeId {
        let index: usize = self.graph.len();
        let id: NodeId = NodeId::from_index(index);

        self.graph.push(AigNode::new_input());
        self.inputs.push(id);

        id
    }

    pub fn add_latch(&mut self, latch_input: NodeId) -> NodeId {
        let index = self.graph.len();
        let id = NodeId::from_index(index);

        self.graph.push(AigNode::new_latch(latch_input));
        self.latches.push(id);

        id
    }

    pub fn set_latch_input(&mut self, latch_id: NodeId, latch_input: NodeId) {
        let latch_index = latch_id.index();
        let latch = &mut self.graph[latch_index];

        assert!(
            latch.is_latch(),
            "Tried to set the input of a non-latch node"
        );

        *latch = AigNode::new_latch(latch_input);
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
