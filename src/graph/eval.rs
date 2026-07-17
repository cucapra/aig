use super::stimulus::Stimulus;
use super::{AigGraph, HashMap, NodeId};

pub type Value = usize;
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
    pub fn simulate(&self, mut inputs: impl Stimulus) -> Vec<Vec<Value>> {
        let mut output_trace = Vec::new();

        // stores the value of every variable right now
        let mut current = Env::new();

        // every latch is initialized as 0
        // TODO: when we want to support resets, this will be the reset
        // value instead of 0
        for &latch_id in &self.latches {
            current.insert(latch_id, 0);
        }

        // Each iteration is one time step / one clock cycle.
        while let Some(input_vector) = inputs.next_vector() {
            let input_values = input_vector.as_ref();

            // Write this time step's inputs into current.
            for (&input_id, &value) in self.inputs.iter().zip(input_values.iter()) {
                current.insert(input_id, value);
            }

            // Read output literals before latch update.
            let outputs: Vec<_> = self
                .outputs
                .iter()
                .map(|&output_id| self.eval(output_id, &current))
                .collect();

            output_trace.push(outputs);

            // Compute next latch values without updating the current latches yet.
            let mut next = Env::new();

            for &latch_id in &self.latches {
                let latch_node = &self[latch_id];
                let latch_input = latch_node.right();
                let next_value = self.eval(latch_input, &current);

                next.insert(latch_id, next_value);
            }

            // Update all latches together.
            current = next;
        }

        output_trace
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{AigBuilder, StimulusParser};

    #[test]
    fn simulate_accepts_stimulus_parser() {
        let mut builder = AigBuilder::new();
        let input = builder.add_input();
        builder.add_output(input);
        let graph = builder.build();

        let stimulus = b"0\n1\n.\n";
        let result = graph.simulate(StimulusParser::new(&stimulus[..]));

        assert_eq!(result, vec![vec![0], vec![Value::MAX]]);
    }
}
