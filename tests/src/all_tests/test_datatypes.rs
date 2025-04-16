use std::str::FromStr;

use crate::test_graph_store::TestGraphStore;
use linked_data_next::{Deserialize, LinkedDataDeserializeSubject, Serialize};
use linked_data_sparql::{Sparql, SparqlQuery};
use rdf_types::Generator;
use rdf_types::generator::Blank;

#[test]
fn test_datatypes() {
  /// NOTE Commented out datatypes are not preserved by oxigraph
  /// https://github.com/oxigraph/oxigraph/issues/526
  #[derive(Sparql, Serialize, Deserialize, Debug, PartialEq)]
  #[ld(prefix("ex" = "http://ex/"))]
  struct Datatypes {
    // #[ld("ex:u8")]
    // u8: u8,
    // #[ld("ex:u16")]
    // u16: u16,
    // #[ld("ex:u32")]
    // u32: u32,
    #[ld("ex:u64")]
    u64: u64,
    // #[ld("ex:i8")]
    // i8: i8,
    // #[ld("ex:i16")]
    // i16: i16,
    // #[ld("ex:i32")]
    // i32: i32,
    // #[ld("ex:i64")]
    // i64: i64,
    #[ld("ex:String")]
    string: String,
    #[ld("ex:DateTime")]
    date_time: xsd_types::DateTime,
  }

  let expected = Datatypes {
    // u8: 255,
    // u16: 65535,
    // u32: 4294967295,
    u64: 18446744073709551615,
    // i8: -128,
    // i16: -32768,
    // i32: -2147483648,
    // i64: -9223372036854775808,
    string: "test string".to_owned(),
    date_time: xsd_types::DateTime::from_str("2024-01-15T12:30:45Z").unwrap(),
  };

  let mut store = TestGraphStore::new();
  store.insert(&expected);

  let dataset = store.query(Datatypes::sparql_algebra());

  let resource = Blank::new().next(&mut ()).into_term();

  let actual = Datatypes::deserialize_subject(&(), &(), &dataset, None, &resource).unwrap();

  assert_eq!(expected, actual);
}
