//! Runtime (de)serialization of Rust values to and from RDF quads, built directly
//! on `oxrdf` types.

use oxrdf::vocab::xsd;
use oxrdf::{BlankNode, Dataset, GraphNameRef, Literal, NamedNode, NamedOrBlankNode, Quad, Term};
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

pub fn vec_field<T: FromRdfTerm>(
  dataset: &Dataset,
  subject: &NamedOrBlankNode,
  predicate: &NamedNode,
) -> Result<Vec<T>, DeserializeError> {
  objects_for(dataset, subject, predicate)
    .map(|term| T::from_term(dataset, &term))
    .collect()
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
