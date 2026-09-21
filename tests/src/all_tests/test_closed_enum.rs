use linked_data_sparql::sparql_graph_store::{
  OxigraphSparqlGraphStore, SparqlGraphStore, UpdateAction,
};
use linked_data_sparql::{Deserialize, Serialize, Sparql, SparqlQuery};

#[tokio::test]
async fn test_closed_enum() {
  // A closed list of RDF resources: every variant is a unit variant, so it's a leaf identified
  // by its own IRI (e.g. a controlled vocabulary) rather than a tagged union wrapping values.
  #[derive(Sparql, Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
  #[ld(prefix("ex" = "http://ex/status#"))]
  enum Status {
    #[ld("ex:active")]
    Active,

    #[ld("ex:inactive")]
    Inactive,
  }

  #[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
  #[ld(prefix("ex" = "http://ex/"))]
  struct Task {
    #[ld("ex:name")]
    name: String,

    #[ld("ex:status")]
    status: Status,
  }

  let expected = Task {
    name: "write tests".to_owned(),
    status: Status::Active,
  };

  let store = OxigraphSparqlGraphStore::default();

  store
    .default_insert(&expected, UpdateAction::Insert)
    .await
    .unwrap();

  let query_results = store.query(Task::sparql_query_algebra()).await.unwrap();

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let actual = query_result_dataset.deserialize_subject::<Task>().unwrap();

  assert_eq!(expected, actual);
}
