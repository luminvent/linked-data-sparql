use crate::rdf_type_conversions::IntoRdfTypes;
use linked_data_next::{LinkedData, to_quads_with};
use oxigraph::sparql::QueryResults;
use oxigraph::store::Store;
use oxttl::NQuadsParser;
use rdf_types::RdfDisplay;
use rdf_types::dataset::IndexedBTreeDataset;
use rdf_types::generator::Blank;
use rdf_types::interpretation::WithGenerator;

pub struct TestGraphStore {
  store: Store,
}

impl TestGraphStore {
  pub fn new() -> Self {
    let store = Store::new().unwrap();

    Self { store }
  }

  pub fn insert(&mut self, data: &impl LinkedData<WithGenerator<Blank>>) {
    let mut interpretation = WithGenerator::new((), Blank::new());

    let triples = to_quads_with(&mut (), &mut interpretation, data)
      .unwrap()
      .iter()
      .map(|quad| format!("{} .", quad.rdf_display()))
      .collect::<Vec<_>>()
      .join("\n")
      + "\n";

    let data = triples.into_bytes();

    let quads = NQuadsParser::new().for_slice(&data);

    quads.filter_map(Result::ok).for_each(|quad| {
      self.store.insert(&quad).unwrap();
    });
  }

  pub fn query(&self, query: spargebra::Query) -> IndexedBTreeDataset {
    let mut expected_dataset = IndexedBTreeDataset::new();

    if let QueryResults::Graph(triples) = self.store.query(query).unwrap() {
      triples.filter_map(Result::ok).for_each(|triple| {
        let quad = triple.into_rdf_types();
        println!("{:?}", quad);
        expected_dataset.insert(quad);
      });
    } else {
      panic!();
    }

    expected_dataset
  }
}
