from pathlib import Path
from random import Random

SCRIPT_DIR = Path(__file__).resolve().parent
INPUT_DIR = SCRIPT_DIR.parent / "tests" / "inputs"


def write_stimulus(aag_path: Path) -> None:
    # get I and L from 'aag M I L O A'
    aag_header = aag_path.read_text().split()
    I = int(aag_header[2])
    L = int(aag_header[3])

    # just for fun: use path name as random seed!
    # (not necessary AT ALL but I think it's cool)
    rng = Random(aag_path.name)
    clock_cycles = 1 if L == 0 else 2**L + 1
    input_rows = []
    for _ in range(clock_cycles):
        input_rows.append("".join(rng.choice("01") for _ in range(I)))

    stim_path = aag_path.with_suffix(".stim")

    with open(stim_path, 'w') as f:
        for row in input_rows:
            print(row, file=f)
        print('.', file =f)

    print(f"wrote {stim_path}")


def main() -> None:
    for aag_path in sorted(INPUT_DIR.glob("*.aag")):
        write_stimulus(aag_path)

    print(f"wrote AIGER stimulus inputs to {INPUT_DIR}")


if __name__ == "__main__":
    main()
