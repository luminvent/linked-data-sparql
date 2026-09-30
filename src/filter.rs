use crate::rdf_serde::ToRdfTerm;
use oxrdf::{Literal, Term};
use spargebra::algebra::{Expression, Function, GraphPattern};
use spargebra::term::{NamedNode, TriplePattern, Variable};

/// A function applied to the object of a filtered predicate before it is compared.
///
/// String functions work on `STR(...)` of their input, so they also apply to IRIs and drop the
/// language tag or datatype of a literal before the comparison. Date and time functions expect an
/// `xsd:dateTime` (or `xsd:date` for `YEAR`, `MONTH`, `DAY`) and are applied to the input as is.
#[derive(Debug, Clone)]
pub enum ValueFunction {
  /// `YEAR(input)`
  Year,
  /// `MONTH(input)`
  Month,
  /// `DAY(input)`
  Day,
  /// `HOURS(input)`
  Hours,
  /// `MINUTES(input)`
  Minutes,
  /// `SECONDS(input)`
  Seconds,
  /// `TIMEZONE(input)`, an `xsd:dayTimeDuration`
  Timezone,
  /// `TZ(input)`, the timezone as a simple literal (`"Z"`, `"-05:00"`, or `""` when absent)
  Tz,
  /// `STRLEN(STR(input))`
  StrLen,
  /// `UCASE(STR(input))`
  UCase,
  /// `LCASE(STR(input))`
  LCase,
  /// `STRBEFORE(STR(input), <separator>)`
  StrBefore(Expression),
  /// `STRAFTER(STR(input), <separator>)`
  StrAfter(Expression),
}

impl ValueFunction {
  fn apply(&self, input: Expression) -> Expression {
    let call = |function: Function, mut arguments: Vec<Expression>| {
      arguments.insert(0, str(input.clone()));
      Expression::FunctionCall(function, arguments)
    };

    match self {
      ValueFunction::Year => Expression::FunctionCall(Function::Year, vec![input]),
      ValueFunction::Month => Expression::FunctionCall(Function::Month, vec![input]),
      ValueFunction::Day => Expression::FunctionCall(Function::Day, vec![input]),
      ValueFunction::Hours => Expression::FunctionCall(Function::Hours, vec![input]),
      ValueFunction::Minutes => Expression::FunctionCall(Function::Minutes, vec![input]),
      ValueFunction::Seconds => Expression::FunctionCall(Function::Seconds, vec![input]),
      ValueFunction::Timezone => Expression::FunctionCall(Function::Timezone, vec![input]),
      ValueFunction::Tz => Expression::FunctionCall(Function::Tz, vec![input]),
      ValueFunction::StrLen => call(Function::StrLen, vec![]),
      ValueFunction::UCase => call(Function::UCase, vec![]),
      ValueFunction::LCase => call(Function::LCase, vec![]),
      ValueFunction::StrBefore(separator) => call(Function::StrBefore, vec![separator.clone()]),
      ValueFunction::StrAfter(separator) => call(Function::StrAfter, vec![separator.clone()]),
    }
  }
}

/// How the (possibly transformed) object is compared with the filter value.
#[derive(Debug, Clone)]
pub enum Operator {
  /// `left = <value>`
  Equal,
  /// `left != <value>`
  NotEqual,
  /// `left < <value>`
  Less,
  /// `left > <value>`
  Greater,
  /// `left <= <value>`
  LessOrEqual,
  /// `left >= <value>`
  GreaterOrEqual,
  /// `STRSTARTS(STR(left), <value>)`
  StrStarts,
  /// `STRENDS(STR(left), <value>)`
  StrEnds,
  /// `CONTAINS(STR(left), <value>)`
  Contains,
  /// `REGEX(STR(left), <value>)`, or `REGEX(STR(left), <value>, "i")` when `case_insensitive`
  /// is set.
  Regex { case_insensitive: bool },
  /// `LANGMATCHES(LANG(left), <value>)`
  LangMatches,
}

impl Operator {
  fn apply(&self, left: Expression, value: Expression) -> Expression {
    let compare = |operator: fn(Box<Expression>, Box<Expression>) -> Expression| {
      operator(Box::new(left.clone()), Box::new(value.clone()))
    };
    let call = |function: Function| {
      Expression::FunctionCall(function, vec![str(left.clone()), value.clone()])
    };

    match self {
      Operator::Equal => compare(Expression::Equal),
      Operator::NotEqual => Expression::Not(Box::new(compare(Expression::Equal))),
      Operator::Less => compare(Expression::Less),
      Operator::Greater => compare(Expression::Greater),
      Operator::LessOrEqual => compare(Expression::LessOrEqual),
      Operator::GreaterOrEqual => compare(Expression::GreaterOrEqual),
      Operator::StrStarts => call(Function::StrStarts),
      Operator::StrEnds => call(Function::StrEnds),
      Operator::Contains => call(Function::Contains),
      Operator::Regex { case_insensitive } => {
        let mut arguments = vec![str(left.clone()), value.clone()];
        if *case_insensitive {
          arguments.push(Expression::Literal(Literal::new_simple_literal("i")));
        }
        Expression::FunctionCall(Function::Regex, arguments)
      }
      Operator::LangMatches => Expression::FunctionCall(
        Function::LangMatches,
        vec![
          Expression::FunctionCall(Function::Lang, vec![left.clone()]),
          value.clone(),
        ],
      ),
    }
  }
}

/// `operator(functions(?value), value)`, the functions being applied in order.
#[derive(Debug, Clone)]
pub struct Condition {
  pub functions: Vec<ValueFunction>,
  pub operator: Operator,
  pub value: Expression,
}

