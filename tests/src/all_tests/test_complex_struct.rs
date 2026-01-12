use iref::IriBuf;
use linked_data_next::{Deserialize, Serialize};
use linked_data_sparql::sparql_graph_store::{OxigraphSparqlGraphStore, SparqlGraphStore};
use linked_data_sparql::{Sparql, SparqlQuery};

#[tokio::test]
async fn test_complex_struct() {
  #[allow(dead_code)]
  #[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
  #[ld(prefix("ex" = "http://ex/"))]
  struct StructId {
    #[ld(id)]
    id: IriBuf,

    #[ld("ex:field")]
    value: String,
  }

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
  struct StructType {
    #[ld("ex:field")]
    field: String,
  }

  #[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
  #[ld(prefix("ex" = "http://ex/"))]
  struct StructFlatten {
    #[ld(flatten)]
    child: Struct,
  }

  #[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
  #[ld(type = "http://ex/Type")]
  #[ld(prefix("ex" = "http://ex/"))]
  struct ComplexStruct {
    #[ld(id)]
    id: IriBuf,
    #[ld("ex:field_0_0")]
    field_0_0: String,
    #[ld("ex:struct_id")]
    id_field: StructId,
    #[ld("ex:struct_type")]
    type_field: StructType,
    #[ld("ex:struct_flatten")]
    flatten_field: StructFlatten,
  }

  let id = IriBuf::new("http://example.org/crazy".to_string()).unwrap();

  let sub_id = IriBuf::new("http://example.org/myBar".to_string()).unwrap();

  let expected = ComplexStruct {
    id: id.clone(),
    field_0_0: "field_0_0_value".to_owned(),
    id_field: StructId {
      id: sub_id.clone(),
      value: "value".to_owned(),
    },
    type_field: StructType {
      field: "type_field".to_owned(),
    },
    flatten_field: StructFlatten {
      child: Struct {
        field_0: "zero".to_owned(),
        field_1: "one".to_owned(),
      },
    },
    // crazy_field: create_crazy_enum(),
  };

  let store = OxigraphSparqlGraphStore::default();

  store.default_insert(&expected).await.unwrap();

  let query_results = store
    .query(ComplexStruct::sparql_query_algebra())
    .await
    .unwrap();

  let query_result_dataset = query_results.get_query_result_dataset().unwrap();

  let actual = query_result_dataset
    .deserialize_subject::<ComplexStruct>()
    .unwrap();

  assert_eq!(expected, actual);

  let objects = query_result_dataset.deserialize_subjects::<ComplexStruct>();

  assert_eq!(objects.len(), 1);
}
