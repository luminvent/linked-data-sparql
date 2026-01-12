use linked_data_next::{Deserialize, Serialize};
use linked_data_sparql::sparql_graph_store::{OxigraphSparqlGraphStore, SparqlGraphStore};
use linked_data_sparql::{ConstructQuery, SparqlQuery, ToConstructQuery};
use spargebra::term::{NamedNode, Variable};

// NOTE Type attribute for enum missing
#[tokio::test]
#[ignore]
async fn test_enum_type() {
  #[derive(Serialize, Deserialize, Debug, PartialEq)]
  #[ld(type = "http://ex/Type")]
  #[ld(prefix("ex" = "http://ex/"))]
  enum EnumType {
    #[ld(type = "http://ex/Type")]
    #[ld("ex:left")]
    Left(String),
  }

  impl ToConstructQuery for EnumType {
    fn to_query_with_binding(binding_variable: Variable) -> ConstructQuery {
      ConstructQuery::new_with_binding::<String>(
        binding_variable.clone(),
        NamedNode::new_unchecked("http://ex/left"),
      )
      .join_with(
        binding_variable.clone(),
        NamedNode::new_unchecked("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
        NamedNode::new_unchecked("http://ex/Type"),
      )
    }
  }

  let expected = EnumType::Left("left".to_owned());

  let store = OxigraphSparqlGraphStore::default();

  store.default_insert(&expected).await.unwrap();

  let query_results = store.query(EnumType::sparql_query_algebra()).await.unwrap();

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let actual = query_result_dataset
    .deserialize_subject::<EnumType>()
    .unwrap();

  assert_eq!(expected, actual);
}
