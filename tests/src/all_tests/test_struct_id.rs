use iref::IriBuf;
use linked_data_next::{Deserialize, Serialize};
use linked_data_sparql::sparql_graph_store::{OxigraphSparqlGraphStore, SparqlGraphStore};
use linked_data_sparql::{Sparql, SparqlQuery};

#[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
#[ld(prefix("ex" = "http://ex/"))]
struct StructId {
  #[ld(id)]
  id: IriBuf,

  #[ld("ex:field")]
  value: String,
}

#[tokio::test]
async fn test_struct_id() {
  let id = IriBuf::new("http://example.org/myBar".to_string()).unwrap();
  let expected = StructId {
    id: id.clone(),
    value: "value".to_owned(),
  };

  let store = OxigraphSparqlGraphStore::default();

  store.default_insert(&expected).await.unwrap();

  let query_results = store.query(StructId::sparql_query_algebra()).await.unwrap();

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let actual = query_result_dataset
    .deserialize_subject::<StructId>()
    .unwrap();

  assert_eq!(expected, actual);
}

#[tokio::test]
async fn test_struct_id_multiple() {
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

  let store = OxigraphSparqlGraphStore::default();

  store.default_insert(&expected_1).await.unwrap();
  store.default_insert(&expected_2).await.unwrap();

  let query_results = store.query(StructId::sparql_query_algebra()).await.unwrap();

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let resource_1 = <rdf_types::Term as rdf_types::FromIri>::from_iri(id_1);
  let actual = query_result_dataset
    .deserialize_subject_with_resource_id::<StructId>(&resource_1)
    .unwrap();

  assert_eq!(expected_1, actual);

  let resource_2 = <rdf_types::Term as rdf_types::FromIri>::from_iri(id_2);
  let actual = query_result_dataset
    .deserialize_subject_with_resource_id::<StructId>(&resource_2)
    .unwrap();

  assert_eq!(expected_2, actual);

  let actual = query_result_dataset.deserialize_subjects::<StructId>();

  assert!(actual.contains(&expected_1));
  assert!(actual.contains(&expected_2));
}
