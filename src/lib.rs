pub use linked_data_sparql_derive::Sparql;
use spargebra::Query;
use spargebra::algebra::{Expression, GraphPattern};
use spargebra::term::{
    NamedNode, NamedNodePattern, TermPattern, TriplePattern, Variable,
};
use sparopt::Optimizer;
use uuid::Uuid;

#[derive(Default)]
pub struct ConstructQuery {
    construct_template: Vec<TriplePattern>,
    where_pattern: GraphPattern,
}

pub trait SparqlQuery {
    fn sparql_query() -> String {
        Self::sparql_algebra().to_string()
    }

    fn as_sparql_query(&self) -> String {
        self.as_sparql_algebra().to_string()
    }

    fn sparql_algebra() -> Query;

    fn as_sparql_algebra(&self) -> Query {
        Self::sparql_algebra()
    }
}

pub trait ToConstructQuery {
    fn to_query_with_binding(binding_variable: Variable) -> ConstructQuery;

    fn to_query() -> ConstructQuery {
        Self::to_query_with_binding(generate_unique_variable())
    }
}

pub trait And {
    fn and(self, other: Self) -> Self;
}

pub trait Join {
    fn join(self, other: Self) -> Self;
}

pub trait Union {
    fn union(self, other: Self) -> Self;
}

impl ConstructQuery {
    pub fn new(
        subject: impl Into<TermPattern>,
        predicate: impl Into<NamedNodePattern>,
        object: impl Into<TermPattern>,
    ) -> ConstructQuery {
        let patterns = vec![TriplePattern {
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
        }];
        ConstructQuery {
            construct_template: patterns.clone(),
            where_pattern: GraphPattern::Bgp { patterns },
        }
    }

    pub fn new_with_binding<F>(
        subject: Variable,
        predicate: NamedNode,
        to_query_with_binding: F,
    ) -> Self
    where
        F: FnOnce(Variable) -> Self,
    {
        let object = generate_unique_variable();
        ConstructQuery::new(subject, predicate, object.clone())
            .join(to_query_with_binding(object))
    }

    pub fn union_with_binding<F>(
        self,
        subject: Variable,
        predicate: NamedNode,
        to_query_with_binding: F,
    ) -> Self
    where
        F: FnOnce(Variable) -> Self,
    {
        let object = generate_unique_variable();
        self.union(ConstructQuery::new(subject, predicate, object.clone()))
            .join(to_query_with_binding(object))
    }

    pub fn join_with_binding<F>(
        self,
        subject: Variable,
        predicate: NamedNode,
        to_query_with_binding: F,
    ) -> Self
    where
        F: FnOnce(Variable) -> Self,
    {
        let object = generate_unique_variable();
        self.join(ConstructQuery::new(subject, predicate, object.clone()))
            .join(to_query_with_binding(object))
    }

    pub fn join_with(
        self,
        subject: Variable,
        predicate: NamedNode,
        object: NamedNode,
    ) -> Self {
        self.join(ConstructQuery::new(subject, predicate, object))
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
        let pattern =
            (&Optimizer::optimize_graph_pattern((&value.where_pattern).into()))
                .into();
        Query::Construct {
            template: value.construct_template,
            dataset: None,
            pattern,
            base_iri: None,
        }
    }
}

impl Join for ConstructQuery {
    fn join(mut self, other: Self) -> Self {
        self.construct_template =
            self.construct_template.and(other.construct_template);
        self.where_pattern = self.where_pattern.join(other.where_pattern);
        self
    }
}

impl Union for ConstructQuery {
    fn union(mut self, other: Self) -> Self {
        self.construct_template =
            self.construct_template.and(other.construct_template);
        self.where_pattern = self.where_pattern.union(other.where_pattern);
        self
    }
}

impl<T> SparqlQuery for T
where
    T: ToConstructQuery,
{
    fn sparql_algebra() -> Query {
        Self::to_query().into()
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

impl Union for GraphPattern {
    fn union(self, other: Self) -> Self {
        GraphPattern::Union {
            left: Box::new(self),
            right: Box::new(other),
        }
    }
}

impl ToConstructQuery for Variable {
    fn to_query_with_binding(_: Variable) -> ConstructQuery {
        ConstructQuery::default()
    }
}

impl ToConstructQuery for String {
    fn to_query_with_binding(_: Variable) -> ConstructQuery {
        ConstructQuery::default()
    }
}

pub fn with_predicate<F>(
    predicate: NamedNode,
    to_query_with_binding: F,
) -> impl FnOnce(Variable) -> ConstructQuery
where
    F: FnOnce(Variable) -> ConstructQuery,
{
    |subject| {
        let object = generate_unique_variable();
        ConstructQuery::new(subject, predicate, object.clone())
            .join(to_query_with_binding(object))
    }
}

fn generate_unique_variable() -> Variable {
    let uuid = format!("{}", Uuid::new_v4().simple());
    let variable = uuid.to_string();
    Variable::new(variable).expect("Should not fail: UUID is a valid Variable")
}
