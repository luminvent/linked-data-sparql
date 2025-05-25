# linked-data-sparql

A Rust library for generating SPARQL queries for RDF annotated Rust types. The results from those queries can be deserialized with [linked-data-rs](https://github.com/spruceid/linked-data-rs).

## Usage

See the [examples](examples/simple.rs) for basic usage.

```rust
use linked_data_sparql::{Sparql, SparqlQuery};

#[derive(Sparql, Debug, PartialEq)]
#[ld(prefix("ex" = "http://example.org/"))]
struct Person {
    #[ld("ex:name")]
    name: String,

    #[ld("ex:age")]
    age: u32,
}

fn main() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    println!("{}", person.as_sparql_query());
}
```

The `tests` crate demonstrates full round-trip serialization with [linked-data-rs](https://github.com/spruceid/linked-data-rs).

