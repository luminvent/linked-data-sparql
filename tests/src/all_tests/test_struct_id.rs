use crate::test_graph_store::TestGraphStore;
use iref::IriBuf;
use linked_data_next::{
  Deserialize, LinkedDataDeserializePredicateObjects, LinkedDataDeserializeSubject, Serialize,
};
use linked_data_sparql::{Sparql, SparqlQuery};

#[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
#[ld(prefix("ex" = "http://ex/"))]
struct StructId {
  #[ld(id)]
  id: IriBuf,

  #[ld("ex:field")]
  value: String,
}

#[test]
fn test_struct_id() {
  let id = IriBuf::new("http://example.org/myBar".to_string()).unwrap();
  let expected = StructId {
    id: id.clone(),
    value: "value".to_owned(),
  };

  let mut store = TestGraphStore::new();
  store.insert(&expected);

  let dataset = store.query(StructId::sparql_algebra());

  let resource = <rdf_types::Term as rdf_types::FromIri>::from_iri(id);

  let actual = StructId::deserialize_subject(&(), &(), &dataset, None, &resource).unwrap();

  assert_eq!(expected, actual);
}

#[test]
fn test_struct_id_multiple() {
  let id_1 = IriBuf::new("http://example.org/myBar1".to_string()).unwrap();
  let expected_1 = StructId {
    id: id_1.clone(),
    value: "value_1".to_owned(),
  };

  let id_2 = IriBuf::new("http://example.org/myBar2".to_string()).unwrap();
  let expected_2 = StructId {
    id: id_2.clone(),
    value: "value_2".to_owned(),
  };

  let mut store = TestGraphStore::new();
  store.insert(&expected_1);
  store.insert(&expected_2);

  let dataset = store.query(StructId::sparql_algebra());

  let resource_1 = <rdf_types::Term as rdf_types::FromIri>::from_iri(id_1);

  let actual = StructId::deserialize_objects(&(), &(), &dataset, None, &vec![resource_1.clone()]).unwrap();

  assert_eq!(expected_1, actual);

  let resource_2 = <rdf_types::Term as rdf_types::FromIri>::from_iri(id_2);

  let actual = StructId::deserialize_objects(&(), &(), &dataset, None, &vec![resource_2.clone()]).unwrap();

  assert_eq!(expected_2, actual);

  let actual = StructId::deserialize_subjects(&(), &(), &dataset, None, [resource_1, resource_2]).unwrap();

  assert_eq!(vec![expected_1, expected_2], actual);
}
