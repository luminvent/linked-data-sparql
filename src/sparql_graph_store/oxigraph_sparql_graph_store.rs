use super::{QueryResults, SparqlGraphStore};
use oxigraph::{
  sparql::{QueryEvaluationError, SparqlEvaluator, UpdateEvaluationError},
  store::Store,
};
use spargebra::{Query, Update};

#[derive(Clone)]
pub struct OxigraphSparqlGraphStore {
  store: Store,
}

impl OxigraphSparqlGraphStore {
  #[cfg(not(target_arch = "wasm32"))]
  /// Creates a new store backed by the given database directory.
  pub fn new(database_directory: &str) -> Self {
    let store = Store::open(database_directory).unwrap();

    Self { store }
  }
}

impl Default for OxigraphSparqlGraphStore {
  /// Creates a new in-memory store.
  fn default() -> Self {
    let store = Store::new().unwrap();

    Self { store }
  }
}

impl SparqlGraphStore for OxigraphSparqlGraphStore {
  async fn update(&self, update: Update) -> Result<(), UpdateEvaluationError> {
    SparqlEvaluator::new()
      .for_update(update)
      .on_store(&self.store)
      .execute()
  }

  async fn query(&self, query: Query) -> Result<QueryResults, QueryEvaluationError> {
    SparqlEvaluator::new()
      .for_query(query)
      .on_store(&self.store)
      .execute()
      .map(|query_results| match query_results {
        oxigraph::sparql::QueryResults::Boolean(boolean) => QueryResults::Boolean(boolean),
        oxigraph::sparql::QueryResults::Graph(triple_iter) => {
          QueryResults::Triples(triple_iter.flatten().collect())
        }
        oxigraph::sparql::QueryResults::Solutions(solution_iter) => {
          let variables = solution_iter.variables().to_vec();
          let solutions = solution_iter.flatten().collect();

          QueryResults::Solutions {
            variables,
            solutions,
          }
        }
      })
  }
}
