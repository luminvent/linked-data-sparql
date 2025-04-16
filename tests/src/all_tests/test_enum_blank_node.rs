use crate::test_graph_store::TestGraphStore;
use linked_data_next::{Deserialize, LinkedDataDeserializeSubject, Serialize};
use linked_data_sparql::{Sparql, SparqlQuery};
use rdf_types::Generator;
use rdf_types::generator::Blank;

#[test]
fn test_enum_blank_node() {
  #[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
  #[ld(prefix("ex" = "http://ex/"))]
  enum EnumBlankNode {
    #[ld("ex:left")]
    Left(#[ld("ex:value")] String),
  }

  let expected = EnumBlankNode::Left("value".to_owned());

  let mut store = TestGraphStore::new();
  store.insert(&expected);

  let dataset = store.query(EnumBlankNode::sparql_algebra());

  let resource = Blank::new().next(&mut ()).into_term();

  let actual = EnumBlankNode::deserialize_subject(&(), &(), &dataset, None, &resource).unwrap();

  assert_eq!(expected, actual);
}
