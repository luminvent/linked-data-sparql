//! Runtime (de)serialization of Rust values to and from RDF quads, built directly
//! on `oxrdf` types.

use oxrdf::vocab::{rdf, xsd};
use oxrdf::{
  BlankNode, Dataset, GraphName, GraphNameRef, Literal, NamedNode, NamedOrBlankNode, Quad, Term,
};
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum DeserializeError {
  MissingField(&'static str),
  UnexpectedTerm { expected: &'static str },
  InvalidLiteral(String),
  NoMatchingVariant,
}

impl fmt::Display for DeserializeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      DeserializeError::MissingField(field) => write!(f, "missing field `{field}`"),
      DeserializeError::UnexpectedTerm { expected } => write!(f, "expected {expected}"),
      DeserializeError::InvalidLiteral(message) => write!(f, "invalid literal: {message}"),
      DeserializeError::NoMatchingVariant => write!(f, "no matching enum variant"),
    }
  }
}

impl std::error::Error for DeserializeError {}

/// Converts a value into an RDF term, emitting any quads required to describe it
/// (nested structs and enums emit their own quads and return a reference to their subject).
pub trait ToRdfTerm {
  fn to_term(&self, quads: &mut Vec<Quad>) -> Term;
}

/// Emits the quads describing `self` under the given subject.
pub trait WriteRdfQuads {
  fn write_quads(&self, subject: &NamedOrBlankNode, quads: &mut Vec<Quad>);
}

/// Reconstructs a value from an RDF term.
pub trait FromRdfTerm: Sized {
  fn from_term(dataset: &Dataset, term: &Term) -> Result<Self, DeserializeError>;
}

/// Reconstructs a value from the quads of an RDF dataset attached to the given subject.
pub trait FromRdfSubject: Sized {
  fn deserialize_subject(
    dataset: &Dataset,
    subject: &NamedOrBlankNode,
  ) -> Result<Self, DeserializeError>;
}

fn expect_literal(term: &Term) -> Result<&Literal, DeserializeError> {
  match term {
    Term::Literal(literal) => Ok(literal),
    _ => Err(DeserializeError::UnexpectedTerm {
      expected: "literal",
    }),
  }
}

macro_rules! impl_rdf_term_for_numeric {
  ($($t:ty => $datatype:expr),* $(,)?) => {
    $(
      impl ToRdfTerm for $t {
        fn to_term(&self, _quads: &mut Vec<Quad>) -> Term {
          Term::Literal(Literal::new_typed_literal(self.to_string(), $datatype))
        }
      }

      impl FromRdfTerm for $t {
        fn from_term(_dataset: &Dataset, term: &Term) -> Result<Self, DeserializeError> {
          expect_literal(term)?
            .value()
            .parse::<$t>()
            .map_err(|error| DeserializeError::InvalidLiteral(error.to_string()))
        }
      }
    )*
  };
}

impl_rdf_term_for_numeric!(
  u8 => xsd::INTEGER,
  u16 => xsd::INTEGER,
  u32 => xsd::INTEGER,
  u64 => xsd::INTEGER,
  i8 => xsd::INTEGER,
  i16 => xsd::INTEGER,
  i32 => xsd::INTEGER,
  i64 => xsd::INTEGER,
  f32 => xsd::FLOAT,
  f64 => xsd::DOUBLE,
);

impl ToRdfTerm for bool {
  fn to_term(&self, _quads: &mut Vec<Quad>) -> Term {
    Term::Literal(Literal::new_typed_literal(self.to_string(), xsd::BOOLEAN))
  }
}

impl FromRdfTerm for bool {
  fn from_term(_dataset: &Dataset, term: &Term) -> Result<Self, DeserializeError> {
    expect_literal(term)?
      .value()
      .parse::<bool>()
      .map_err(|error| DeserializeError::InvalidLiteral(error.to_string()))
  }
}

impl ToRdfTerm for String {
  fn to_term(&self, _quads: &mut Vec<Quad>) -> Term {
    Term::Literal(Literal::from(self.clone()))
  }
}

