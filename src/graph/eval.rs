use super::{AigGraph, NodeId, HashMap};

type Value = usize;
type Env = HashMap<NodeId, Value>;

impl AigGraph {
    pub fn eval(&self, id: NodeId, input_values: &Env) -> Value {
        if id.is_false() {
            0
        } else if id.is_true() {
            1
        } else if id.is_inverted() {
            1 - self.eval(id.regular(), input_values)
        } else {
            let node = &self[id];

            if node.is_latch() {
                panic!("cannot evaluate latch {:?} in combinational evaluator", id)
            } else if node.is_input() {
                *input_values.get(&id).unwrap()
            } else if node.is_and() {
                let left = self.eval(node.left(), input_values);
                let right = self.eval(node.right(), input_values);
                left & right
            } else {
                panic!("invalid AIG node classification for {:?}", id)
            }
        }
    }
}
