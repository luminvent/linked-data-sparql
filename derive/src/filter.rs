use crate::{Sparql, is_option};
use linked_data_core::{RdfField, RdfType};
use proc_macro_error::abort;
use proc_macro2::TokenStream;
use std::str::FromStr;
use strum::{EnumString, VariantNames};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, LitStr, Token, Type};

/// Function applied to the object before the comparison.
enum Function {
  Year,
  Month,
  Day,
  Hours,
  Minutes,
  Seconds,
  Timezone,
  Tz,
  StrLen,
  UCase,
  LCase,
  StrBefore(LitStr),
  StrAfter(LitStr),
}

/// Keyword of a [`Function`]; `str_before`/`str_after` are followed by their `("separator")`.
#[derive(EnumString, VariantNames)]
#[strum(serialize_all = "snake_case")]
enum FunctionName {
  Year,
  Month,
  Day,
  Hours,
  Minutes,
  Seconds,
  Timezone,
  Tz,
  StrLength,
  Uppercase,
  Lowercase,
  StrBefore,
  StrAfter,
}

/// Comparison between the (possibly transformed) object and the field value.
#[derive(PartialEq, EnumString, VariantNames)]
enum Operator {
  #[strum(serialize = "eq")]
  Equal,
  #[strum(serialize = "ne")]
  NotEqual,
  #[strum(serialize = "lt")]
  Less,
  #[strum(serialize = "gt")]
  Greater,
  #[strum(serialize = "le")]
  LessOrEqual,
  #[strum(serialize = "ge")]
  GreaterOrEqual,
  #[strum(serialize = "str_starts")]
  StrStarts,
  #[strum(serialize = "str_ends")]
  StrEnds,
  #[strum(serialize = "contains")]
  Contains,
  #[strum(serialize = "regex")]
  Regex,
  #[strum(serialize = "lang")]
  LangMatches,
}

/// Value-less check on the predicate, driven by a `bool` field.
#[derive(EnumString, VariantNames)]
#[strum(serialize_all = "snake_case")]
enum Switch {
  Exists,
  NotExists,
}

#[derive(EnumString, VariantNames)]
#[strum(serialize_all = "snake_case")]
enum Modifier {
  CaseInsensitive,
  Optional,
}

/// Value computed by the query itself, compared with instead of the field value: the field is
/// then a `bool` switch turning the filter on.
#[derive(EnumString, VariantNames)]
#[strum(serialize_all = "snake_case")]
enum DynamicValue {
  /// `NOW()`
  Now,
}

/// Typing of the field value as an RDF literal: `<key> = "<value>"`.
#[derive(EnumString, VariantNames)]
#[strum(serialize_all = "snake_case")]
enum TypedLiteral {
  /// Datatype IRI (or prefixed name) of the field value, turning e.g. a `String` holding
  /// `"2015-01-01T00:00:00Z"` into `"2015-01-01T00:00:00Z"^^xsd:dateTime`.
  Datatype,
}

/// One comma-separated item of `#[filter(...)]`.
enum FilterItem {
  Function(Function),
  Operator(Operator),
  Switch(Switch),
  Modifier(Modifier),
  DynamicValue(DynamicValue),
  TypedLiteral(TypedLiteral, LitStr),
}

impl Parse for FilterItem {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    // Two-character operators first, so `<=` isn't read as `<` followed by `=`.
    macro_rules! operator {
      ($($token:tt => $operator:ident),*) => {
        $(
          if input.peek(Token![$token]) {
            input.parse::<Token![$token]>()?;
            return Ok(FilterItem::Operator(Operator::$operator));
          }
        )*
      };
    }
    operator!(
      <= => LessOrEqual,
      >= => GreaterOrEqual,
      != => NotEqual,
      == => Equal,
      = => Equal,
      < => Less,
      > => Greater
    );

    let ident: syn::Ident = input.parse()?;
    let keyword = ident.to_string();

