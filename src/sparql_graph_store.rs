mod client_sparql_graph_store;
mod oxigraph_sparql_graph_store;
mod query_result_dataset;
mod query_results;

pub use client_sparql_graph_store::SparqlClientDatabase;
use oxigraph::sparql::UpdateEvaluationError;
pub use oxigraph_sparql_graph_store::OxigraphSparqlGraphStore;
use oxrdf::Quad;
pub use query_results::QueryResults;
use spareval::QueryEvaluationError;
use std::fmt::Display;
use std::str::FromStr;

use crate::ToRdfTerm;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum UpdateAction {
  #[default]
  Insert,
  Delete,
}

impl Display for UpdateAction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      UpdateAction::Insert => write!(f, "INSERT"),
      UpdateAction::Delete => write!(f, "DELETE"),
    }
  }
}

pub trait SparqlGraphStore {
  fn generate_prepared_sparql_update(
    data: &impl ToRdfTerm,
    update_action: UpdateAction,
  ) -> Result<spargebra::Update, String> {
    let mut quads: Vec<Quad> = Vec::new();
    data.to_term(&mut quads);

    let triples = quads
      .iter()
      .map(|quad| format!("{quad} ."))
      .collect::<Vec<_>>()
      .join("\n")
      + "\n";

    let update = format!(
      r#"
      {} DATA {{
        {}
      }}
    "#,
      update_action, triples
    );

    spargebra::Update::from_str(&update).map_err(|e| e.to_string())
  }

  #[cfg(not(target_arch = "wasm32"))]
  fn default_insert(
    &self,
    data: &impl ToRdfTerm,
    update_action: UpdateAction,
  ) -> impl Future<Output = Result<(), UpdateEvaluationError>> + Send + '_ {
    let update = Self::generate_prepared_sparql_update(data, update_action).unwrap();
    log::trace!("{}", update);
    self.update(update)
  }

  #[cfg(target_arch = "wasm32")]
  fn default_insert(
    &self,
    data: &impl ToRdfTerm,
    update_action: UpdateAction,
  ) -> impl Future<Output = Result<(), UpdateEvaluationError>> + '_ {
    let update = Self::generate_prepared_sparql_update(data, update_action).unwrap();
    self.update(update)
  }

  #[cfg(not(target_arch = "wasm32"))]
  fn update(
    &self,
    update: spargebra::Update,
  ) -> impl Future<Output = Result<(), UpdateEvaluationError>> + Send + '_;

  #[cfg(target_arch = "wasm32")]
  fn update(
    &self,
    update: spargebra::Update,
  ) -> impl Future<Output = Result<(), UpdateEvaluationError>> + '_;

  #[cfg(not(target_arch = "wasm32"))]
  fn query(
    &self,
    query: spargebra::Query,
  ) -> impl Future<Output = Result<QueryResults, QueryEvaluationError>> + Send + '_;

  #[cfg(target_arch = "wasm32")]
  fn query(
    &self,
    query: spargebra::Query,
  ) -> impl Future<Output = Result<QueryResults, QueryEvaluationError>> + '_;
}
