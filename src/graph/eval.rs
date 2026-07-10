use super::{AigGraph, HashMap, NodeId};

type Value = usize;
type Env = HashMap<NodeId, Value>;

impl AigGraph {
    pub fn eval(&self, id: NodeId, values: &Env) -> Value {
        if id.is_false() {
            0
        } else if id.is_true() {
            1
        } else if id.is_inverted() {
            1 - self.eval(id.regular(), values)
        } else {
            let node = &self[id];

            if node.is_input() || node.is_latch() {
                *values.get(&id).unwrap()
            } else if node.is_and() {
                let left = self.eval(node.left(), values);
                let right = self.eval(node.right(), values);
                left & right
            } else {
                panic!("invalid AIG node classification for {:?}", id)
            }
        }
    }

    /// Simulate the whole circuit for several time steps.
    ///
    /// Each Env in `input_vectors` is one time step's input assignment.
    ///
    /// Returns one Vec<Value> per time step, containing the output values
    /// for that time step.
    ///
    /// example input_vectors:
    /// [
    /// { a: 1, b: 0 },   // time 0
    /// { a: 0, b: 0 },   // time 1
    /// { a: 1, b: 1 },   // time 2
    /// ]
    pub fn simulate(&self, input_vectors: &[Env]) -> Vec<Vec<Value>> {
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
            for &input_id in &self.inputs {
                let value = *input_values.get(&input_id).unwrap();
                // maybe todo: have a helper function to generate an input_vectors
                // that includes this invariant check, also maybe includes a simple way
                // to build an input_vector with inputs that do not change over time
                //i.e., every entry in input_vectors is the same
                assert!(value == 0 || value == 1);

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
            // could this be done with threads???
            for &latch_id in &self.latches {
                let next_value = *next.get(&latch_id).unwrap();
                current.insert(latch_id, next_value);
            }
        }

        output_trace
    }
}

