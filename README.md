# raig

`raig` (pronounced “rage”) is a dependency-free library for working with
<a href="https://en.wikipedia.org/wiki/And-inverter_graph" target="_blank" rel="noopener noreferrer">And-Inverter Graphs (AIGs)</a>
in Rust, developed at
<a href="https://capra.cs.cornell.edu/" target="_blank" rel="noopener noreferrer">Cornell's Capra Lab</a>.

An AIG represents Boolean logic with AND gates and inverted edges. This compact
form is useful for logic verification, synthesis, and testing tools.

The library has no default dependencies. The optional `cli` feature enables the
`raig` command-line tool and its `clap` dependency.

## Installation

Add the library to a Rust project:

```sh
cargo add raig
```

Install the command-line tool:

```sh
cargo install raig --features cli
```

## Tutorial

### 1.0 Start With an AIGER File

The most common starting point is an AIGER file from another tool. `raig`
accepts anything that implements `std::io::BufRead`, so the same parser works
for local files, stdin, network responses, and uploaded file bytes.

This tiny ASCII AIGER circuit has one input and one output. The output is
exactly the input, so it behaves like an identity function.

```text
aag 1 1 0 1 0
2
2
```

The header is `aag M I L O A`. In this example, `M = 1`, `I = 1`, `L = 0`,
`O = 1`, and `A = 0`: one input, no latches, one output, and no AND gates.

#### 1.1 Parse Uploaded Bytes

```rust
use raig::aiger::run_parser_with_options;
use std::io::BufReader;

let uploaded_aiger = b"aag 1 1 0 1 0\n2\n2\n";
let mut reader = BufReader::new(&uploaded_aiger[..]);
let graph = run_parser_with_options(&mut reader, true)?;
# Ok::<(), std::io::Error>(())
```

The second argument is `pre_optimize`. Pass `true` to simplify common identities
while parsing, such as `x & true`, `x & false`, `x & x`, and `x & !x`.

#### 1.2 Parse a File From Disk

```rust
use raig::aiger::run_parser_with_options;
use std::fs::File;
use std::io::BufReader;

let file = File::open("circuit.aag")?;
let mut reader = BufReader::new(file);
let graph = run_parser_with_options(&mut reader, true)?;
# Ok::<(), std::io::Error>(())
```

### 2.0 Work With the Parsed Graph

After parsing, you have an internal AIG representation. From there, the most
common next steps are simulation and visualization.

#### 2.1 Simulate the Circuit

Simulation inputs are vectors of `raig::graph::Value`. Use `0` for false and
`Value::MAX` for true.

```rust
use raig::aiger::run_parser_with_options;
use raig::graph::Value;
use std::io::BufReader;

let uploaded_aiger = b"aag 1 1 0 1 0\n2\n2\n";
let mut reader = BufReader::new(&uploaded_aiger[..]);
let graph = run_parser_with_options(&mut reader, true)?;

let stimulus = vec![
    vec![0],
    vec![Value::MAX],
];
let trace = graph.simulate(stimulus.as_slice());

assert_eq!(trace[0].outputs[0], 0);
assert_eq!(trace[1].outputs[0], Value::MAX);
# Ok::<(), std::io::Error>(())
```

`Value` is a packed Boolean value. Other bit patterns can be used to evaluate
many independent Boolean lanes in parallel with bitwise operations.

#### 2.2 Render Graphviz DOT

```rust
use raig::aiger::run_parser_with_options;
use std::io::BufReader;

let uploaded_aiger = b"aag 1 1 0 1 0\n2\n2\n";
let mut reader = BufReader::new(&uploaded_aiger[..]);
let graph = run_parser_with_options(&mut reader, true)?;

let dot = graph.to_dot();
assert!(dot.contains("digraph AIG"));
# Ok::<(), std::io::Error>(())
```

Render DOT with Graphviz:

```sh
dot -Tsvg graph.dot -o graph.svg
```

### 3.0 Use the CLI

Parse an AIGER file:

```sh
raig parse circuit.aag
```

Generate DOT:

```sh
raig dot circuit.aag --output circuit.dot
```

Simulate with a stimulus file:

```sh
raig simulate circuit.aag stimulus.txt
```

Stimulus files contain one input vector per line and may end with `.`:

```text
0
1
.
```

### 4.0 Build Graphs Directly

You can also build AIGs directly in Rust with `AigBuilder`. This is useful for
generating circuits, writing tests, or integrating with another frontend.

#### 4.1 Build an AND Gate

```rust
use raig::graph::{AigBuilder, Value};

let mut builder = AigBuilder::new();
let a = builder.add_input();
let b = builder.add_input();
let output = builder.add_and_optimized(a, b);
builder.add_output(output);

let graph = builder.build();
let inputs = vec![
    vec![Value::MAX, Value::MAX],
    vec![Value::MAX, 0],
];
let trace = graph.simulate(inputs.as_slice());

assert_eq!(trace[0].outputs[0], Value::MAX);
assert_eq!(trace[1].outputs[0], 0);
```

## AIGER Support

`raig` supports the core ASCII and binary AIGER formats:

- headers with `aag M I L O A` or `aig M I L O A`
- inputs
- latches
- outputs
- AND gates
- AIGER 1.9 extension counts in the header

ASCII latch reset fields are accepted when the reset value is `0`. Other reset
values are not represented yet.

## License

Licensed under the MIT license.
