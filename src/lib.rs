//! SPARQL query generation for linked data.
//!
//! ```rust
//! use linked_data_sparql::{Sparql, SparqlQuery};
//!
//! #[derive(Sparql, Debug, PartialEq)]
//! #[ld(prefix("ex" = "http://example.org/"))]
//! struct Person {
//!   #[ld("ex:name")]
//!   name: String,
//!
//!   #[ld("ex:age")]
//!   age: u32,
//! }
//!
//! let _string_sparql_query = Person::sparql_query();
//! ```

mod construct_query;
pub mod sparql_graph_store;
mod sparql_query;
mod to_construct_query;

pub mod rdf_type_conversions;

pub use crate::construct_query::ConstructQuery;
pub use crate::sparql_query::SparqlQuery;
pub use crate::to_construct_query::ToConstructQuery;
pub use construct_query::and::And;
pub use construct_query::join::Join;
pub use construct_query::union::Union;
pub use linked_data_sparql_derive::Sparql;
use spargebra::Query;

pub mod reexport {
  pub use spargebra;
}

impl<T> SparqlQuery for T
where
  T: ToConstructQuery,
{
  fn sparql_query_algebra() -> Query {
    Self::to_query().into()
  }
}
