use crate::construct_query::left_join::LeftJoin;
use crate::to_construct_query::ToConstructQuery;
use and::And;
use join::Join;
use oxrdf::vocab::rdf;
use spargebra::Query;
use spargebra::algebra::{Expression, GraphPattern, PropertyPathExpression};
use spargebra::term::{NamedNode, NamedNodePattern, TermPattern, TriplePattern, Variable};
use union::Union;

pub mod and;
pub mod join;
pub mod left_join;
pub mod union;

#[derive(Default, Debug)]
pub struct ConstructQuery {
  construct_template: Vec<TriplePattern>,
  where_pattern: GraphPattern,
}

impl ConstructQuery {
  pub fn new(
    subject: impl Into<TermPattern>,
    predicate: impl Into<NamedNodePattern>,
    object: impl Into<TermPattern>,
  ) -> Self {
    let patterns = vec![TriplePattern {
      subject: subject.into(),
      predicate: predicate.into(),
      object: object.into(),
    }];

    Self {
      construct_template: patterns.clone(),
      where_pattern: GraphPattern::Bgp { patterns },
    }
  }

  pub fn new_with_binding<T: ToConstructQuery>(subject: Variable, predicate: NamedNode) -> Self {
    let object = Variable::new_unchecked(spargebra::term::BlankNode::default().into_string());

    Self::new(subject, predicate, object.clone()).join(T::to_query_with_binding(object))
  }

  pub fn union_with_binding<T: ToConstructQuery>(
    self,
    subject: Variable,
    predicate: NamedNode,
  ) -> (Self, Variable) {
    let object = Variable::new_unchecked(spargebra::term::BlankNode::default().into_string());

    let construct_query = self.union(
      Self::new(subject, predicate, object.clone()).join(T::to_query_with_binding(object.clone())),
    );

    (construct_query, object)
  }

  pub fn join_with_binding<T: ToConstructQuery>(
    self,
    subject: Variable,
    predicate: NamedNode,
  ) -> (Self, Variable) {
    let object = Variable::new_unchecked(spargebra::term::BlankNode::default().into_string());

    let construct_query = self.join(
      Self::new(subject, predicate, object.clone()).join(T::to_query_with_binding(object.clone())),
    );

    (construct_query, object)
  }

  pub fn left_join_with_binding<T: ToConstructQuery>(
    self,
    subject: Variable,
    predicate: NamedNode,
  ) -> (Self, Variable) {
    let object = Variable::new_unchecked(spargebra::term::BlankNode::default().into_string());

    let construct_query = self.left_join(
      Self::new(subject, predicate, object.clone()).join(T::to_query_with_binding(object.clone())),
    );

    (construct_query, object)
  }

  pub fn join_with(self, subject: Variable, predicate: NamedNode, object: NamedNode) -> Self {
    self.join(Self::new(subject, predicate, object))
  }

  pub fn filter_variable(self, variable: Variable, id: NamedNode) -> Self {
    let expr = Expression::Equal(
      Box::new(Expression::Variable(variable)),
      Box::new(Expression::NamedNode(id)),
    );
    Self {
      construct_template: self.construct_template,
      where_pattern: GraphPattern::Filter {
        expr,
        inner: Box::new(self.where_pattern),
      },
    }
  }
}

impl From<ConstructQuery> for Query {
  fn from(value: ConstructQuery) -> Self {
    let mut template = value.construct_template.clone();
    template.sort_by_key(|a| a.subject.to_string());

    Query::Construct {
      template,
      dataset: None,
      pattern: value.where_pattern,
      base_iri: None,
    }
  }
}

impl ToConstructQuery for Variable {
  fn to_query_with_binding(_: Variable) -> ConstructQuery {
    ConstructQuery::default()
  }
}

macro_rules! to_construct_query_datatypes {
  ($($t:ty),*) => {
    $(
      impl ToConstructQuery for $t {
        fn to_query_with_binding(_: Variable) -> ConstructQuery {
          ConstructQuery::default()
        }
      }
    )*
  };
}

