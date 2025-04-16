use crate::test_graph_store::TestGraphStore;
use linked_data_next::{Deserialize, LinkedDataDeserializeSubject, Serialize};
use linked_data_sparql::{Sparql, SparqlQuery};
use rdf_types::Generator;
use rdf_types::generator::Blank;

#[test]
fn test_enum() {
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
  enum Enum {
    #[ld("ex:left")]
    Left(String),

    #[ld("ex:right")]
    Right(Struct),
  }

  let expected = Enum::Right(Struct {
    field_0: "zero".to_owned(),
    field_1: "one".to_owned(),
  });

  let mut store = TestGraphStore::new();
  store.insert(&expected);

  let dataset = store.query(Enum::sparql_algebra());

  let resource = Blank::new().next(&mut ()).into_term();

  let actual = Enum::deserialize_subject(&(), &(), &dataset, None, &resource).unwrap();

  assert_eq!(expected, actual);
}
