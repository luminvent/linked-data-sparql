use linked_data_next::{Deserialize, Serialize};
use linked_data_sparql::sparql_graph_store::{OxigraphSparqlGraphStore, SparqlGraphStore};
use linked_data_sparql::{Sparql, SparqlQuery};
use std::collections::HashSet;

#[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
#[ld(prefix("ex" = "http://ex/"))]
struct Struct {
  #[ld("ex:field_0")]
  field_0: String,

  #[ld("ex:field_1")]
  field_1: HashSet<String>,
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
    field_1: HashSet::from(["one".to_owned(), "two".to_owned()]),
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
  assert!(actual.field_1.contains("one"));
  assert!(actual.field_1.contains("two"));
}

#[tokio::test]
async fn test_struct_with_empty_hashset() {
  let expected = Struct {
    field_0: "zero".to_owned(),
    field_1: HashSet::new(),
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
async fn test_struct_with_hashset_of_struct() {
  #[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize, PartialEq, Sparql)]
  #[ld(prefix("ex" = "http://ex/"))]
  pub struct Title {
    #[ld("ex:name")]
    name: String,
    #[ld("ex:kind")]
    kind: String,
  }

  #[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
  #[ld(prefix("ex" = "http://ex/"))]
  struct Movie {
    #[ld("ex:title")]
    title: HashSet<Title>,
  }

  let expected = Movie {
    title: HashSet::from([
      Title {
        name: "My Title".to_string(),
        kind: "Original".to_string(),
      },
      Title {
        name: "Mon Titre".to_string(),
        kind: "Translated".to_string(),
      },
    ]),
  };

  let store = OxigraphSparqlGraphStore::default();

  store.default_insert(&expected).await.unwrap();

  let query_results = store.query(Movie::sparql_query_algebra()).await.unwrap();

  println!("{:?}", query_results);

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let actual = query_result_dataset.deserialize_subject::<Movie>().unwrap();

  assert_eq!(expected, actual);
}