    if let Ok(operator) = Operator::from_str(&keyword) {
      return Ok(FilterItem::Operator(operator));
    }
    if let Ok(switch) = Switch::from_str(&keyword) {
      return Ok(FilterItem::Switch(switch));
    }
    if let Ok(modifier) = Modifier::from_str(&keyword) {
      return Ok(FilterItem::Modifier(modifier));
    }
    if let Ok(dynamic_value) = DynamicValue::from_str(&keyword) {
      return Ok(FilterItem::DynamicValue(dynamic_value));
    }
    if let Ok(typed_literal) = TypedLiteral::from_str(&keyword) {
      input.parse::<Token![=]>()?;
      return Ok(FilterItem::TypedLiteral(typed_literal, input.parse()?));
    }

    let separator = || -> syn::Result<LitStr> {
      if !input.peek(syn::token::Paren) {
        return Err(syn::Error::new(
          ident.span(),
          format!("`{ident}` takes a separator, e.g. `{ident}(\"@\")`"),
        ));
      }
      let content;
      syn::parenthesized!(content in input);
      content.parse()
    };

    let function = match FunctionName::from_str(&keyword) {
      Ok(FunctionName::Year) => Function::Year,
      Ok(FunctionName::Month) => Function::Month,
      Ok(FunctionName::Day) => Function::Day,
      Ok(FunctionName::Hours) => Function::Hours,
      Ok(FunctionName::Minutes) => Function::Minutes,
      Ok(FunctionName::Seconds) => Function::Seconds,
      Ok(FunctionName::Timezone) => Function::Timezone,
      Ok(FunctionName::Tz) => Function::Tz,
      Ok(FunctionName::StrLength) => Function::StrLen,
      Ok(FunctionName::Uppercase) => Function::UCase,
      Ok(FunctionName::Lowercase) => Function::LCase,
      Ok(FunctionName::StrBefore) => Function::StrBefore(separator()?),
      Ok(FunctionName::StrAfter) => Function::StrAfter(separator()?),
      Err(_) => {
        let list = |names: &[&str]| {
          names
            .iter()
            .map(|name| format!("`{name}`"))
            .collect::<Vec<_>>()
            .join(", ")
        };
        return Err(syn::Error::new(
          ident.span(),
          format!(
            "unknown filter item `{ident}`, expected an operator ({}, `=`, `!=`, `<`, `>`, `<=`, \
            `>=`), a function ({}), a switch ({}), a modifier ({}), a dynamic value ({}) or a \
            typed literal ({})",
            list(Operator::VARIANTS),
            list(FunctionName::VARIANTS),
            list(Switch::VARIANTS),
            list(Modifier::VARIANTS),
            list(DynamicValue::VARIANTS),
            TypedLiteral::VARIANTS
              .iter()
              .map(|name| format!("`{name} = \"...\"`"))
              .collect::<Vec<_>>()
              .join(", "),
          ),
        ));
      }
    };

    Ok(FilterItem::Function(function))
  }
}

/// Parsed `#[filter(...)]` field attribute: either a switch, or functions applied in order to the
/// object followed by an operator (`=` when omitted).
enum FilterAttribute {
  Switch(Switch),
  Condition {
    functions: Vec<Function>,
    operator: Operator,
    case_insensitive: bool,
    optional: bool,
    datatype: Option<LitStr>,
    dynamic_value: Option<DynamicValue>,
  },
}

