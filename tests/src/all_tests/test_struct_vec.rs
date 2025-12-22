use crate::test_graph_store::TestGraphStore;
use linked_data_next::{Deserialize, LinkedDataDeserializeSubject, Serialize};
use linked_data_sparql::sparql_graph_store::SparqlGraphStore;
use linked_data_sparql::{Sparql, SparqlQuery};
use rdf_types::Generator;
use rdf_types::generator::Blank;

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

#[test]
fn test_struct_with_vec_2_values() {
  let expected = Struct {
    field_0: "zero".to_owned(),
    field_1: vec!["one".to_owned(), "two".to_owned()],
  };

  let mut store = TestGraphStore::new();
  store.insert(&expected).unwrap();

  let query = Struct::sparql_query();
  println!("{}", query);

  let dataset = store.query(Struct::sparql_query_algebra()).unwrap();

  let resource = Blank::new().next(&mut ()).into_term();

  let actual = Struct::deserialize_subject(&(), &(), &dataset, None, &resource).unwrap();

  println!("{:?}", actual);
  assert_eq!(expected.field_0, actual.field_0);
  assert!(actual.field_1.contains(&"one".to_string()));
  assert!(actual.field_1.contains(&"two".to_string()));
}

#[test]
fn test_struct_with_vec_empty() {
  let expected = Struct {
    field_0: "zero".to_owned(),
    field_1: vec![],
  };

  let mut store = TestGraphStore::new();
  store.insert(&expected).unwrap();

  let query = Struct::sparql_query();
  println!("{}", query);

  let dataset = store.query(Struct::sparql_query_algebra()).unwrap();

  let resource = Blank::new().next(&mut ()).into_term();

  let actual = Struct::deserialize_subject(&(), &(), &dataset, None, &resource).unwrap();

  assert_eq!(expected, actual);
}

#[test]
fn test_struct_with_vec_1_value() {
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
    #[ld("ex:field_0")]
    sub_field_0: String,

    #[ld("ex:field_1")]
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

  let mut store = TestGraphStore::new();
  store.insert(&expected).unwrap();

  let query = Struct::sparql_query();
  println!("{}", query);

  let dataset = store.query(Struct::sparql_query_algebra()).unwrap();

  let resource = Blank::new().next(&mut ()).into_term();

  let actual = MainStruct::deserialize_subject(&(), &(), &dataset, None, &resource).unwrap();

  println!("{:?}", actual);
  assert_eq!(expected.field_0, actual.field_0);
}
