use linked_data_next::LinkedData;
use rdf_types::dataset::IndexedBTreeDataset;
use rdf_types::generator::Blank;
use rdf_types::interpretation::WithGenerator;

pub trait SparqlGraphStore {
  fn insert(&mut self, data: &impl LinkedData<WithGenerator<Blank>>) -> Result<(), String>;
  fn query(&self, query: spargebra::Query) -> Result<IndexedBTreeDataset, String>;
}
