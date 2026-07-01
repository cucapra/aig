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
