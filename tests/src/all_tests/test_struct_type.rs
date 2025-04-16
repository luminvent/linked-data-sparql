use crate::test_graph_store::TestGraphStore;
use linked_data_next::{Deserialize, LinkedDataDeserializeSubject, Serialize};
use linked_data_sparql::{Sparql, SparqlQuery};
use rdf_types::Generator;
use rdf_types::generator::Blank;

#[test]
fn test_struct_type() {
  #[derive(Sparql, Serialize, Deserialize, Debug, Default, PartialEq)]
  #[ld(type = "http://ex/Type")]
  #[ld(prefix("ex" = "http://ex/"))]
  struct StructType {
    #[ld("ex:field")]
    field: String,
  }

  let expected = StructType {
    field: "type_field".to_owned(),
  };

  let mut store = TestGraphStore::new();
  store.insert(&expected);

  let dataset = store.query(StructType::sparql_algebra());

  let resource = Blank::new().next(&mut ()).into_term();

  let actual = StructType::deserialize_subject(&(), &(), &dataset, None, &resource).unwrap();

  assert_eq!(expected, actual);
}