impl Condition {
  pub fn new(operator: Operator, value: Expression) -> Self {
    Self {
      functions: Vec::new(),
      operator,
      value,
    }
  }

  /// Applies `function` to the object, after the functions already added.
  pub fn with_function(mut self, function: ValueFunction) -> Self {
    self.functions.push(function);
    self
  }

  fn to_expression(&self, object: Expression) -> Expression {
    let left = self
      .functions
      .iter()
      .fold(object, |input, function| function.apply(input));

    self.operator.apply(left, self.value.clone())
  }
}

/// What a [`PredicateFilter`] checks on the objects of its predicate.
///
/// Every mode is evaluated inside an `EXISTS` (or `NOT EXISTS`) sub-pattern, so a filter never
/// adds solutions to the query nor removes triples from the constructed graph: a multi-valued
/// predicate matches as soon as one of its values satisfies the condition.
#[derive(Debug, Clone)]
pub enum FilterMode {
  /// `FILTER EXISTS { ?subject <predicate> ?value FILTER(<condition>) }`
  Condition(Condition),
  /// `FILTER EXISTS { ?subject <predicate> ?value }`
  Exists,
  /// `FILTER NOT EXISTS { ?subject <predicate> ?value }`
  NotExists,
}

impl FilterMode {
  /// `?value = <value>`
  pub fn equal(value: Expression) -> Self {
    FilterMode::Condition(Condition::new(Operator::Equal, value))
  }
}

fn str(expression: Expression) -> Expression {
  Expression::FunctionCall(Function::Str, vec![expression])
}

/// A condition on the objects of one predicate of the filtered subject.
#[derive(Debug, Clone)]
pub struct PredicateFilter {
  pub predicate: NamedNode,
  pub mode: FilterMode,
  /// When set, a subject without any value for `predicate` also passes the filter, like a
  /// condition on an `OPTIONAL` binding: `!BOUND(?value) || condition`.
  pub optional: bool,
}

impl PredicateFilter {
  pub fn new(predicate: NamedNode, mode: FilterMode) -> Self {
    Self {
      predicate,
      mode,
      optional: false,
    }
  }

  /// Lets subjects without the predicate pass the filter. See [`PredicateFilter::optional`].
  pub fn optional(mut self) -> Self {
    self.optional = true;
    self
  }

  /// Builds the `EXISTS`/`NOT EXISTS` expression constraining `subject`.
  pub fn to_expression(&self, subject: &Variable) -> Expression {
    let value = Variable::new_unchecked(spargebra::term::BlankNode::default().into_string());

    let triple = GraphPattern::Bgp {
      patterns: vec![TriplePattern {
        subject: subject.clone().into(),
        predicate: self.predicate.clone().into(),
        object: value.clone().into(),
      }],
    };

    let condition = match &self.mode {
      FilterMode::Condition(condition) => condition.to_expression(Expression::Variable(value)),
      FilterMode::Exists => return Expression::Exists(Box::new(triple)),
      FilterMode::NotExists => {
        return Expression::Not(Box::new(Expression::Exists(Box::new(triple))));
      }
    };

    let matches = Expression::Exists(Box::new(GraphPattern::Filter {
      expr: condition,
      inner: Box::new(triple.clone()),
    }));

    if self.optional {
      Expression::Or(
        Box::new(Expression::Not(Box::new(Expression::Exists(Box::new(
          triple,
        ))))),
        Box::new(matches),
      )
    } else {
      matches
    }
  }
}

/// A set of conditions applied to the subject of a [`ToConstructQuery`](crate::ToConstructQuery)
/// query, usually implemented with `#[derive(SparqlFilter)]`.
pub trait ToSparqlFilter {
  fn predicate_filters(&self) -> Vec<PredicateFilter>;

  /// Conjunction of every predicate filter, or `None` when there is nothing to filter on.
  fn to_expression(&self, subject: &Variable) -> Option<Expression> {
    self
      .predicate_filters()
      .iter()
      .map(|predicate_filter| predicate_filter.to_expression(subject))
      .reduce(|left, right| Expression::And(Box::new(left), Box::new(right)))
  }
}

/// Converts a Rust value into the SPARQL expression it is compared with, using the same RDF
/// encoding as [`ToRdfTerm`] so that a filter value matches what was stored.
pub fn filter_value<T: ToRdfTerm + ?Sized>(value: &T) -> Expression {
  match value.to_term(&mut Vec::new()) {
    Term::NamedNode(named_node) => Expression::NamedNode(named_node),
    Term::Literal(literal) => Expression::Literal(literal),
    term => panic!("filter value must be an IRI or a literal, got {term}"),
  }
}

/// Same as [`filter_value`], with the lexical form of `value` typed as `datatype`, e.g. a `String`
/// holding `"2015-01-01T00:00:00Z"` becomes `"2015-01-01T00:00:00Z"^^xsd:dateTime`.
pub fn typed_filter_value<T: ToRdfTerm + ?Sized>(value: &T, datatype: NamedNode) -> Expression {
  let lexical_form = match value.to_term(&mut Vec::new()) {
    Term::NamedNode(named_node) => named_node.into_string(),
    Term::Literal(literal) => literal.value().to_owned(),
    term => panic!("filter value must be an IRI or a literal, got {term}"),
  };

  Expression::Literal(Literal::new_typed_literal(lexical_form, datatype))
}

/// `NOW()`: the query evaluation time, to compare an `xsd:dateTime` object with.
pub fn now() -> Expression {
  Expression::FunctionCall(Function::Now, Vec::new())
}
