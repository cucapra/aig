# raig

`raig` (pronounced “rage”) is a dependency-free library for working with
<a href="https://en.wikipedia.org/wiki/And-inverter_graph" target="_blank" rel="noopener noreferrer">And-Inverter Graphs (AIGs)</a>
in Rust, developed at
<a href="https://capra.cs.cornell.edu/" target="_blank" rel="noopener noreferrer">Cornell's Capra Lab</a>.

An AIG represents Boolean logic with AND gates and inverted edges. This compact
form is useful for logic verification, synthesis, and testing tools.

The library crate has no dependencies. The command-line tool is published as the
separate `raig-cli` crate, which depends on `raig` and `clap`.

## Installation

Add the library to a Rust project:

```sh
cargo add raig
```

Install the command-line tool:

```sh
cargo install raig-cli
```

## License

Licensed under the MIT license.
