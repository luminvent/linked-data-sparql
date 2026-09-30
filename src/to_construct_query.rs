use crate::ConstructQuery;
use crate::filter::ToSparqlFilter;
use spargebra::term::Variable;

pub trait ToConstructQuery {
  fn to_query_with_binding(binding_variable: Variable) -> ConstructQuery;

  fn to_query() -> ConstructQuery {
    let object = spargebra::term::BlankNode::default();

    Self::to_query_with_binding(Variable::new_unchecked(object.into_string()))
  }

  /// Same as [`ToConstructQuery::to_query`], keeping only the subjects matching `filter`.
  fn to_filtered_query(filter: &impl ToSparqlFilter) -> ConstructQuery {
    let binding_variable =
      Variable::new_unchecked(spargebra::term::BlankNode::default().into_string());

    let construct_query = Self::to_query_with_binding(binding_variable.clone());

    match filter.to_expression(&binding_variable) {
      Some(expression) => construct_query.filter(expression),
      None => construct_query,
    }
  }
}
