from pathlib import Path

import aiger

SCRIPT_DIR = Path(__file__).resolve().parent
INPUT_DIR = SCRIPT_DIR.parent / "tests" / "inputs"
INPUT_DIR.mkdir(parents=True, exist_ok=True)


def write_case(filename: str, circuit) -> None:
    aag_path = INPUT_DIR / filename

    circuit.write(aag_path)
    print(f"wrote {aag_path}")


def latch(input_name: str, output_name: str, latch_name: str) -> dict:
    return {
        "input": input_name,
        "output": output_name,
        "latch": latch_name,
        "init": False,
        "keep_output": False,
    }


a, b, c, d = aiger.atoms("a", "b", "c", "d")
q = aiger.atom("q_in")
q1 = aiger.atom("q1_in")
q2 = aiger.atom("q2_in")

majority_three = (a & b) | (a & c) | (b & c)
at_least_three_of_four = (
    (a & b & c)
    | (a & b & d)
    | (a & c & d)
    | (b & c & d)
)
all_equal_four = (a == b) & (b == c) & (c == d)

exactly_one_of_four = (
    (a & ~b & ~c & ~d)
    | (~a & b & ~c & ~d)
    | (~a & ~b & c & ~d)
    | (~a & ~b & ~c & d)
)

at_most_one_of_four = ~(
    (a & b)
    | (a & c)
    | (a & d)
    | (b & c)
    | (b & d)
    | (c & d)
)

exactly_two_of_four = (
    (a & b & ~c & ~d)
    | (a & ~b & c & ~d)
    | (a & ~b & ~c & d)
    | (~a & b & c & ~d)
    | (~a & b & ~c & d)
    | (~a & ~b & c & d)
)

next_and_latch = (
    (a & b).with_output("q_next").aig
    | q.with_output("out").aig
).loopback(latch("q_in", "q_next", "q"))

self_inverting_latch = (
    (~q).with_output("q_next").aig
    | q.with_output("out").aig
).loopback(latch("q_in", "q_next", "q"))

input_delay_latch = (
    a.with_output("q_next").aig
    | q.with_output("out").aig
).loopback(latch("q_in", "q_next", "q"))

two_latch_feedback = (
    (a & q2).with_output("q1_next").aig
    | (q1 ^ b).with_output("q2_next").aig
    | (q1 ^ q2).with_output("out").aig
).loopback(
    latch("q1_in", "q1_next", "q1"),
    latch("q2_in", "q2_next", "q2"),
)

TEST_CASES = [
    # Constants and simple inputs.
    ("comb_constant_false.aag", aiger.atom(False).with_output("out").aig),
    ("comb_constant_true.aag", aiger.atom(True).with_output("out").aig),
    ("comb_direct_input.aag", a.with_output("out").aig),
    ("comb_inverted_input.aag", (~a).with_output("out").aig),
    # Basic Boolean operations.
    ("comb_and.aag", (a & b).with_output("out").aig),
    ("comb_and_inverted_left.aag", (~a & b).with_output("out").aig),
    ("comb_and_inverted_right.aag", (a & ~b).with_output("out").aig),
    ("comb_and_both_inputs_inverted.aag", (~a & ~b).with_output("out").aig),
    ("comb_nand.aag", (~(a & b)).with_output("out").aig),
    ("comb_or.aag", (a | b).with_output("out").aig),
    ("comb_nor.aag", (~(a | b)).with_output("out").aig),
    ("comb_xor.aag", (a ^ b).with_output("out").aig),
    ("comb_xnor.aag", (a == b).with_output("out").aig),
    ("comb_implies.aag", a.implies(b).with_output("out").aig),
    ("comb_ite.aag", aiger.ite(a, b, c).with_output("out").aig),
    # Cases that optimize to constants.
    ("true_or_complement.aag", (a | ~a).with_output("out").aig),
    ("true_or_true.aag", (a | True).with_output("out").aig),
    ("true_xor_complement.aag", (a ^ ~a).with_output("out").aig),
    ("true_xnor_same_signal.aag", (a == a).with_output("out").aig),
    ("true_implies_self.aag", a.implies(a).with_output("out").aig),
    ("false_and_complement.aag", (a & ~a).with_output("out").aig),
    ("false_and_false.aag", (a & False).with_output("out").aig),
    ("false_xor_same_signal.aag", (a ^ a).with_output("out").aig),
    # Cases that optimize to one input.
    ("identity_and_same_signal.aag", (a & a).with_output("out").aig),
    ("identity_and_true.aag", (a & True).with_output("out").aig),
    ("identity_or_same_signal.aag", (a | a).with_output("out").aig),
    ("identity_or_false.aag", (a | False).with_output("out").aig),
    ("identity_double_inversion.aag", (~~a).with_output("out").aig),
    ("identity_ite_equal_branches.aag", aiger.ite(a, b, b).with_output("out").aig),
    # Larger Boolean functions.
    ("function_parity_four.aag", (a ^ b ^ c ^ d).with_output("out").aig),
    ("function_majority_three.aag", majority_three.with_output("out").aig),
    ("function_at_least_three_of_four.aag", at_least_three_of_four.with_output("out").aig),
    ("function_all_equal_four.aag", all_equal_four.with_output("out").aig),
    ("function_exactly_one_of_four.aag", exactly_one_of_four.with_output("out").aig),
    ("function_at_most_one_of_four.aag", at_most_one_of_four.with_output("out").aig),
    ("function_exactly_two_of_four.aag", exactly_two_of_four.with_output("out").aig),
    # Sequential circuits.
    ("seq_next_and_latch.aag", next_and_latch),
    ("seq_self_inverting_latch.aag", self_inverting_latch),
    ("seq_input_delay_latch.aag", input_delay_latch),
    ("seq_two_latch_feedback.aag", two_latch_feedback),
]


def main() -> None:
    for filename, circuit in TEST_CASES:
        write_case(filename, circuit)

    print(f"wrote py-aiger AAG tests to {INPUT_DIR}")


if __name__ == "__main__":
    main()
