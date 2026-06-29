from pathlib import Path
import aiger

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent

OUT = REPO_ROOT / "tests" / "aiger" / "inputs"
OUT.mkdir(parents=True, exist_ok=True)


def write(name: str, circ) -> None:

    path = OUT / name

    circ.aig.write(path)
    print(f"wrote {path}")


a, b, c, d = aiger.atoms("a", "b", "c", "d")

write("comb_01_and.aag", (a & b).with_output("out"))

write("comb_02_inverted_input.aag", (a & ~b).with_output("out"))

write("comb_03_inverted_output.aag", (~(a & b)).with_output("out"))

write("comb_04_or.aag", (a | b).with_output("out"))

write("comb_05_xor.aag", (a ^ b).with_output("out"))

write("comb_06_xnor.aag", (a == b).with_output("out"))

write("comb_07_implies.aag", a.implies(b).with_output("out"))

write("comb_08_ite.aag", aiger.ite(a, b, c).with_output("out"))

write("comb_09_and_or_mix.aag", ((a & b) | c).with_output("out"))

write("comb_10_shared_subexpr.aag", ((a & b) | (a & b)).with_output("out"))

write("comb_11_same_input_and.aag", (a & a).with_output("out"))

write("comb_12_signal_and_not_signal.aag", (a & ~a).with_output("out"))

write(
    "comb_13_three_input_majority.aag",
    ((a & b) | (a & c) | (b & c)).with_output("out"),
)

write(
    "comb_14_four_input_tree.aag",
    ((a & b) | (c & d)).with_output("out"),
)

write(
    "comb_15_nested_expression.aag",
    (((a ^ b) & (c | d)) | (~a & c)).with_output("out"),
)


write("const_01_false.aag", aiger.atom(False).with_output("out"))

write("const_02_true.aag", aiger.atom(True).with_output("out"))



multi_01 = (
    (a & b).with_output("and_out").aig
    | (~(a & b)).with_output("nand_out").aig
)

write("multi_01_and_and_nand.aag", multi_01)


multi_02 = (
    (a & b).with_output("and_out").aig
    | (a | b).with_output("or_out").aig
    | (a ^ b).with_output("xor_out").aig
)

write("multi_02_and_or_xor.aag", multi_02)


multi_03 = (
    aiger.atom(False).with_output("false_out").aig
    | aiger.atom(True).with_output("true_out").aig
    | a.with_output("a_out").aig
    | (~a).with_output("not_a_out").aig
)

write("multi_03_constants_and_input.aag", multi_03)

q = aiger.atom("q_in")

seq_01_base = (
    (a & b).with_output("q_next").aig
    | q.with_output("q_out").aig
)

seq_01 = seq_01_base.loopback(
    {
        "input": "q_in",
        "output": "q_next",
        "latch": "q",
        "init": False,
        "keep_output": False,
    }
)

write("seq_01_latch_next_and.aag", seq_01)

q = aiger.atom("q_in")

seq_02_base = (
    (~q).with_output("q_next").aig
    | q.with_output("q_out").aig
)

seq_02 = seq_02_base.loopback(
    {
        "input": "q_in",
        "output": "q_next",
        "latch": "q",
        "init": False,
        "keep_output": False,
    }
)

write("seq_02_latch_self_invert.aag", seq_02)

q = aiger.atom("q_in")

seq_03_base = (
    a.with_output("q_next").aig
    | q.with_output("q_out").aig
)

seq_03 = seq_03_base.loopback(
    {
        "input": "q_in",
        "output": "q_next",
        "latch": "q",
        "init": False,
        "keep_output": False,
    }
)

write("seq_03_latch_next_input.aag", seq_03)

q = aiger.atom("q_in")

seq_04_base = (
    q.with_output("q_next").aig
    | q.with_output("q_out").aig
)

seq_04 = seq_04_base.loopback(
    {
        "input": "q_in",
        "output": "q_next",
        "latch": "q",
        "init": True,
        "keep_output": False,
    }
)

write("seq_04_latch_init_true_hold.aag", seq_04)


q1 = aiger.atom("q1_in")
q2 = aiger.atom("q2_in")

seq_05_base = (
    (a & q2).with_output("q1_next").aig
    | (~q1).with_output("q2_next").aig
    | q1.with_output("q1_out").aig
    | q2.with_output("q2_out").aig
    | (b | q1).with_output("mixed_out").aig
)

seq_05 = seq_05_base.loopback(
    {
        "input": "q1_in",
        "output": "q1_next",
        "latch": "q1",
        "init": False,
        "keep_output": False,
    },
    {
        "input": "q2_in",
        "output": "q2_next",
        "latch": "q2",
        "init": True,
        "keep_output": False,
    },
)

write("seq_05_two_latches_multi_output.aag", seq_05)

q = aiger.atom("q_in")

seq_06_base = (
    (a ^ q).with_output("q_next").aig
    | q.with_output("q_out").aig
)

seq_06 = seq_06_base.loopback(
    {
        "input": "q_in",
        "output": "q_next",
        "latch": "q",
        "init": False,
        "keep_output": True,
    }
)

write("seq_06_latch_keep_next_output.aag", seq_06)


print(f"wrote AIGER tests to {OUT}")