impl FilterAttribute {
  fn from_field(field: &syn::Field) -> Self {
    let mut attributes = field
      .attrs
      .iter()
      .filter(|attr| attr.path().is_ident("filter"));

    let Some(attribute) = attributes.next() else {
      return FilterAttribute::Condition {
        functions: Vec::new(),
        operator: Operator::Equal,
        case_insensitive: false,
        optional: false,
        datatype: None,
        dynamic_value: None,
      };
    };

    if let Some(duplicate) = attributes.next() {
      abort!(
        duplicate.span(),
        "`filter` attribute is only allowed once per field"
      );
    }

    let items = attribute
      .parse_args_with(Punctuated::<FilterItem, Token![,]>::parse_terminated)
      .unwrap_or_else(|error| abort!(error.span(), "{}", error));

    let span = attribute.span();
    let mut functions = Vec::new();
    let mut operator = None;
    let mut switch = None;
    let mut case_insensitive = false;
    let mut optional = false;
    let mut datatype = None;
    let mut dynamic_value = None;

    for item in items {
      match item {
        FilterItem::Function(function) => functions.push(function),
        FilterItem::Operator(item) => {
          if operator.replace(item).is_some() {
            abort!(span, "only one operator is allowed per field");
          }
        }
        FilterItem::Switch(item) => {
          if switch.replace(item).is_some() {
            abort!(
              span,
              "only one of `exists`, `not_exists` is allowed per field"
            );
          }
        }
        FilterItem::Modifier(Modifier::CaseInsensitive) => case_insensitive = true,
        FilterItem::Modifier(Modifier::Optional) => optional = true,
        FilterItem::DynamicValue(item) => {
          if dynamic_value.replace(item).is_some() {
            abort!(span, "only one dynamic value is allowed per field");
          }
        }
        FilterItem::TypedLiteral(TypedLiteral::Datatype, value) => {
          if datatype.replace(value).is_some() {
            abort!(span, "only one `datatype` is allowed per field");
          }
        }
      }
    }

    if let Some(switch) = switch {
      if operator.is_some()
        || !functions.is_empty()
        || case_insensitive
        || optional
        || datatype.is_some()
        || dynamic_value.is_some()
      {
        abort!(
          span,
          "`exists` and `not_exists` can't be combined with an operator, a function or a modifier"
        );
      }
      return FilterAttribute::Switch(switch);
    }

    let operator = operator.unwrap_or(Operator::Equal);

    if case_insensitive && operator != Operator::Regex {
      abort!(
        span,
        "`case_insensitive` is only supported by the `regex` operator"
      );
    }
    if operator == Operator::LangMatches && !functions.is_empty() {
      abort!(span, "`lang` can't be combined with a function");
    }
    let is_comparison = matches!(
      operator,
      Operator::Equal
        | Operator::NotEqual
        | Operator::Less
        | Operator::Greater
        | Operator::LessOrEqual
        | Operator::GreaterOrEqual
    );
    if datatype.is_some() && !is_comparison {
      abort!(
        span,
        "`datatype` is only supported by the `=`, `!=`, `<`, `>`, `<=` and `>=` operators"
      );
    }
    if dynamic_value.is_some() && !is_comparison {
      abort!(
        span,
        "`now` is only supported by the `=`, `!=`, `<`, `>`, `<=` and `>=` operators"
      );
    }
    if dynamic_value.is_some() && datatype.is_some() {
      abort!(span, "`now` can't be combined with `datatype`");
    }

    FilterAttribute::Condition {
      functions,
      operator,
      case_insensitive,
      optional,
      datatype,
      dynamic_value,
    }
  }
}

pub fn derive_sparql_filter(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let raw_input = syn::parse_macro_input!(item as DeriveInput);
  let ident = raw_input.ident.clone();

  let Data::Struct(data_struct) = &raw_input.data else {
    abort!(
      ident.span(),
      "`SparqlFilter` can only be derived on structs"
    );
  };

  let Fields::Named(named_fields) = &data_struct.fields else {
    abort!(
      ident.span(),
      "`SparqlFilter` requires a struct with named fields"
    );
  };
  let syn_fields: Vec<syn::Field> = named_fields.named.iter().cloned().collect();

  let RdfType::Struct(rdf_struct) = RdfType::<Sparql>::from_derive(raw_input) else {
    unreachable!("a struct is always parsed as `RdfType::Struct`")
  };

  let type_filter = rdf_struct.type_iri().map(|iri| {
    let iri = iri.as_str();
    quote::quote! {
      predicate_filters.push(::linked_data_sparql::PredicateFilter::new(
        ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
        ::linked_data_sparql::FilterMode::equal(
          ::linked_data_sparql::reexport::spargebra::algebra::Expression::NamedNode(
            ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked(#iri),
          ),
        ),
      ));
    }
  });

  let field_filters = rdf_struct
    .fields
    .iter()
    .zip(&syn_fields)
    .map(|(rdf_field, syn_field)| {
      field_filter_tokens(rdf_field, syn_field, &|datatype| {
        expand_datatype(
          rdf_struct.prefix_mappings().expand(datatype.value()),
          datatype,
        )
      })
    });

  quote::quote! {
    impl ::linked_data_sparql::ToSparqlFilter for #ident {
      fn predicate_filters(&self) -> ::std::vec::Vec<::linked_data_sparql::PredicateFilter> {
        let mut predicate_filters = ::std::vec::Vec::new();
        #type_filter
        #(#field_filters)*
        predicate_filters
      }
    }
  }
  .into()
}