impl FromRdfTerm for String {
  fn from_term(_dataset: &Dataset, term: &Term) -> Result<Self, DeserializeError> {
    Ok(expect_literal(term)?.value().to_owned())
  }
}

impl ToRdfTerm for xsd_types::DateTime {
  fn to_term(&self, _quads: &mut Vec<Quad>) -> Term {
    Term::Literal(Literal::new_typed_literal(self.to_string(), xsd::DATE_TIME))
  }
}

impl FromRdfTerm for xsd_types::DateTime {
  fn from_term(_dataset: &Dataset, term: &Term) -> Result<Self, DeserializeError> {
    xsd_types::DateTime::from_str(expect_literal(term)?.value())
      .map_err(|error| DeserializeError::InvalidLiteral(error.to_string()))
  }
}

impl ToRdfTerm for NamedNode {
  fn to_term(&self, _quads: &mut Vec<Quad>) -> Term {
    Term::NamedNode(self.clone())
  }
}

impl FromRdfTerm for NamedNode {
  fn from_term(_dataset: &Dataset, term: &Term) -> Result<Self, DeserializeError> {
    match term {
      Term::NamedNode(named_node) => Ok(named_node.clone()),
      _ => Err(DeserializeError::UnexpectedTerm {
        expected: "named node",
      }),
    }
  }
}

impl<T: ToRdfTerm> ToRdfTerm for Box<T> {
  fn to_term(&self, quads: &mut Vec<Quad>) -> Term {
    (**self).to_term(quads)
  }
}

impl<T: FromRdfTerm> FromRdfTerm for Box<T> {
  fn from_term(dataset: &Dataset, term: &Term) -> Result<Self, DeserializeError> {
    T::from_term(dataset, term).map(Box::new)
  }
}

pub fn objects_for<'a>(
  dataset: &'a Dataset,
  subject: &NamedOrBlankNode,
  predicate: &NamedNode,
) -> impl Iterator<Item = Term> + 'a {
  let subject = subject.clone();
  let predicate = predicate.clone();
  dataset
    .graph(GraphNameRef::DefaultGraph)
    .objects_for_subject_predicate(&subject, &predicate)
    .map(|term| term.into_owned())
    .collect::<Vec<_>>()
    .into_iter()
}

pub fn single_field<T: FromRdfTerm>(
  dataset: &Dataset,
  subject: &NamedOrBlankNode,
  predicate: &NamedNode,
  field_name: &'static str,
) -> Result<T, DeserializeError> {
  let term = objects_for(dataset, subject, predicate)
    .next()
    .ok_or(DeserializeError::MissingField(field_name))?;
  T::from_term(dataset, &term)
}

pub fn optional_field<T: FromRdfTerm>(
  dataset: &Dataset,
  subject: &NamedOrBlankNode,
  predicate: &NamedNode,
) -> Result<Option<T>, DeserializeError> {
  match objects_for(dataset, subject, predicate).next() {
    Some(term) => Ok(Some(T::from_term(dataset, &term)?)),
    None => Ok(None),
  }
}

/// Writes `items` as an RDF Collection: `subject predicate _:head .`, followed by the
/// `rdf:first`/`rdf:rest` linked list rooted at `_:head` and terminated by `rdf:nil`. An empty
/// `items` is written as `subject predicate rdf:nil .` directly, with no list nodes at all.
pub fn write_collection<T: ToRdfTerm>(
  subject: &NamedOrBlankNode,
  predicate: &NamedNode,
  items: &[T],
  quads: &mut Vec<Quad>,
) {
  let mut tail = Term::NamedNode(NamedNode::from(rdf::NIL));

  for item in items.iter().rev() {
    let item_term = item.to_term(quads);
    let node = fresh_subject();
    quads.push(Quad::new(
      node.clone(),
      NamedNode::from(rdf::FIRST),
      item_term,
      GraphName::DefaultGraph,
    ));
    quads.push(Quad::new(
      node.clone(),
      NamedNode::from(rdf::REST),
      tail,
      GraphName::DefaultGraph,
    ));
    tail = Term::from(node);
  }

  quads.push(Quad::new(
    subject.clone(),
    predicate.clone(),
    tail,
    GraphName::DefaultGraph,
  ));
}

