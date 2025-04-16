use crate::ConstructQuery;
use spargebra::term::Variable;

pub trait ToConstructQuery {
  fn to_query_with_binding(binding_variable: Variable) -> ConstructQuery;

  fn to_query() -> ConstructQuery {
    let object = spargebra::term::BlankNode::default();

    Self::to_query_with_binding(Variable::new_unchecked(object.into_string()))
  }
}
