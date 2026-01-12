mod client_sparql_graph_store;
mod oxigraph_sparql_graph_store;
mod query_result_dataset;
mod query_results;

pub use client_sparql_graph_store::SparqlClientDatabase;
use linked_data_next::LinkedData;
use oxigraph::sparql::UpdateEvaluationError;
pub use oxigraph_sparql_graph_store::OxigraphSparqlGraphStore;
pub use query_results::QueryResults;
use rdf_types::RdfDisplay;
use rdf_types::generator::Blank;
use rdf_types::interpretation::WithGenerator;
use spareval::QueryEvaluationError;
use std::str::FromStr;

pub trait SparqlGraphStore {
  fn generate_prepared_sparql_update(
    data: &impl LinkedData<WithGenerator<Blank>>,
  ) -> Result<spargebra::Update, String> {
    let mut interpretation = WithGenerator::new((), Blank::new());

    let triples = linked_data_next::to_quads_with(&mut (), &mut interpretation, data)
      .unwrap()
      .iter()
      .map(|quad| format!("{} .", quad.rdf_display()))
      .collect::<Vec<_>>()
      .join("\n")
      + "\n";

    let update = format!(
      r#"
      INSERT DATA {{
        {}
      }}
    "#,
      triples
    );

    spargebra::Update::from_str(&update).map_err(|e| e.to_string())
  }

  fn default_insert(
    &self,
    data: &impl LinkedData<WithGenerator<Blank>>,
  ) -> impl Future<Output = Result<(), UpdateEvaluationError>> + Send + '_ {
    let update = Self::generate_prepared_sparql_update(data).unwrap();
    self.update(update)
  }

  fn update(
    &self,
    update: spargebra::Update,
  ) -> impl Future<Output = Result<(), UpdateEvaluationError>> + Send + '_;

  fn query(
    &self,
    query: spargebra::Query,
  ) -> impl Future<Output = Result<QueryResults, QueryEvaluationError>> + Send + '_;
}
