use linked_data_next::{Deserialize, Serialize};
use linked_data_sparql::sparql_graph_store::{OxigraphSparqlGraphStore, SparqlGraphStore};
use linked_data_sparql::{Sparql, SparqlQuery};

#[tokio::test]
async fn test_struct_flatten() {
  #[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
  #[ld(prefix("ex" = "http://ex/"))]
  struct Struct {
    #[ld("ex:field_0")]
    field_0: String,

    #[ld("ex:field_1")]
    field_1: String,
  }

  #[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
  #[ld(prefix("ex" = "http://ex/"))]
  struct StructFlatten {
    #[ld(flatten)]
    child: Struct,
  }

  let expected = StructFlatten {
    child: Struct {
      field_0: "zero".to_owned(),
      field_1: "one".to_owned(),
    },
  };

  let store = OxigraphSparqlGraphStore::default();

  store.default_insert(&expected).await.unwrap();

  let query_results = store
    .query(StructFlatten::sparql_query_algebra())
    .await
    .unwrap();

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let actual = query_result_dataset
    .deserialize_subject::<StructFlatten>()
    .unwrap();

  assert_eq!(expected, actual);
}
