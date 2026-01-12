use linked_data_next::{Deserialize, Serialize};
use linked_data_sparql::sparql_graph_store::{OxigraphSparqlGraphStore, SparqlGraphStore};
use linked_data_sparql::{Sparql, SparqlQuery};

#[tokio::test]
async fn test_enum_blank_node() {
  #[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
  #[ld(prefix("ex" = "http://ex/"))]
  enum EnumBlankNode {
    #[ld("ex:left")]
    Left(#[ld("ex:value")] String),
  }

  let expected = EnumBlankNode::Left("value".to_owned());

  let store = OxigraphSparqlGraphStore::default();

  store.default_insert(&expected).await.unwrap();

  let query_results = store
    .query(EnumBlankNode::sparql_query_algebra())
    .await
    .unwrap();

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let actual = query_result_dataset
    .deserialize_subject::<EnumBlankNode>()
    .unwrap();

  assert_eq!(expected, actual);
}
