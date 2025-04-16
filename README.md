# linked-data-sparql

A Rust library for generating SPARQL queries for RDF annotated Rust types. The results from those queries can be deserialized with [linked-data-rs](https://github.com/spruceid/linked-data-rs).

## Usage

See the [examples](examples/simple.rs) for basic usage.

The `tests` crate demonstrates full round-trip serialization with [linked-data-rs](https://github.com/spruceid/linked-data-rs).
