# raig-cli

Command-line tools for working with And-Inverter Graphs (AIGs).

Install with:

```sh
cargo install raig-cli
```

This installs the `raig` executable.

## Usage

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
