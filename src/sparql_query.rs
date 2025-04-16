use spargebra::Query;

pub trait SparqlQuery {
  fn sparql_query() -> String {
    Self::sparql_algebra().to_string()
  }

  fn as_sparql_query(&self) -> String {
    self.as_sparql_algebra().to_string()
  }

  fn sparql_algebra() -> Query;

  fn as_sparql_algebra(&self) -> Query {
    Self::sparql_algebra()
  }
}
