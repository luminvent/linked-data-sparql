use crate::sparql_graph_store::query_result_dataset::QueryResultDataset;
use oxrdf::{Dataset, GraphName, Quad, Triple, Variable};
use sparesults::QuerySolution;

#[derive(Debug)]
pub enum QueryResults {
  Boolean(bool),
  Solutions {
    variables: Vec<Variable>,
    solutions: Vec<QuerySolution>,
  },
  Triples(Vec<Triple>),
}

impl QueryResults {
  pub fn get_dataset(&self) -> Option<Dataset> {
    if let Self::Triples(triples) = &self {
      Some(
        triples
          .iter()
          .map(|triple| {
            Quad::new(
              triple.subject.clone(),
              triple.predicate.clone(),
              triple.object.clone(),
              GraphName::DefaultGraph,
            )
          })
          .collect(),
      )
    } else {
      None
    }
  }

  pub fn get_query_result_dataset(&self) -> Option<QueryResultDataset> {
    self.get_dataset().map(QueryResultDataset::new)
  }
}