to_construct_query_datatypes!(
  bool,
  u8,
  u16,
  u32,
  u64,
  i8,
  i16,
  i32,
  i64,
  f32,
  f64,
  String,
  xsd_types::DateTime
);

impl<T: ToConstructQuery> ToConstructQuery for Option<T> {
  fn to_query_with_binding(variable: Variable) -> ConstructQuery {
    T::to_query_with_binding(variable)
  }
}

impl<T: ToConstructQuery> ToConstructQuery for Vec<T> {
  /// Reconstructs an RDF Collection: the `rdf:first`/`rdf:rest` linked list rooted at
  /// `binding_variable` and terminated by `rdf:nil`, including each item's own structure.
  fn to_query_with_binding(binding_variable: Variable) -> ConstructQuery {
    let node = Variable::new_unchecked(spargebra::term::BlankNode::default().into_string());
    let item = Variable::new_unchecked(spargebra::term::BlankNode::default().into_string());
    let rest = Variable::new_unchecked(spargebra::term::BlankNode::default().into_string());

    let rdf_first = NamedNode::from(rdf::FIRST);
    let rdf_rest = NamedNode::from(rdf::REST);

    // Every list node, however far down the chain, is reachable from the head by zero or more
    // `rdf:rest` hops; this doesn't itself construct anything, it only binds `node`.
    let reachable = ConstructQuery {
      construct_template: Vec::new(),
      where_pattern: GraphPattern::Path {
        subject: binding_variable.into(),
        path: PropertyPathExpression::ZeroOrMore(Box::new(PropertyPathExpression::NamedNode(
          rdf_rest.clone(),
        ))),
        object: node.clone().into(),
      },
    };

    let walk = reachable
      .join(ConstructQuery::new(node.clone(), rdf_first, item.clone()))
      .join(ConstructQuery::new(node, rdf_rest, rest))
      .join(T::to_query_with_binding(item));

    // An empty list is just `binding_variable` bound to `rdf:nil`, with no `rdf:first`/`rdf:rest`
    // node to walk; left-joining against the always-matching identity pattern keeps that case
    // from vanishing instead of failing the whole query.
    ConstructQuery::default().left_join(walk)
  }
}

impl<T: ToConstructQuery> ToConstructQuery for std::collections::HashSet<T> {
  fn to_query_with_binding(variable: Variable) -> ConstructQuery {
    T::to_query_with_binding(variable)
  }
}

impl Join for ConstructQuery {
  fn join(mut self, other: Self) -> Self {
    self.construct_template.extend(other.construct_template);
    self.where_pattern = self.where_pattern.join(other.where_pattern);
    self
  }
}

impl LeftJoin for ConstructQuery {
  fn left_join(mut self, other: Self) -> Self {
    self.construct_template.extend(other.construct_template);
    self.where_pattern = self.where_pattern.left_join(other.where_pattern);
    self
  }
}

impl Union for ConstructQuery {
  fn union(mut self, other: Self) -> Self {
    self.construct_template.extend(other.construct_template);
    self.where_pattern = self.where_pattern.union(other.where_pattern);
    self
  }
}

impl And for Vec<TriplePattern> {
  fn and(mut self, other: Self) -> Self {
    self.extend(other);
    self
  }
}

impl Join for GraphPattern {
  fn join(self, other: Self) -> Self {
    GraphPattern::Join {
      left: Box::new(self),
      right: Box::new(other),
    }
  }
}

impl LeftJoin for GraphPattern {
  fn left_join(self, other: Self) -> Self {
    GraphPattern::LeftJoin {
      left: Box::new(self),
      right: Box::new(other),
      expression: None,
    }
  }
}

impl Union for GraphPattern {
  fn union(self, other: Self) -> Self {
    GraphPattern::Union {
      left: Box::new(self),
      right: Box::new(other),
    }
  }
}
