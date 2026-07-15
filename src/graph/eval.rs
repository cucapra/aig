use super::stimulus::Stimulus;
use super::{AigGraph, HashMap, NodeId};

pub type Value = usize;
type Env = HashMap<NodeId, Value>;

/// The complete simulation information for one clock cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationStep {
    /// Latch values at the beginning of this clock cycle.
    pub state: Vec<Value>,

    /// Input values supplied during this clock cycle.
    pub inputs: Vec<Value>,

    /// Output values produced during this clock cycle.
    pub outputs: Vec<Value>,

    /// Latch values after the clock tick.
    pub next_state: Vec<Value>,
}

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
                    "The node is not an AND, so it cannot be calculated \
                     and a value needs to be provided"
                );

                let left = self.eval(node.left(), values);
                let right = self.eval(node.right(), values);

                left & right
            }
        }
    }

    /// Simulate the circuit for several clock cycles.
    ///
    /// The stimulus supplies one input vector per clock cycle. Values in each
    /// vector are ordered to match the graph's inputs.
    ///
    /// Returns one `SimulationStep` per clock cycle. Each step contains the
    /// current latch state, input values, output values, and next latch state.
    pub fn simulate(&self, mut inputs: impl Stimulus) -> Vec<SimulationStep> {
        let mut trace = Vec::new();

        // `current` stores all latch and input values needed during the
        // current clock cycle.
        let mut current = Env::with_capacity(self.latches.len() + self.inputs.len());

        // `next` stores the next value of every latch. It is reused across
        // clock cycles to avoid allocating a new HashMap each time.
        let mut next = Env::with_capacity(self.latches.len());

        // Every latch is initially false.
        //
        // TODO: Use each latch's reset value when reset values are supported.
        for &latch_id in &self.latches {
            current.insert(latch_id, 0);
        }

        // Each iteration is one clock cycle.
        while let Some(input_vector) = inputs.next_vector() {
            let input_values = input_vector.as_ref();

            assert_eq!(
                input_values.len(),
                self.inputs.len(),
                "stimulus supplied {} inputs, but the circuit expects {}",
                input_values.len(),
                self.inputs.len()
            );

            // Record the latch state at the beginning of this cycle.
            let mut state = Vec::with_capacity(self.latches.len());

            for &latch_id in &self.latches {
                state.push(self.eval(latch_id, &current));
            }

            // Write this clock cycle's inputs into the current environment.
            for (&input_id, &value) in self.inputs.iter().zip(input_values.iter()) {
                current.insert(input_id, value);
            }

            // Evaluate outputs before updating the latches.
            let mut outputs = Vec::with_capacity(self.outputs.len());

            for &output_id in &self.outputs {
                outputs.push(self.eval(output_id, &current));
            }

            // Compute all next latch values without updating any latch yet.
            next.clear();

            let mut next_state = Vec::with_capacity(self.latches.len());

            for &latch_id in &self.latches {
                let latch_node = &self[latch_id];
                let latch_input = latch_node.right();
                let next_value = self.eval(latch_input, &current);

                next_state.push(next_value);
                next.insert(latch_id, next_value);
            }

            trace.push(SimulationStep {
                state,
                inputs: input_values.to_vec(),
                outputs,
                next_state,
            });

            // `current` becomes the next latch state. The old `current` map
            // moves into `next` so its allocation can be reused later.
            std::mem::swap(&mut current, &mut next);
        }

        trace
    }
}
