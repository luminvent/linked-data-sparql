use linked_data_next::{Deserialize, Serialize};
use linked_data_sparql::sparql_graph_store::{OxigraphSparqlGraphStore, SparqlGraphStore};
use linked_data_sparql::{Sparql, SparqlQuery};

#[tokio::test]
async fn test_struct_type() {
  #[derive(Sparql, Serialize, Deserialize, Debug, Default, PartialEq)]
  #[ld(type = "http://ex/Type")]
  #[ld(prefix("ex" = "http://ex/"))]
  struct StructType {
    #[ld("ex:field")]
    field: String,
  }

  let expected = StructType {
    field: "type_field".to_owned(),
  };

  let store = OxigraphSparqlGraphStore::default();

  store.default_insert(&expected).await.unwrap();

  let query_results = store
    .query(StructType::sparql_query_algebra())
    .await
    .unwrap();

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let actual = query_result_dataset
    .deserialize_subject::<StructType>()
    .unwrap();

  assert_eq!(expected, actual);
}
