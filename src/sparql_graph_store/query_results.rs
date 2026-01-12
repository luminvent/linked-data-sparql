use crate::rdf_type_conversions::IntoRdfTypes;
use crate::sparql_graph_store::query_result_dataset::QueryResultDataset;
use oxrdf::{Triple, Variable};
use rdf_types::dataset::IndexedBTreeDataset;
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
  pub fn get_indexed_db_tree_dataset(&self) -> Option<IndexedBTreeDataset> {
    if let Self::Triples(triples) = &self {
      let mut expected_dataset = IndexedBTreeDataset::new();

      triples.iter().for_each(|triple| {
        expected_dataset.insert(triple.clone().into_rdf_types());
      });

      Some(expected_dataset)
    } else {
      None
    }
  }

  pub fn get_query_result_dataset(&self) -> Option<QueryResultDataset> {
    self
      .get_indexed_db_tree_dataset()
      .map(QueryResultDataset::new)
  }
}
