use crate::test_graph_store::TestGraphStore;
use linked_data_next::{Deserialize, LinkedDataDeserializeSubject, Serialize};
use linked_data_sparql::sparql_graph_store::SparqlGraphStore;
use linked_data_sparql::{Sparql, SparqlQuery, ToConstructQuery};
use oxigraph::model::Variable;
use rdf_types::Generator;
use rdf_types::generator::Blank;
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

#[test]
fn test_struct_with_vec_2_values() {
  let expected = Struct {
    field_0: "zero".to_owned(),
    field_1: HashSet::from(["one".to_owned(), "two".to_owned()]),
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
  assert!(actual.field_1.contains("one"));
  assert!(actual.field_1.contains("two"));
}

#[test]
fn test_struct_with_empty_hashset() {
  let expected = Struct {
    field_0: "zero".to_owned(),
    field_1: HashSet::new(),
  };

  let mut store = TestGraphStore::new();
  store.insert(&expected).unwrap();

  let query = Struct::sparql_query();
  println!("{}", query);

  let dataset = store.query(Struct::sparql_query_algebra()).unwrap();

  println!("{:?}", dataset);
  let resource = Blank::new().next(&mut ()).into_term();

  let actual = Struct::deserialize_subject(&(), &(), &dataset, None, &resource).unwrap();

  assert_eq!(expected, actual);
}

#[test]
fn test_struct_with_hashset_of_struct() {
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

  let variable = Variable::new_unchecked(spargebra::term::BlankNode::default().into_string());
  println!("$$$$ {:?}", Title::to_query_with_binding(variable));

  let mut store = TestGraphStore::new();
  store.insert(&expected).unwrap();

  let query = Movie::sparql_query();
  println!("{}", query);

  let dataset = store.query(Movie::sparql_query_algebra()).unwrap();

  let resource = Blank::new().next(&mut ()).into_term();

  let actual = Movie::deserialize_subject(&(), &(), &dataset, None, &resource).unwrap();

  println!("{:?}", actual);
  assert_eq!(expected, actual);
}
