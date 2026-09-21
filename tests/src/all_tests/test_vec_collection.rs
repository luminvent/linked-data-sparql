//! `Vec` fields are written as an RDF Collection (`rdf:first`/`rdf:rest`/`rdf:nil`). These tests
//! exercise the (de)serialization support functions directly, independently of the SPARQL
//! `CONSTRUCT` query machinery already covered by `test_struct_vec`, and check that reading
//! accepts RDF Containers (`rdf:_1`, `rdf:_2`, ...) and plain multi-valued properties too, since
//! other RDF producers may use either shape.

use linked_data_sparql::rdf_serde_support::{vec_field, write_collection};
use linked_data_sparql::reexport::oxrdf::{
  Dataset, GraphName, Literal, NamedNode, NamedOrBlankNode, Quad, Term,
};

const RDF_FIRST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#first";
const RDF_REST: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#rest";
const RDF_NIL: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil";

fn subject() -> NamedOrBlankNode {
  NamedOrBlankNode::NamedNode(NamedNode::new_unchecked("http://ex/subject"))
}

fn predicate() -> NamedNode {
  NamedNode::new_unchecked("http://ex/items")
}

#[test]
fn write_collection_emits_rdf_first_rest_nil_chain() {
  let subject = subject();
  let predicate = predicate();
  let mut quads = Vec::new();

  write_collection(
    &subject,
    &predicate,
    &["a".to_owned(), "b".to_owned()],
    &mut quads,
  );

  // subject predicate head . head rdf:first "a" . head rdf:rest tail .
  // tail rdf:first "b" . tail rdf:rest rdf:nil .
  assert_eq!(quads.len(), 5);

  let head_quad = quads
    .iter()
    .find(|quad| quad.subject == subject)
    .expect("subject predicate head triple");
  assert_eq!(head_quad.predicate, predicate);
  let head = NamedOrBlankNode::try_from(head_quad.object.clone()).unwrap();

  let head_first = quads
    .iter()
    .find(|quad| quad.subject == head && quad.predicate.as_str() == RDF_FIRST)
    .expect("head rdf:first triple");
  assert_eq!(head_first.object.to_string(), "\"a\"");

  let head_rest = quads
    .iter()
    .find(|quad| quad.subject == head && quad.predicate.as_str() == RDF_REST)
    .expect("head rdf:rest triple");
  let tail = NamedOrBlankNode::try_from(head_rest.object.clone()).unwrap();
  assert_ne!(tail, head);

  let tail_first = quads
    .iter()
    .find(|quad| quad.subject == tail && quad.predicate.as_str() == RDF_FIRST)
    .expect("tail rdf:first triple");
  assert_eq!(tail_first.object.to_string(), "\"b\"");

  let tail_rest = quads
    .iter()
    .find(|quad| quad.subject == tail && quad.predicate.as_str() == RDF_REST)
    .expect("tail rdf:rest triple");
  assert_eq!(
    tail_rest.object,
    Term::from(NamedNode::new_unchecked(RDF_NIL))
  );
}

#[test]
fn write_collection_of_empty_vec_is_rdf_nil() {
  let subject = subject();
  let predicate = predicate();
  let mut quads = Vec::new();

  write_collection(&subject, &predicate, &Vec::<String>::new(), &mut quads);

  assert_eq!(
    quads,
    vec![Quad::new(
      subject,
      predicate,
      NamedNode::new_unchecked(RDF_NIL),
      GraphName::DefaultGraph,
    )]
  );
}

#[test]
fn vec_field_round_trips_through_write_collection() {
  let subject = subject();
  let predicate = predicate();
  let mut quads = Vec::new();

  write_collection(
    &subject,
    &predicate,
    &["a".to_owned(), "b".to_owned(), "c".to_owned()],
    &mut quads,
  );

  let dataset: Dataset = quads.into_iter().collect();
  let actual: Vec<String> = vec_field(&dataset, &subject, &predicate).unwrap();

  assert_eq!(actual, vec!["a", "b", "c"]);
}

#[test]
fn vec_field_reads_a_hand_built_rdf_collection() {
  let subject = subject();
  let predicate = predicate();

  let head = NamedOrBlankNode::BlankNode(Default::default());
  let tail = NamedOrBlankNode::BlankNode(Default::default());

  let dataset: Dataset = vec![
    Quad::new(
      subject.clone(),
      predicate.clone(),
      head.clone(),
      GraphName::DefaultGraph,
    ),
    Quad::new(
      head.clone(),
      NamedNode::new_unchecked(RDF_FIRST),
      Literal::from("one"),
      GraphName::DefaultGraph,
    ),
    Quad::new(
      head,
      NamedNode::new_unchecked(RDF_REST),
      tail.clone(),
      GraphName::DefaultGraph,
    ),
    Quad::new(
      tail.clone(),
      NamedNode::new_unchecked(RDF_FIRST),
      Literal::from("two"),
      GraphName::DefaultGraph,
    ),
    Quad::new(
      tail,
      NamedNode::new_unchecked(RDF_REST),
      NamedNode::new_unchecked(RDF_NIL),
      GraphName::DefaultGraph,
    ),
  ]
  .into_iter()
  .collect();

  let actual: Vec<String> = vec_field(&dataset, &subject, &predicate).unwrap();

  assert_eq!(actual, vec!["one", "two"]);
}

#[test]
fn vec_field_reads_an_rdf_container() {
  let subject = subject();
  let predicate = predicate();
  let container = NamedOrBlankNode::BlankNode(Default::default());

  const RDF_NS: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

  let dataset: Dataset = vec![
    Quad::new(
      subject.clone(),
      predicate.clone(),
      container.clone(),
      GraphName::DefaultGraph,
    ),
    // Deliberately out of insertion order to prove the numeric index, not insertion order,
    // determines the result.
    Quad::new(
      container.clone(),
      NamedNode::new_unchecked(format!("{RDF_NS}_2")),
      Literal::from("two"),
      GraphName::DefaultGraph,
    ),
    Quad::new(
      container.clone(),
      NamedNode::new_unchecked(format!("{RDF_NS}_10")),
      Literal::from("ten"),
      GraphName::DefaultGraph,
    ),
    Quad::new(
      container,
      NamedNode::new_unchecked(format!("{RDF_NS}_1")),
      Literal::from("one"),
      GraphName::DefaultGraph,
    ),
  ]
  .into_iter()
  .collect();

  let actual: Vec<String> = vec_field(&dataset, &subject, &predicate).unwrap();

  // `_10` orders after `_2` numerically even though `"_10"` sorts before `"_2"` lexically.
  assert_eq!(actual, vec!["one", "two", "ten"]);
}

#[test]
fn vec_field_reads_plain_multi_valued_triples() {
  let subject = subject();
  let predicate = predicate();

  let dataset: Dataset = vec![
    Quad::new(
      subject.clone(),
      predicate.clone(),
      Literal::from("one"),
      GraphName::DefaultGraph,
    ),
    Quad::new(
      subject.clone(),
      predicate.clone(),
      Literal::from("two"),
      GraphName::DefaultGraph,
    ),
  ]
  .into_iter()
  .collect();

  let mut actual: Vec<String> = vec_field(&dataset, &subject, &predicate).unwrap();
  actual.sort();

  assert_eq!(actual, vec!["one", "two"]);
}
