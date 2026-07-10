use super::{AigGraph, HashMap, NodeId};

type Value = usize;
type Env = HashMap<NodeId, Value>;

impl AigGraph {
    pub fn eval(&self, id: NodeId, values: &Env) -> Value {
        if id.is_false() {
            0
        } else if id.is_true() {
            Value::MAX
        } else if id.is_inverted() {
            !self.eval(id.regular(), values)
        } else {
            let node = &self[id];

            if let Some(value) = values.get(&id) {
                *value
            } else {
                assert!(
                    node.is_and(),
                    "The node is not an AND, so it cannot be calculated and a value needs to be provided"
                );

                let left = self.eval(node.left(), values);
                let right = self.eval(node.right(), values);
                left & right
            }
        }
    }

    /// Simulate the whole circuit for several time steps.
    ///
    /// Each Vec<Value> in `input_vectors` is one time step's input assignment.
    /// Values are ordered to match the graph's inputs.
    ///
    /// Returns one Vec<Value> per time step, containing the output values
    /// for that time step.
    ///
    /// example input_vectors:
    /// [
    /// vec![1, 0],   // time 0
    /// vec![0, 0],   // time 1
    /// vec![1, 1],   // time 2
    /// ]
    pub fn simulate(&self, input_vectors: &[Vec<Value>]) -> Vec<Vec<Value>> {
        // format: output_trace[time_step][output_index]
        let mut output_trace = Vec::new();

        // stores the value of every variable right now
        let mut current = Env::new();

        // every latch is initialized as 0 (TODO: when adding
        // supporting for reset values, this will need to change from 0
        // to whatever the reset value is)
        for &latch_id in &self.latches {
            current.insert(latch_id, 0);
        }

        // Each iteration is one time step / one clock cycle.
        for input_values in input_vectors {
            // write this time step's inputs into current
            for (&input_id, &value) in self.inputs.iter().zip(input_values.iter()) {
                current.insert(input_id, value);
            }

            // read output literals before latch update.
            let outputs: Vec<_> = self
                .outputs
                .iter()
                .map(|&output_id| self.eval(output_id, &current))
                .collect();

            output_trace.push(outputs);

            // compute next latch values using current without updating yet
            let mut next = Env::new();

            for &latch_id in &self.latches {
                let latch_node = &self[latch_id];

                let latch_input = latch_node.right();
                let next_value = self.eval(latch_input, &current);

                next.insert(latch_id, next_value);
            }

            // update all latches together (i.e., clock tick)
            current = next;
        }

        output_trace
    }
}
