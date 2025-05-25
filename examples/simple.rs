use linked_data_sparql::{Sparql, SparqlQuery};

#[derive(Sparql, Debug, PartialEq)]
#[ld(prefix("ex" = "http://example.org/"))]
struct Person {
    #[ld("ex:name")]
    name: String,

    #[ld("ex:age")]
    age: u32,
}

fn main() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
    };

    println!("{}", person.as_sparql_query());
}
