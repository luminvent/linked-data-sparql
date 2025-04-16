use spargebra::Query;

pub trait SparqlQuery {
  fn sparql_query() -> String {
    Self::sparql_algebra().to_string()
  }

  fn sparql_algebra() -> Query;
}
