use spargebra::Query;

pub trait SparqlQuery {
  fn sparql_query() -> String {
    Self::sparql_query_algebra().to_string()
  }

  fn sparql_query_algebra() -> Query;
}
