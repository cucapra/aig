use std::collections::HashMap;

use super::{AigGraph, NodeId};

impl AigGraph {
    pub fn eval(&self, id: NodeId, input_values: &HashMap<NodeId, usize>) -> usize {
        match id {
            id if id.is_false() => 0,
            id if id.is_true() => 1,
            id if id.is_inverted() => 1 - (self.eval(id.regular(), input_values)),
            id => {
                let node = &self[id];

                match (node.is_input(), node.is_latch(), node.is_and()) {
                    (_, true, _) => {
                        panic!("cannot evaluate latch {:?} in combinational evaluator", id)
                    }
                    (true, false, false) => {
                        let value = *input_values.get(&id).unwrap();
                        value
                    }

                    (false, false, true) => {
                        let left = self.eval(node.left(), input_values);
                        let right = self.eval(node.right(), input_values);
                        left & right
                    }
                    _ => panic!("invalid AIG node classification for {:?}", id),
                }
            }
        }
    }
}