/// Reads a `Vec` field, accepting whichever RDF list shape the data actually uses:
/// an RDF Collection (`rdf:first`/`rdf:rest`/`rdf:nil`), an RDF Container (`rdf:_1`, `rdf:_2`,
/// ...), or a plain multi-valued property (repeated `subject predicate value` triples, the
/// shape `write_collection` no longer produces but which other RDF producers may still use).
pub fn vec_field<T: FromRdfTerm>(
  dataset: &Dataset,
  subject: &NamedOrBlankNode,
  predicate: &NamedNode,
) -> Result<Vec<T>, DeserializeError> {
  let objects: Vec<Term> = objects_for(dataset, subject, predicate).collect();

  let [object] = objects.as_slice() else {
    return objects
      .iter()
      .map(|term| T::from_term(dataset, term))
      .collect();
  };

  if *object == Term::NamedNode(NamedNode::from(rdf::NIL)) {
    return Ok(Vec::new());
  }

  let Some(node) = as_subject(object) else {
    return Ok(vec![T::from_term(dataset, object)?]);
  };

  if let Some(items) = read_collection(dataset, &node)? {
    return Ok(items);
  }

  if let Some(items) = read_container(dataset, &node)? {
    return Ok(items);
  }

  Ok(vec![T::from_term(dataset, object)?])
}

fn as_subject(term: &Term) -> Option<NamedOrBlankNode> {
  NamedOrBlankNode::try_from(term.clone()).ok()
}

/// Walks the `rdf:first`/`rdf:rest` chain starting at `head`, returning `None` if `head` isn't
/// a list node at all (no `rdf:first`), so the caller can fall back to another shape.
fn read_collection<T: FromRdfTerm>(
  dataset: &Dataset,
  head: &NamedOrBlankNode,
) -> Result<Option<Vec<T>>, DeserializeError> {
  let rdf_first = NamedNode::from(rdf::FIRST);
  let rdf_rest = NamedNode::from(rdf::REST);

  let mut items = Vec::new();
  let mut current = head.clone();

  loop {
    let Some(first) = objects_for(dataset, &current, &rdf_first).next() else {
      return Ok(if items.is_empty() { None } else { Some(items) });
    };
    items.push(T::from_term(dataset, &first)?);

    match objects_for(dataset, &current, &rdf_rest).next() {
      Some(term) if term == Term::NamedNode(NamedNode::from(rdf::NIL)) => break,
      Some(term) => match as_subject(&term) {
        Some(next) => current = next,
        None => break,
      },
      None => break,
    }
  }

  Ok(Some(items))
}

/// Collects `rdf:_1`, `rdf:_2`, ... membership properties of `node` in numeric order, returning
/// `None` if there are none at all, so the caller can fall back to another shape.
fn read_container<T: FromRdfTerm>(
  dataset: &Dataset,
  node: &NamedOrBlankNode,
) -> Result<Option<Vec<T>>, DeserializeError> {
  const MEMBER_PREFIX: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#_";

  let mut indexed: Vec<(u64, Term)> = dataset
    .graph(GraphNameRef::DefaultGraph)
    .triples_for_subject(node)
    .filter_map(|triple| {
      let index = triple.predicate.as_str().strip_prefix(MEMBER_PREFIX)?;
      Some((index.parse::<u64>().ok()?, triple.object.into_owned()))
    })
    .collect();

  if indexed.is_empty() {
    return Ok(None);
  }

  indexed.sort_by_key(|(index, _)| *index);

  indexed
    .into_iter()
    .map(|(_, term)| T::from_term(dataset, &term))
    .collect::<Result<Vec<_>, _>>()
    .map(Some)
}

pub fn hash_set_field<T: FromRdfTerm + Eq + Hash>(
  dataset: &Dataset,
  subject: &NamedOrBlankNode,
  predicate: &NamedNode,
) -> Result<HashSet<T>, DeserializeError> {
  objects_for(dataset, subject, predicate)
    .map(|term| T::from_term(dataset, &term))
    .collect()
}

pub fn fresh_subject() -> NamedOrBlankNode {
  NamedOrBlankNode::BlankNode(BlankNode::default())
}
