use linked_data_next::{Deserialize, Serialize};
use linked_data_sparql::sparql_graph_store::{OxigraphSparqlGraphStore, SparqlGraphStore};
use linked_data_sparql::{Sparql, SparqlQuery};

#[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
#[ld(prefix("ex" = "http://ex/"))]
struct Struct {
  #[ld("ex:field_0")]
  field_0: String,

  #[ld("ex:field_1")]
  field_1: Vec<String>,
}

#[test]
fn query_struct() {
  let query = Struct::sparql_query_algebra();

  println!("{:?}", query.to_string());
}

#[tokio::test]
async fn test_struct_with_vec_2_values() {
  let expected = Struct {
    field_0: "zero".to_owned(),
    field_1: vec!["one".to_owned(), "two".to_owned()],
  };

  let store = OxigraphSparqlGraphStore::default();

  store.default_insert(&expected).await.unwrap();

  let query_results = store.query(Struct::sparql_query_algebra()).await.unwrap();

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let actual = query_result_dataset
    .deserialize_subject::<Struct>()
    .unwrap();

  println!("{:?}", actual);
  assert_eq!(expected.field_0, actual.field_0);
  assert!(actual.field_1.contains(&"one".to_string()));
  assert!(actual.field_1.contains(&"two".to_string()));
}

#[tokio::test]
async fn test_struct_with_vec_empty() {
  let expected = Struct {
    field_0: "zero".to_owned(),
    field_1: vec![],
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
async fn test_struct_with_vec_1_value() {
  #[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
  #[ld(prefix("ex" = "http://ex/"))]
  struct MainStruct {
    #[ld("ex:field_0")]
    field_0: String,

    #[ld("ex:field_1")]
    field_1: Vec<SubStruct>,
  }

  #[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
  #[ld(prefix("ex" = "http://ex/"))]
  struct SubStruct {
    #[ld("ex:sub_field_0")]
    sub_field_0: String,

    #[ld("ex:sub_field_1")]
    sub_field_1: Vec<String>,
  }

  let expected = MainStruct {
    field_0: "zero".to_owned(),
    field_1: vec![
      SubStruct {
        sub_field_0: "zero".to_owned(),
        sub_field_1: vec!["one".to_owned(), "two".to_owned()],
      },
      SubStruct {
        sub_field_0: "tree".to_owned(),
        sub_field_1: vec!["four".to_owned(), "five".to_owned()],
      },
    ],
  };

  let store = OxigraphSparqlGraphStore::default();

  store.default_insert(&expected).await.unwrap();

  let query_results = store
    .query(MainStruct::sparql_query_algebra())
    .await
    .unwrap();

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let actual = query_result_dataset
    .deserialize_subject::<MainStruct>()
    .unwrap();

  assert_eq!(expected.field_0, actual.field_0);
}
