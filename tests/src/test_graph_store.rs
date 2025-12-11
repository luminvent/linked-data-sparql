use linked_data_next::{LinkedData, to_quads_with};
use linked_data_sparql::rdf_type_conversions::IntoRdfTypes;
use linked_data_sparql::sparql_graph_store::SparqlGraphStore;
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
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
}

impl SparqlGraphStore for TestGraphStore {
  fn insert(&mut self, data: &impl LinkedData<WithGenerator<Blank>>) -> Result<(), String> {
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

    self
      .store
      .extend(quads.filter_map(Result::ok))
      .map_err(|e| e.to_string())?;

    Ok(())
  }

  fn query(&self, query: spargebra::Query) -> Result<IndexedBTreeDataset, String> {
    let mut expected_dataset = IndexedBTreeDataset::new();

    if let QueryResults::Graph(triples) = SparqlEvaluator::new()
      .for_query(query)
      .on_store(&self.store)
      .execute()
      .map_err(|e| e.to_string())?
    {
      triples.filter_map(Result::ok).for_each(|triple| {
        let quad = triple.into_rdf_types();
        println!("{:?}", quad);
        expected_dataset.insert(quad);
      });
    } else {
      return Err("No graph".to_string());
    }

    Ok(expected_dataset)
  }
}
