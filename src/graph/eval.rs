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
    /// TODO: maybe expand upon this doc some more (i.e. explain Stimulus type a bit)
    pub fn simulate(&self, mut inputs: impl Stimulus) -> Vec<SimulationStep> {
        let mut trace = Vec::new();
        let mut current = Env::with_capacity(self.latches.len() + self.inputs.len());

        // `next` stores the next value of every latch. It is reused across
        // clock cycles to avoid allocating a new HashMap each time. Yippy!
        let mut next = Env::with_capacity(self.latches.len());

        // TODO: Use each latch's reset value when reset values are supported.
        for &latch_id in &self.latches {
            current.insert(latch_id, 0);
        }

        // Each iteration is one time step / one clock cycle
        while let Some(input_vector) = inputs.next_vector() {
            let input_values = input_vector.as_ref();

            // Record the latch state at the beginning of this cycle
            let state: Vec<_> = self
                        .latches
                        .iter()
                        .map(|&latch_id| self.eval(latch_id, &current))
                        .collect();

            // Write this clock cycle's inputs into the current environment
            for (&input_id, &value) in self.inputs.iter().zip(input_values.iter()) {
                current.insert(input_id, value);
            }

            // Evaluate outputs before updating the latches
            let outputs: Vec<_> = self
                .outputs
                .iter()
                .map(|&output_id| self.eval(output_id, &current))
                .collect();

            // Compute all next latch values without updating any latch yet
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Commands::Parse;
    use crate::graph::AigBuilder;

    fn and(g: &mut AigBuilder, a: NodeId, b: NodeId) -> NodeId {
        g.add_and_optimized(a, b)
    }

    fn or(g: &mut AigBuilder, a: NodeId, b: NodeId) -> NodeId {
        // a || b = not(not(a || b)) = not(not(a) && not(b))
        and(g, a.invert(), b.invert()).invert()
    }

    fn xor(g: &mut AigBuilder, a: NodeId, b: NodeId) -> NodeId {
        // a xor b = (a || b) && !(a && b)
        let a_or_b = or(g, a, b);
        let a_and_b = and(g, a, b);
        and(g, a_or_b, a_and_b.invert())
    }

    fn xor3(g: &mut AigBuilder, a: NodeId, b: NodeId, c: NodeId) -> NodeId {
        let a_xor_b = xor(g, a, b);
        xor(g, a_xor_b, c)
    }

    fn majority(g: &mut AigBuilder, a: NodeId, b: NodeId, c: NodeId) -> NodeId {
        // majority(a,b,c) = ((a xor b) and c) or (a and b)
        //                 = or(and(a, b), or(and(a,c), and(b,c)))
        let a_xor_b = xor(g, a, b);
        let a_and_b = and(g, a, b);
        let a_xor_b_and_c = and(g, a_xor_b, c);
        or(g, a_xor_b_and_c, a_and_b)
    }

    fn half_adder(g: &mut AigBuilder, a: NodeId, b: NodeId) -> (NodeId, NodeId) {
        let sum = xor(g, a, b);
        let carry = and(g, a, b);
        (carry, sum)
    }

    fn full_adder(g: &mut AigBuilder, a: NodeId, b: NodeId, c: NodeId) -> (NodeId, NodeId) {
        let sum = xor3(g, a, b, c);
        let carry = majority(g, a, b, c);
        (carry, sum)
    }

    fn add(g: &mut AigBuilder, a: &[NodeId], b: &[NodeId]) -> Vec<NodeId> {
        assert_eq!(a.len(), b.len());
        assert!(a.len() >= 1);
        let (c0, s0) = half_adder(g, a[0], b[0]);
        let mut sum = vec![s0];
        let mut carry = c0;
        for (&a, &b) in a.iter().skip(1).zip(b.iter().skip(1)) {
            let (c, s) = full_adder(g, a, b, carry);
            sum.push(s);
            carry = c;
        }
        sum
    }

    fn zero(bits: u32) -> Vec<NodeId> {
        vec![NodeId::FALSE; bits as usize]
    }

    fn one(bits: u32) -> Vec<NodeId> {
        assert!(bits > 0);
        let mut out = zero(bits);
        out[0] = NodeId::TRUE;
        out
    }

    fn set_latch_inputs(g: &mut AigBuilder, latches: &[NodeId], next: &[NodeId]) {
        assert_eq!(latches.len(), next.len());
        for (&l, &n) in latches.iter().zip(next.iter()) {
            g.node(l).set_latch_input(n);
        }
    }

    fn make_counter(g: &mut AigBuilder, bits: u32) {
        let counter: Vec<_> = (0..bits).map(|_| g.add_latch(NodeId::FALSE)).collect();
        let one = one(bits);
        let counter_next = add(g, &counter, &one);
        set_latch_inputs(g, &counter, &counter_next);
        for &c in &counter {
            g.add_output(c);
        }
    }

    fn read_bit_vector(values: &[Value], offset: usize, width: u32) -> u64 {
        assert!(width <= 64);
        let width = width as usize;
        assert!(values.len() >= width + offset);
        values
            .iter()
            .skip(offset)
            .enumerate()
            .map(|(idx, &value)| ((value as u64) & 1) << idx)
            .reduce(|a, b| a | b)
            .unwrap_or(0)
    }

    #[test]
    fn test_sim_counter() {
        let bits = 8;
        let cycles = (1 << bits) + 1;

        let counter_mask = (1 << bits) - 1;
        let mut g = AigBuilder::new();
        make_counter(&mut g, bits);
        let g = g.build();
        let inputs = vec![vec![]; cycles];
        let result = g.simulate(inputs.as_slice());
        assert_eq!(result.len(), cycles);
        for (step, values) in result.iter().enumerate() {
            let count = read_bit_vector(values, 0, bits);
            assert_eq!(count, (step & counter_mask) as u64);
            // println!("step: {}, count: {}", step, count);
        }
    }
}
