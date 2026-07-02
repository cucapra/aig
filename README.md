# AIG in Rust (last updated: 6/22/26)

And-Inverter Graphs (AIGs) implemented in Rust for formal verification and circuit synthesis.

## Overview

An And-Inverter Graph (AIG) is a data structure used to represent Boolean logic circuits. Since any Boolean circuit can be represented using only `AND` and `NOT`, an AIG represents nodes as `AND` gates and edges as either regular or inverted connections


## Internal Representation

The graph stores AIG nodes in a `Vec<AigNode>`:

```rust
pub struct AigGraph {
    nodes: Vec<AigNode>,
}
```

AND nodes store two child `NodeId: u32`s:

```rust
AigNode {
    left: NodeId,
    right: NodeId,
}
```

Each child `NodeId` can refer to a constant, an input, an AND node, or an inverted version of any of those. This means NOT gates are not stored as separate nodes. Instead, inversion is represented directly on the `NodeId`.

```rust
pub struct AigNode {
    left: NodeId,
    right: NodeId,
}
``` 

The least significant bit of a `NodeId` is used as the inversion bit:

```text
even NodeId = regular signal
odd NodeId  = inverted signal
```

So inverting a `NodeId` just toggles the last bit.

```text
a = NodeId(2);
!a = NodeId(3);
b = NodeId(4);
!b = NodeId(5);
```

Constants are represented directly as special reserved `NodeId` values:

```rust
impl NodeId {
    pub const FALSE: NodeId = NodeId(0);
    pub const TRUE: NodeId = NodeId(1);
}
```

This works because `NodeId(1)` is just `NodeId(0)` with the inversion bit set:

```text
NodeId(0) = false
NodeId(1) = !false = true
```

Constants are not stored as nodes in the graph vector. Real graph nodes start at `NodeId(2)` since `NodeId(0)` and `NodeId(1)` are reserved for the constants `true` and `false`:

```text
graph[0] -> NodeId(2)
graph[1] -> NodeId(4)
graph[2] -> NodeId(6)
```

Their inverted versions are represented by setting the least significant bit:

```text
NodeId(2) = graph[0]
NodeId(3) = !graph[0]

NodeId(4) = graph[1]
NodeId(5) = !graph[1]

NodeId(6) = graph[2]
NodeId(7) = !graph[2]
```

Inputs are stored as `NodeId`s and are represented by setting both child fields to a special marker value:

```rust
const INPUT_NODE_MARKER: NodeId = NodeId(NODE_ID_MASK);
```

`NODE_ID_MASK` is all `1`s except for the least significant inversion bit:

```text
NODE_ID_MASK = 11111111111111111111111111111110
```

So the input marker is:

```text
INPUT_NODE_MARKER = NodeId(11111111111111111111111111111110)
```

For example, an input node is stored like this:

```rust
AigNode {
    left: INPUT_NODE_MARKER,
    right: INPUT_NODE_MARKER,
}
```

While multiple inputs contain the same internal marker data, but they are still different inputs because they have different `NodeId`s:

```text
graph[0] = input node -> NodeId(2)
graph[1] = input node -> NodeId(4)
graph[2] = input node -> NodeId(6)
```

Latches are also stored as graph nodes. They are recognized by setting the left child to the same marker and storing the latch input, or next-state signal, in the right child:

```rust
AigNode {
    left: INPUT_NODE_MARKER,
    right: next_state,
}
```

This lets latch state variables have stable `NodeId`s like inputs and AND nodes, while the right side points to the signal that drives the latch on the next step.





## AIGER Input Support

The parser supports both ASCII `.aag` files and binary `.aig` files.

ASCII AIGER files begin with a header of the form:

```text
aag M I L O A
```

Binary AIGER files use the same counts with an `aig` header:

```text
aig M I L O A
```

where:

```text
M = maximum variable index
I = number of inputs
L = number of latches
O = number of outputs
A = number of AND gates
```

The header must satisfy:

```text
M >= I + L + A
```

Latch lines are supported in the ASCII parser:

```text
<latch literal> <next-state literal> [reset]
```

The optional reset field is currently accepted only when it is `0`, because reset values are not represented in `AigNode`.

In binary AIGER, input literals and latch current-state literals are implicit. The parser reads one next-state literal per latch, one output literal per output, then decodes each AND gate from its binary delta encoding. Binary files must satisfy:

```text
M = I + L + A
```

The parser is split by responsibility:

```text
src/aiger_parser.rs         = header parsing, format validation, dispatch
src/aiger_ascii_parser.rs   = ASCII body parsing
src/aiger_binary_parser.rs  = binary body parsing
```

## Parsing an AIGER File

To parse an AIGER file, use:

```rust
run_parser_with_options(file_name: &str, pre_optimize: bool) -> io::Result<()>
```

Example:

```rust
run_parser_with_options("example.aag", true)?;
run_parser_with_options("example.aig", true)?;
```

The `pre_optimize` option controls whether the parser performs simple on-the-fly optimizations while building the graph.

If `pre_optimize` is `true`, the parser simplifies expressions before inserting new AND nodes. For example:

```text
x & false = false
x & true  = x
x & x     = x
x & !x    = false
...
```

If `pre_optimize` is `false`, the parser builds the graph directly from the AIGER file without applying these simplifications.
