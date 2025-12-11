use crate::test_graph_store::TestGraphStore;
use linked_data_next::{Deserialize, LinkedDataDeserializeSubject, Serialize};
use linked_data_sparql::sparql_graph_store::SparqlGraphStore;
use linked_data_sparql::{Sparql, SparqlQuery};
use rdf_types::Generator;
use rdf_types::generator::Blank;

#[test]
fn test_struct_flatten() {
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

  let mut store = TestGraphStore::new();
  store.insert(&expected).unwrap();

  let dataset = store.query(StructFlatten::sparql_algebra()).unwrap();

  let resource = Blank::new().next(&mut ()).into_term();

  let actual = StructFlatten::deserialize_subject(&(), &(), &dataset, None, &resource).unwrap();

  assert_eq!(expected, actual);
}
