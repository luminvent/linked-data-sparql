# Linked data Sparql

A Rust library for mapping Rust types to RDF and generating SPARQL queries from them.

From a single set of `#[ld(...)]` annotations it derives:

- **`Sparql`**: a SPARQL `CONSTRUCT` query that fetches every triple needed to rebuild a value of the type,
- **`Serialize`**: conversion of a value into RDF quads (and into a SPARQL `INSERT DATA` / `DELETE DATA` update),
- **`Deserialize`**: reconstruction of a value from the RDF dataset returned by the query,
- **`SparqlFilter`**: a filter struct that narrows the `CONSTRUCT` query down to the matching subjects.

Two graph stores are provided: an embedded [Oxigraph](https://github.com/oxigraph/oxigraph) store (in memory or on disk) and an HTTP client for any SPARQL 1.1 endpoint.

## Installation

```toml
[dependencies]
linked-data-sparql = "0.8"
```

### Features

- `rdf12`: enables RDF 1.2 / SPARQL 1.2 support in the underlying Oxigraph crates.

## Quick start

```rust
use linked_data_sparql::{Sparql, SparqlQuery};

#[derive(Sparql, Debug, PartialEq)]
#[ld(prefix("ex" = "http://example.org/"))]
struct Person {
  #[ld("ex:name")]
  name: String,

  #[ld("ex:age")]
  age: Option<u32>,
}

fn main() {
  println!("{}", Person::sparql_query());
}
```

prints (variable names are generated):

```sparql
CONSTRUCT {
  ?s <http://example.org/name> ?name .
  ?s <http://example.org/age> ?age .
} WHERE {
  SELECT * WHERE {
    ?s <http://example.org/name> ?name .
    OPTIONAL { ?s <http://example.org/age> ?age . }
  }
}
```

Run it with `cargo run --example simple`.

## Round trip with a graph store

```rust
use linked_data_sparql::sparql_graph_store::{
  OxigraphSparqlGraphStore, SparqlGraphStore, UpdateAction,
};
use linked_data_sparql::{Deserialize, Serialize, Sparql, SparqlQuery};

#[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
#[ld(prefix("ex" = "http://ex/"))]
struct Task {
  #[ld("ex:name")]
  name: String,
}

#[tokio::main]
async fn main() {
  let store = OxigraphSparqlGraphStore::default(); // in memory

  let task = Task { name: "write tests".to_owned() };

  // INSERT DATA { ... }
  store.default_insert(&task, UpdateAction::Insert).await.unwrap();

  // CONSTRUCT { ... } WHERE { ... }
  let results = store.query(Task::sparql_query_algebra()).await.unwrap();
  let dataset = results.get_query_result_dataset().unwrap();

  let actual: Task = dataset.deserialize_subject().unwrap();
  assert_eq!(task, actual);
}
```

`QueryResultDataset` exposes:

| Method | Returns |
| --- | --- |
| `deserialize_subject::<T>()` | the first subject that deserializes as `T` |
| `deserialize_subjects::<T>()` | every subject that deserializes as `T` |
| `deserialize_subject_with_resource_id::<T>(&id)` | the subject `id` as `T` |
| `deserialize_subject_with_resource_ids::<T>(ids)` | the given subjects as `T` |
| `resource_ids()` | every subject of the dataset |

### Graph stores

Both stores implement the `SparqlGraphStore` trait (`query`, `update`, `default_insert`):

- `OxigraphSparqlGraphStore::default()`: in-memory Oxigraph store; `OxigraphSparqlGraphStore::new("path/to/db")` opens an on-disk one.
- `SparqlClientDatabase::new(update_endpoint, query_endpoint)`: remote SPARQL 1.1 endpoint over HTTP. Use `new_with_header_map` (e.g. for authentication) or `new_with_header_map_and_client` to provide your own `reqwest::Client`.

## Annotations

### Container attributes

| Attribute | Meaning |
| --- | --- |
| `#[ld(prefix("ex" = "http://example.org/"))]` | declares a prefix usable in predicates (repeatable) |
| `#[ld(type = "http://example.org/Person")]` | adds (and requires) `?s rdf:type <http://example.org/Person>` |

### Field attributes

| Attribute | Meaning |
| --- | --- |
| `#[ld("ex:name")]` | the field is the object of the `ex:name` predicate |
| `#[ld(id)]` | the field (a `NamedNode`) is the IRI of the subject; without it, the subject is a blank node |
| `#[ld(flatten)]` | the predicates of the nested struct are attached directly to the current subject |
| `#[ld(ignore)]` | the field is not mapped; it is `Default::default()` when deserialized |

### Supported field types

- `String`, `bool`, integers (`u8`..`u64`, `i8`..`i64`, as `xsd:integer`), `f32` (`xsd:float`), `f64` (`xsd:double`), `xsd_types::DateTime` (`xsd:dateTime`), `oxrdf::NamedNode` (IRI), `Box<T>`,
- any type deriving `Sparql` / `Serialize` / `Deserialize` (nested as its own subject),
- `Option<T>`: optional predicate (`OPTIONAL` in the query),
- `Vec<T>`: ordered, written as an RDF Collection (`rdf:first` / `rdf:rest` / `rdf:nil`); reading also accepts RDF Containers (`rdf:_1`, `rdf:_2`, ...) and plain repeated properties,
- `HashSet<T>`: unordered, written as a repeated property.

### Enums

An enum whose variants each wrap one value is a tagged union: the variant is selected by the predicate linking the subject to its value.

```rust
#[derive(Sparql, Serialize, Deserialize)]
#[ld(prefix("ex" = "http://ex/"))]
enum Shape {
  #[ld("ex:circle")]
  Circle(Circle),

  #[ld("ex:label")]
  Label(String),

  // The value is wrapped in a blank node: ?s ex:tagged [ ex:value "..." ]
  #[ld("ex:tagged")]
  Tagged(#[ld("ex:value")] String),
}
```

An enum made only of unit variants is a closed list of RDF resources (e.g. a controlled vocabulary), each variant being identified by its own IRI:

```rust
#[derive(Sparql, Serialize, Deserialize)]
#[ld(prefix("ex" = "http://ex/status#"))]
enum Status {
  #[ld("ex:active")]
  Active,

  #[ld("ex:inactive")]
  Inactive,
}
```

## Filtering

`#[derive(SparqlFilter)]` builds a filter struct whose fields constrain the objects of a predicate. Pass it to `ToConstructQuery::to_filtered_query` to keep only the matching subjects:

```rust
use linked_data_sparql::reexport::oxrdf::NamedNode;
use linked_data_sparql::{Sparql, SparqlFilter, ToConstructQuery};

#[derive(Sparql)]
#[ld(prefix("ex" = "http://ex/"))]
struct Book {
  #[ld(id)]
  id: NamedNode,

  #[ld("ex:title")]
  title: String,

  #[ld("ex:published")]
  published: xsd_types::DateTime,

  #[ld("ex:pages")]
  pages: u32,

  #[ld("ex:deletedAt")]
  deleted_at: Option<xsd_types::DateTime>,
}

#[derive(SparqlFilter, Default)]
#[ld(prefix("ex" = "http://ex/"))]
struct BookFilter {
  #[ld("ex:title")]
  #[filter(str_starts)]
  title_prefix: Option<String>,

  #[ld("ex:published")]
  #[filter(year)]
  published_year: Option<i32>,

  #[ld("ex:pages")]
  #[filter(>=)]
  min_pages: Option<u32>,

  #[ld("ex:deletedAt")]
  #[filter(not_exists)]
  exclude_deleted: bool,
}

let filter = BookFilter {
  title_prefix: Some("Rust".to_owned()),
  published_year: Some(2021),
  exclude_deleted: true,
  ..Default::default()
};

let query: spargebra::Query = Book::to_filtered_query(&filter).into();
```

Rules:

- An `Option` field only filters when it is `Some`, so one filter struct can express any combination of optional criteria (e.g. built from query-string parameters). A non-`Option` field always filters.
- Every condition is evaluated inside a `FILTER EXISTS { ... }`: it never adds solutions nor removes triples from the constructed graph, and a multi-valued predicate matches as soon as one of its values satisfies it.
- All the fields are combined with `&&`. A `#[ld(type = "...")]` on the filter struct also requires that `rdf:type`, and `#[ld(flatten)]` merges the conditions of a nested filter struct.
- The field value is encoded like `Serialize` does, so it matches what was stored.

`#[filter(...)]` takes comma-separated items: functions applied in order to the object, then at most one operator (`=` when omitted), then modifiers.

| Kind | Items |
| --- | --- |
| Comparison | `=` / `==` / `eq`, `!=` / `ne`, `<` / `lt`, `>` / `gt`, `<=` / `le`, `>=` / `ge` |
| String operator | `str_starts`, `str_ends`, `contains`, `regex`, `lang` (`LANGMATCHES(LANG(?o), value)`) |
| Date and time function | `year`, `month`, `day`, `hours`, `minutes`, `seconds`, `timezone`, `tz` |
| String function | `str_length`, `uppercase`, `lowercase`, `str_before("sep")`, `str_after("sep")` |
| Existence switch (`bool` field) | `exists`, `not_exists` |
| Modifier | `case_insensitive` (with `regex`), `optional` (subjects without the predicate also pass) |
| Value | `datatype = "xsd:dateTime"` types the field value, `now` compares with `NOW()` (`bool` field) |

Examples:

```rust
#[filter(str_after("@"), lowercase, str_starts)]
email_domain_starts: Option<String>,

#[filter(regex, case_insensitive)]
title_regex: Option<String>,

#[filter(<, optional)]
not_deleted_or_deleted_before: Option<xsd_types::DateTime>,

#[filter(>=, datatype = "xsd:dateTime")]
born_from: Option<String>,

#[filter(>=, now)]
upcoming: bool,
```

Filters can also be built by hand with `PredicateFilter`, `FilterMode`, `Condition`, `Operator` and `ValueFunction`, by implementing `ToSparqlFilter`.

## Composing queries

`ToConstructQuery::to_query()` returns a `ConstructQuery`, which can be combined with the `Join`, `Union` and `And` traits and filtered with `ConstructQuery::filter` before being converted into a `spargebra::Query` with `.into()`. `oxrdf` and `spargebra` are re-exported under `linked_data_sparql::reexport`.

## Development

```sh
cargo test --all-features --workspace
cargo clippy --all-targets --all-features --workspace -- -D warnings
cargo fmt --all -- --check
```

The `tests` crate covers full round trips (insert, query, deserialize) for every supported shape.

## License

MIT, see [LICENSE](LICENSE).
