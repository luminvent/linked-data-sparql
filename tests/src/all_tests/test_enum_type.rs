use crate::test_graph_store::TestGraphStore;
use linked_data_next::{Deserialize, LinkedDataDeserializeSubject, Serialize};
use linked_data_sparql::sparql_graph_store::SparqlGraphStore;
use linked_data_sparql::{ConstructQuery, SparqlQuery, ToConstructQuery};
use rdf_types::Generator;
use rdf_types::generator::Blank;
use spargebra::term::{NamedNode, Variable};

// NOTE Type attribute for enum missing
#[test]
#[ignore]
fn test_enum_type() {
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

  let mut store = TestGraphStore::new();
  store.insert(&expected).unwrap();

  let dataset = store.query(EnumType::sparql_query_algebra()).unwrap();

  let resource = Blank::new().next(&mut ()).into_term();

  let actual = EnumType::deserialize_subject(&(), &(), &dataset, None, &resource).unwrap();

  assert_eq!(expected, actual);
}