/// Expands a `datatype = "..."` value with the struct prefixes, falling back to the XSD namespace
/// for an undeclared `xsd:` prefix; `linked-data-core` would otherwise keep `xsd:dateTime` as is,
/// `xsd` being a valid IRI scheme.
fn expand_datatype<I: ToString, E: std::fmt::Display>(
  expanded: Result<I, E>,
  datatype: &LitStr,
) -> String {
  let value = datatype.value();
  match expanded {
    Ok(iri) => {
      let iri = iri.to_string();
      match iri.strip_prefix("xsd:") {
        Some(name) if iri == value => format!("http://www.w3.org/2001/XMLSchema#{name}"),
        _ => iri,
      }
    }
    Err(error) => abort!(datatype.span(), "invalid datatype `{}`: {}", value, error),
  }
}

fn field_filter_tokens(
  rdf_field: &RdfField<Sparql>,
  syn_field: &syn::Field,
  expand_datatype: &dyn Fn(&LitStr) -> String,
) -> TokenStream {
  let field_ident = syn_field.ident.as_ref().expect("named field");
  let span = syn_field.span();

  if rdf_field.is_ignored() {
    return TokenStream::new();
  }

  if rdf_field.is_flattened() {
    return quote::quote! {
      predicate_filters.extend(::linked_data_sparql::ToSparqlFilter::predicate_filters(&self.#field_ident));
    };
  }

  if rdf_field.is_id() || rdf_field.is_graph() {
    abort!(
      span,
      "`id` and `graph` are not supported on a `SparqlFilter` field"
    );
  }

  let Some(predicate) = rdf_field.predicate() else {
    abort!(
      span,
      "a `SparqlFilter` field needs a predicate: `#[ld(\"prefix:predicate\")]`"
    );
  };
  let predicate_iri = predicate.as_str();

  let push = |mode: TokenStream, optional: bool| {
    let optional = optional.then(|| quote::quote! { .optional() });
    quote::quote! {
      predicate_filters.push(
        ::linked_data_sparql::PredicateFilter::new(
          ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked(#predicate_iri),
          #mode,
        )
        #optional
      );
    }
  };

  let (functions, operator, case_insensitive, optional, datatype, dynamic_value) =
    match FilterAttribute::from_field(syn_field) {
      // `exists`/`not_exists` have no value to compare with: the field is a switch turning the
      // filter on.
      FilterAttribute::Switch(switch) => {
        if !is_bool(&syn_field.ty) {
          abort!(
            syn_field.ty.span(),
            "an `exists` or `not_exists` filter field must be a `bool`"
          );
        }
        let variant = match switch {
          Switch::Exists => quote::quote! { Exists },
          Switch::NotExists => quote::quote! { NotExists },
        };
        let push = push(
          quote::quote! { ::linked_data_sparql::FilterMode::#variant },
          false,
        );
        return quote::quote! {
          if self.#field_ident {
            #push
          }
        };
      }
      FilterAttribute::Condition {
        functions,
        operator,
        case_insensitive,
        optional,
        datatype,
        dynamic_value,
      } => (
        functions,
        operator,
        case_insensitive,
        optional,
        datatype,
        dynamic_value,
      ),
    };

  let functions = functions.iter().map(|function| {
    let separator = |separator: &LitStr| {
      quote::quote! {
        (::linked_data_sparql::reexport::spargebra::algebra::Expression::Literal(
          ::linked_data_sparql::reexport::spargebra::term::Literal::new_simple_literal(#separator),
        ))
      }
    };
    let (variant, argument) = match function {
      Function::Year => (quote::quote! { Year }, None),
      Function::Month => (quote::quote! { Month }, None),
      Function::Day => (quote::quote! { Day }, None),
      Function::Hours => (quote::quote! { Hours }, None),
      Function::Minutes => (quote::quote! { Minutes }, None),
      Function::Seconds => (quote::quote! { Seconds }, None),
      Function::Timezone => (quote::quote! { Timezone }, None),
      Function::Tz => (quote::quote! { Tz }, None),
      Function::StrLen => (quote::quote! { StrLen }, None),
      Function::UCase => (quote::quote! { UCase }, None),
      Function::LCase => (quote::quote! { LCase }, None),
      Function::StrBefore(lit) => (quote::quote! { StrBefore }, Some(separator(lit))),
      Function::StrAfter(lit) => (quote::quote! { StrAfter }, Some(separator(lit))),
    };
    quote::quote! { .with_function(::linked_data_sparql::ValueFunction::#variant #argument) }
  });

  let operator = match operator {
    Operator::Equal => quote::quote! { Equal },
    Operator::NotEqual => quote::quote! { NotEqual },
    Operator::Less => quote::quote! { Less },
    Operator::Greater => quote::quote! { Greater },
    Operator::LessOrEqual => quote::quote! { LessOrEqual },
    Operator::GreaterOrEqual => quote::quote! { GreaterOrEqual },
    Operator::StrStarts => quote::quote! { StrStarts },
    Operator::StrEnds => quote::quote! { StrEnds },
    Operator::Contains => quote::quote! { Contains },
    Operator::Regex => quote::quote! { Regex { case_insensitive: #case_insensitive } },
    Operator::LangMatches => quote::quote! { LangMatches },
  };

  let dynamic_value_is_set = dynamic_value.is_some();
  let value = match (dynamic_value, datatype) {
    (Some(DynamicValue::Now), _) => quote::quote! { ::linked_data_sparql::filter_support::now() },
    (None, Some(datatype)) => {
      let datatype_iri = expand_datatype(&datatype);
      quote::quote! {
        ::linked_data_sparql::filter_support::typed_filter_value(
          value,
          ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked(#datatype_iri),
        )
      }
    }
    (None, None) => quote::quote! { ::linked_data_sparql::filter_support::filter_value(value) },
  };

  let push = push(
    quote::quote! {
      ::linked_data_sparql::FilterMode::Condition(
        ::linked_data_sparql::Condition::new(
          ::linked_data_sparql::Operator::#operator,
          #value,
        )
        #(#functions)*
      )
    },
    optional,
  );

  // With a dynamic value there is no field value to compare with: the field is a switch.
  if dynamic_value_is_set {
    if !is_bool(&syn_field.ty) {
      abort!(syn_field.ty.span(), "a `now` filter field must be a `bool`");
    }
    return quote::quote! {
      if self.#field_ident {
        #push
      }
    };
  }

  // An `Option` field only filters when it holds a value, so a single filter struct can express
  // every combination of optional criteria (e.g. built from query-string parameters).
  if is_option(&syn_field.ty) {
    quote::quote! {
      if let ::std::option::Option::Some(value) = &self.#field_ident {
        #push
      }
    }
  } else {
    quote::quote! {
      {
        let value = &self.#field_ident;
        #push
      }
    }
  }
}

fn is_bool(ty: &Type) -> bool {
  matches!(ty, Type::Path(type_path) if type_path.path.is_ident("bool"))
}
