use linked_data_sparql::{Sparql, SparqlQuery};

#[derive(Sparql, Debug, PartialEq)]
#[ld(prefix("ex" = "http://example.org/"))]
struct Person {
  #[ld("ex:name")]
  name: String,

  #[ld("ex:age")]
  age: Option<u32>,
}

fn main() {
  println!("{}", Person::sparql_query());
}
