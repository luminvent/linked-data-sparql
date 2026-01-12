use linked_data_next::{Deserialize, Serialize};
use linked_data_sparql::sparql_graph_store::{OxigraphSparqlGraphStore, SparqlGraphStore};
use linked_data_sparql::{Sparql, SparqlQuery};

#[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
#[ld(prefix("ex" = "http://ex/"))]
struct Struct {
  #[ld("ex:field_0")]
  field_0: String,

  #[ld("ex:field_1")]
  field_1: Option<String>,

  #[ld("ex:field_2")]
  field_2: Option<OptionalSubStruct>,
}

#[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
#[ld(prefix("ex" = "http://ex/"))]
struct OptionalSubStruct {
  #[ld("ex:sub_field_0")]
  sub_field_0: String,
  #[ld("ex:sub_field_1")]
  sub_field_1: String,
  #[ld("ex:sub_field_2")]
  sub_field_2: Option<String>,
}

#[test]
fn query_struct() {
  let query = Struct::sparql_query_algebra();

  println!("{:?}", query.to_string());
}

#[tokio::test]
async fn test_struct_with_option_to_some() {
  let expected = Struct {
    field_0: "zero".to_owned(),
    field_1: Some("one".to_owned()),
    field_2: None,
  };

  let store = OxigraphSparqlGraphStore::default();

  store.default_insert(&expected).await.unwrap();

  let query_results = store.query(Struct::sparql_query_algebra()).await.unwrap();

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let actual = query_result_dataset
    .deserialize_subject::<Struct>()
    .unwrap();

  assert_eq!(expected, actual);
}

#[tokio::test]
async fn test_struct_with_option_to_none() {
  let expected = Struct {
    field_0: "zero".to_owned(),
    field_1: None,
    field_2: None,
  };

  let store = OxigraphSparqlGraphStore::default();

  store.default_insert(&expected).await.unwrap();

  let query_results = store.query(Struct::sparql_query_algebra()).await.unwrap();

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let actual = query_result_dataset
    .deserialize_subject::<Struct>()
    .unwrap();

  assert_eq!(expected, actual);
}

#[tokio::test]
async fn test_struct_with_option_of_struct() {
  let expected = Struct {
    field_0: "zero".to_owned(),
    field_1: None,
    field_2: Some(OptionalSubStruct {
      sub_field_0: "sub_zero".to_owned(),
      sub_field_1: "sub_one".to_owned(),
      sub_field_2: Some("sub_two".to_owned()),
    }),
  };
  let store = OxigraphSparqlGraphStore::default();

  store.default_insert(&expected).await.unwrap();

  let query_results = store.query(Struct::sparql_query_algebra()).await.unwrap();

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let actual = query_result_dataset
    .deserialize_subject::<Struct>()
    .unwrap();

  assert_eq!(expected, actual);
}
