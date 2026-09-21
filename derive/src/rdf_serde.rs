use linked_data_core::{PredicatePath, RdfEnum, RdfField, RdfStruct, RdfType, RdfVariant};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{DeriveInput, Fields, GenericArgument, Index, PathArguments, Type};

use crate::Sparql;

enum FieldName {
  Named(syn::Ident),
  Unnamed(Index),
}

impl FieldName {
  fn describe(&self) -> String {
    match self {
      FieldName::Named(ident) => ident.to_string(),
      FieldName::Unnamed(index) => index.index.to_string(),
    }
  }
}

impl ToTokens for FieldName {
  fn to_tokens(&self, tokens: &mut TokenStream) {
    match self {
      FieldName::Named(ident) => ident.to_tokens(tokens),
      FieldName::Unnamed(index) => index.to_tokens(tokens),
    }
  }
}

#[derive(Clone, Copy)]
enum FieldsKind {
  Named,
  Unnamed,
  Unit,
}

fn field_names_of(fields: &Fields) -> (FieldsKind, Vec<FieldName>) {
  match fields {
    Fields::Named(named) => (
      FieldsKind::Named,
      named
        .named
        .iter()
        .map(|field| FieldName::Named(field.ident.clone().unwrap()))
        .collect(),
    ),
    Fields::Unnamed(unnamed) => (
      FieldsKind::Unnamed,
      unnamed
        .unnamed
        .iter()
        .enumerate()
        .map(|(index, _)| FieldName::Unnamed(Index::from(index)))
        .collect(),
    ),
    Fields::Unit => (FieldsKind::Unit, vec![]),
  }
}

enum Cardinality {
  Plain,
  Optional,
  Vec,
  HashSet,
}

fn detect_cardinality(ty: &Type) -> Cardinality {
  if let Type::Path(type_path) = ty
    && let Some(path_segment) = type_path.path.segments.first()
  {
    if path_segment.ident == "Option" {
      return Cardinality::Optional;
    }
    if path_segment.ident == "Vec" {
      return Cardinality::Vec;
    }
    if path_segment.ident == "HashSet" {
      return Cardinality::HashSet;
    }
  }
  Cardinality::Plain
}

fn inner_type(ty: &Type) -> Type {
  if let Type::Path(type_path) = ty
    && let Some(path_segment) = type_path.path.segments.first()
    && (path_segment.ident == "Option"
      || path_segment.ident == "Vec"
      || path_segment.ident == "HashSet")
    && let PathArguments::AngleBracketed(arguments) = &path_segment.arguments
    && let Some(GenericArgument::Type(argument_type)) = arguments.args.first()
  {
    return argument_type.clone();
  }
  ty.clone()
}

pub fn derive_to_rdf(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let raw_input = syn::parse_macro_input!(item as DeriveInput);

  let output = match &raw_input.data {
    syn::Data::Struct(data_struct) => {
      let (_, field_names) = field_names_of(&data_struct.fields);
      let RdfType::Struct(rdf_struct) = RdfType::<Sparql>::from_derive(raw_input.clone()) else {
        unreachable!()
      };
      generate_struct_to_rdf(&rdf_struct, &field_names)
    }
    syn::Data::Enum(data_enum) => {
      let variant_idents: Vec<_> = data_enum.variants.iter().map(|v| v.ident.clone()).collect();
      let RdfType::Enum(rdf_enum) = RdfType::<Sparql>::from_derive(raw_input.clone()) else {
        unreachable!()
      };
      if rdf_enum.is_closed_list() {
        generate_closed_enum_to_rdf(&rdf_enum, &variant_idents)
      } else {
        generate_enum_to_rdf(&rdf_enum, &variant_idents)
      }
    }
    syn::Data::Union(_) => {
      proc_macro_error::abort_call_site!("union types are not supported")
    }
  };

  output.into()
}

pub fn derive_from_rdf(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let raw_input = syn::parse_macro_input!(item as DeriveInput);

  let output = match &raw_input.data {
    syn::Data::Struct(data_struct) => {
      let (fields_kind, field_names) = field_names_of(&data_struct.fields);
      let RdfType::Struct(rdf_struct) = RdfType::<Sparql>::from_derive(raw_input.clone()) else {
        unreachable!()
      };
      generate_struct_from_rdf(&rdf_struct, &field_names, fields_kind)
    }
    syn::Data::Enum(data_enum) => {
      let variant_idents: Vec<_> = data_enum.variants.iter().map(|v| v.ident.clone()).collect();
      let RdfType::Enum(rdf_enum) = RdfType::<Sparql>::from_derive(raw_input.clone()) else {
        unreachable!()
      };
      if rdf_enum.is_closed_list() {
        generate_closed_enum_from_rdf(&rdf_enum, &variant_idents)
      } else {
        generate_enum_from_rdf(&rdf_enum, &variant_idents)
      }
    }
    syn::Data::Union(_) => {
      proc_macro_error::abort_call_site!("union types are not supported")
    }
  };

  output.into()
}

/// The IRI of a closed-list enum variant. Unit variants can only ever carry a plain
/// `PredicatePath::Predicate` (a `ChainedPath` needs an attribute on an inner field, which unit
/// variants don't have), so this always matches.
fn closed_variant_iri(variant: &RdfVariant<Sparql>) -> &str {
  match variant.predicate_path() {
    PredicatePath::Predicate(iri) => iri.as_str(),
    PredicatePath::ChainedPath { .. } => {
      unreachable!("a unit variant cannot carry a chained path")
    }
  }
}

fn generate_closed_enum_to_rdf(
  rdf_enum: &RdfEnum<Sparql>,
  variant_idents: &[syn::Ident],
) -> TokenStream {
  let ident = &rdf_enum.ident;
  let variants = &rdf_enum.variants;

  let match_arms = variants
    .iter()
    .zip(variant_idents)
    .map(|(variant, variant_ident)| {
      let iri_str = closed_variant_iri(variant);
      quote! {
        #ident::#variant_ident => ::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#iri_str),
      }
    });

  quote! {
    impl ::linked_data_sparql::ToRdfTerm for #ident {
      fn to_term(
        &self,
        _quads: &mut Vec<::linked_data_sparql::reexport::oxrdf::Quad>,
      ) -> ::linked_data_sparql::reexport::oxrdf::Term {
        let named_node = match self {
          #(#match_arms)*
        };
        ::linked_data_sparql::reexport::oxrdf::Term::from(named_node)
      }
    }
  }
}

fn generate_closed_enum_from_rdf(
  rdf_enum: &RdfEnum<Sparql>,
  variant_idents: &[syn::Ident],
) -> TokenStream {
  let ident = &rdf_enum.ident;
  let variants = &rdf_enum.variants;

  let match_arms = variants
    .iter()
    .zip(variant_idents)
    .map(|(variant, variant_ident)| {
      let iri_str = closed_variant_iri(variant);
      quote! {
        #iri_str => Ok(#ident::#variant_ident),
      }
    });

  quote! {
    impl ::linked_data_sparql::FromRdfTerm for #ident {
      fn from_term(
        _dataset: &::linked_data_sparql::reexport::oxrdf::Dataset,
        term: &::linked_data_sparql::reexport::oxrdf::Term,
      ) -> Result<Self, ::linked_data_sparql::DeserializeError> {
        let named_node = match term {
          ::linked_data_sparql::reexport::oxrdf::Term::NamedNode(named_node) => named_node,
          _ => {
            return Err(::linked_data_sparql::DeserializeError::UnexpectedTerm {
              expected: "named node",
            });
          }
        };

        match named_node.as_str() {
          #(#match_arms)*
          other => Err(::linked_data_sparql::DeserializeError::InvalidLiteral(other.to_owned())),
        }
      }
    }
  }
}

fn generate_struct_to_rdf(
  rdf_struct: &RdfStruct<Sparql>,
  field_names: &[FieldName],
) -> TokenStream {
  let ident = &rdf_struct.ident;
  let fields = &rdf_struct.fields;

  let type_quad = rdf_struct.type_iri().map(|iri| iri.to_string()).map(|iri| {
    quote! {
      quads.push(::linked_data_sparql::reexport::oxrdf::Quad::new(
        subject.clone(),
        ::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
        ::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#iri),
        ::linked_data_sparql::reexport::oxrdf::GraphName::DefaultGraph,
      ));
    }
  });

  let id_field_name = fields
    .iter()
    .zip(field_names)
    .find(|(field, _)| field.is_id())
    .map(|(_, name)| name);

  let field_writes = fields
    .iter()
    .zip(field_names)
    .map(|(field, name)| generate_field_write(field, name));

  let subject_expr = if let Some(id_name) = id_field_name {
    quote! {
      ::linked_data_sparql::reexport::oxrdf::NamedOrBlankNode::from(self.#id_name.clone())
    }
  } else {
    quote! { ::linked_data_sparql::rdf_serde_support::fresh_subject() }
  };

  quote! {
    impl ::linked_data_sparql::WriteRdfQuads for #ident {
      fn write_quads(
        &self,
        subject: &::linked_data_sparql::reexport::oxrdf::NamedOrBlankNode,
        quads: &mut Vec<::linked_data_sparql::reexport::oxrdf::Quad>,
      ) {
        #type_quad
        #(#field_writes)*
      }
    }

    impl ::linked_data_sparql::ToRdfTerm for #ident {
      fn to_term(
        &self,
        quads: &mut Vec<::linked_data_sparql::reexport::oxrdf::Quad>,
      ) -> ::linked_data_sparql::reexport::oxrdf::Term {
        let subject = #subject_expr;
        ::linked_data_sparql::WriteRdfQuads::write_quads(self, &subject, quads);
        ::linked_data_sparql::reexport::oxrdf::Term::from(subject)
      }
    }
  }
}

fn generate_field_write(field: &RdfField<Sparql>, name: &FieldName) -> TokenStream {
  if field.is_ignored() || field.is_id() || field.is_graph() {
    return TokenStream::new();
  }

  if field.is_flattened() {
    return quote! {
      ::linked_data_sparql::WriteRdfQuads::write_quads(&self.#name, subject, quads);
    };
  }

  let Some(predicate) = field.predicate() else {
    return TokenStream::new();
  };
  let predicate_str = predicate.as_str();

  match detect_cardinality(&field.ty) {
    Cardinality::Plain => quote! {
      {
        let term = ::linked_data_sparql::ToRdfTerm::to_term(&self.#name, quads);
        quads.push(::linked_data_sparql::reexport::oxrdf::Quad::new(
          subject.clone(),
          ::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#predicate_str),
          term,
          ::linked_data_sparql::reexport::oxrdf::GraphName::DefaultGraph,
        ));
      }
    },
    Cardinality::Optional => quote! {
      if let Some(value) = &self.#name {
        let term = ::linked_data_sparql::ToRdfTerm::to_term(value, quads);
        quads.push(::linked_data_sparql::reexport::oxrdf::Quad::new(
          subject.clone(),
          ::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#predicate_str),
          term,
          ::linked_data_sparql::reexport::oxrdf::GraphName::DefaultGraph,
        ));
      }
    },
    // A `Vec` is ordered and may contain duplicates, so it's written as an RDF Collection, the
    // only one of the two standard RDF list shapes that preserves both.
    Cardinality::Vec => quote! {
      ::linked_data_sparql::rdf_serde_support::write_collection(
        subject,
        &::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#predicate_str),
        &self.#name,
        quads,
      );
    },
    // A `HashSet` has neither order nor duplicates, so a plain multi-valued property (repeated
    // `subject predicate value` triples) already represents it exactly.
    Cardinality::HashSet => quote! {
      for value in &self.#name {
        let term = ::linked_data_sparql::ToRdfTerm::to_term(value, quads);
        quads.push(::linked_data_sparql::reexport::oxrdf::Quad::new(
          subject.clone(),
          ::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#predicate_str),
          term,
          ::linked_data_sparql::reexport::oxrdf::GraphName::DefaultGraph,
        ));
      }
    },
  }
}

fn generate_enum_to_rdf(rdf_enum: &RdfEnum<Sparql>, variant_idents: &[syn::Ident]) -> TokenStream {
  let ident = &rdf_enum.ident;
  let variants = &rdf_enum.variants;

  let match_arms =
    variants
      .iter()
      .zip(variant_idents)
      .map(|(variant, variant_ident)| match variant.predicate_path() {
        PredicatePath::Predicate(iri) => {
          let iri_str = iri.as_str();
          quote! {
            #ident::#variant_ident(inner) => {
              let term = ::linked_data_sparql::ToRdfTerm::to_term(inner, quads);
              quads.push(::linked_data_sparql::reexport::oxrdf::Quad::new(
                subject.clone(),
                ::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#iri_str),
                term,
                ::linked_data_sparql::reexport::oxrdf::GraphName::DefaultGraph,
              ));
            }
          }
        }
        PredicatePath::ChainedPath {
          to_blank,
          from_blank,
        } => {
          let to_blank_str = to_blank.as_str();
          let from_blank_str = from_blank.as_str();
          quote! {
            #ident::#variant_ident(inner) => {
              let blank = ::linked_data_sparql::rdf_serde_support::fresh_subject();
              quads.push(::linked_data_sparql::reexport::oxrdf::Quad::new(
                subject.clone(),
                ::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#from_blank_str),
                ::linked_data_sparql::reexport::oxrdf::Term::from(blank.clone()),
                ::linked_data_sparql::reexport::oxrdf::GraphName::DefaultGraph,
              ));
              let term = ::linked_data_sparql::ToRdfTerm::to_term(inner, quads);
              quads.push(::linked_data_sparql::reexport::oxrdf::Quad::new(
                blank,
                ::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#to_blank_str),
                term,
                ::linked_data_sparql::reexport::oxrdf::GraphName::DefaultGraph,
              ));
            }
          }
        }
      });

  quote! {
    impl ::linked_data_sparql::WriteRdfQuads for #ident {
      fn write_quads(
        &self,
        subject: &::linked_data_sparql::reexport::oxrdf::NamedOrBlankNode,
        quads: &mut Vec<::linked_data_sparql::reexport::oxrdf::Quad>,
      ) {
        match self {
          #(#match_arms)*
        }
      }
    }

    impl ::linked_data_sparql::ToRdfTerm for #ident {
      fn to_term(
        &self,
        quads: &mut Vec<::linked_data_sparql::reexport::oxrdf::Quad>,
      ) -> ::linked_data_sparql::reexport::oxrdf::Term {
        let subject = ::linked_data_sparql::rdf_serde_support::fresh_subject();
        ::linked_data_sparql::WriteRdfQuads::write_quads(self, &subject, quads);
        ::linked_data_sparql::reexport::oxrdf::Term::from(subject)
      }
    }
  }
}

fn generate_struct_from_rdf(
  rdf_struct: &RdfStruct<Sparql>,
  field_names: &[FieldName],
  fields_kind: FieldsKind,
) -> TokenStream {
  let ident = &rdf_struct.ident;
  let fields = &rdf_struct.fields;

  let field_inits = fields.iter().zip(field_names).map(|(field, name)| {
    let value_expr = generate_field_read(field, name);
    match fields_kind {
      FieldsKind::Named | FieldsKind::Unit => quote! { #name: #value_expr },
      FieldsKind::Unnamed => quote! { #value_expr },
    }
  });

  let construct_expr = match fields_kind {
    FieldsKind::Named => quote! { #ident { #(#field_inits),* } },
    FieldsKind::Unnamed => quote! { #ident ( #(#field_inits),* ) },
    FieldsKind::Unit => quote! { #ident },
  };

  quote! {
    impl ::linked_data_sparql::FromRdfSubject for #ident {
      fn deserialize_subject(
        dataset: &::linked_data_sparql::reexport::oxrdf::Dataset,
        subject: &::linked_data_sparql::reexport::oxrdf::NamedOrBlankNode,
      ) -> Result<Self, ::linked_data_sparql::DeserializeError> {
        Ok(#construct_expr)
      }
    }

    impl ::linked_data_sparql::FromRdfTerm for #ident {
      fn from_term(
        dataset: &::linked_data_sparql::reexport::oxrdf::Dataset,
        term: &::linked_data_sparql::reexport::oxrdf::Term,
      ) -> Result<Self, ::linked_data_sparql::DeserializeError> {
        let subject = ::linked_data_sparql::reexport::oxrdf::NamedOrBlankNode::try_from(term.clone())
          .map_err(|_| ::linked_data_sparql::DeserializeError::UnexpectedTerm { expected: "subject" })?;
        <Self as ::linked_data_sparql::FromRdfSubject>::deserialize_subject(dataset, &subject)
      }
    }
  }
}

fn generate_field_read(field: &RdfField<Sparql>, name: &FieldName) -> TokenStream {
  if field.is_id() {
    return quote! {
      <::linked_data_sparql::reexport::oxrdf::NamedNode as ::linked_data_sparql::FromRdfTerm>::from_term(
        dataset,
        &::linked_data_sparql::reexport::oxrdf::Term::from(subject.clone()),
      )?
    };
  }

  if field.is_ignored() || field.is_graph() {
    return quote! { ::core::default::Default::default() };
  }

  if field.is_flattened() {
    let ty = &field.ty;
    return quote! {
      <#ty as ::linked_data_sparql::FromRdfSubject>::deserialize_subject(dataset, subject)?
    };
  }

  let Some(predicate) = field.predicate() else {
    return quote! { ::core::default::Default::default() };
  };
  let predicate_str = predicate.as_str();
  let field_name_str = name.describe();

  match detect_cardinality(&field.ty) {
    Cardinality::Plain => quote! {
      ::linked_data_sparql::rdf_serde_support::single_field(
        dataset,
        subject,
        &::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#predicate_str),
        #field_name_str,
      )?
    },
    Cardinality::Optional => quote! {
      ::linked_data_sparql::rdf_serde_support::optional_field(
        dataset,
        subject,
        &::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#predicate_str),
      )?
    },
    Cardinality::Vec => quote! {
      ::linked_data_sparql::rdf_serde_support::vec_field(
        dataset,
        subject,
        &::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#predicate_str),
      )?
    },
    Cardinality::HashSet => quote! {
      ::linked_data_sparql::rdf_serde_support::hash_set_field(
        dataset,
        subject,
        &::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#predicate_str),
      )?
    },
  }
}

fn generate_enum_from_rdf(
  rdf_enum: &RdfEnum<Sparql>,
  variant_idents: &[syn::Ident],
) -> TokenStream {
  let ident = &rdf_enum.ident;
  let variants = &rdf_enum.variants;

  let attempts = variants.iter().zip(variant_idents).map(|(variant, variant_ident)| {
    // Only reached for tagged-union enums (closed lists take the `generate_closed_enum_*` path),
    // where every variant is guaranteed to wrap a single field.
    let ty = inner_type(
      variant
        .ty
        .as_ref()
        .expect("tagged union enum variant is missing its inner type"),
    );
    match variant.predicate_path() {
      PredicatePath::Predicate(iri) => {
        let iri_str = iri.as_str();
        quote! {
          {
            let mut objects = ::linked_data_sparql::rdf_serde_support::objects_for(
              dataset,
              subject,
              &::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#iri_str),
            );
            if let Some(term) = objects.next() {
              return Ok(#ident::#variant_ident(
                <#ty as ::linked_data_sparql::FromRdfTerm>::from_term(dataset, &term)?,
              ));
            }
          }
        }
      }
      PredicatePath::ChainedPath {
        to_blank,
        from_blank,
      } => {
        let to_blank_str = to_blank.as_str();
        let from_blank_str = from_blank.as_str();
        quote! {
          {
            let mut blanks = ::linked_data_sparql::rdf_serde_support::objects_for(
              dataset,
              subject,
              &::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#from_blank_str),
            );
            if let Some(blank_term) = blanks.next()
              && let Ok(blank_subject) = ::linked_data_sparql::reexport::oxrdf::NamedOrBlankNode::try_from(blank_term)
            {
              let mut objects = ::linked_data_sparql::rdf_serde_support::objects_for(
                dataset,
                &blank_subject,
                &::linked_data_sparql::reexport::oxrdf::NamedNode::new_unchecked(#to_blank_str),
              );
              if let Some(term) = objects.next() {
                return Ok(#ident::#variant_ident(
                  <#ty as ::linked_data_sparql::FromRdfTerm>::from_term(dataset, &term)?,
                ));
              }
            }
          }
        }
      }
    }
  });

  quote! {
    impl ::linked_data_sparql::FromRdfSubject for #ident {
      fn deserialize_subject(
        dataset: &::linked_data_sparql::reexport::oxrdf::Dataset,
        subject: &::linked_data_sparql::reexport::oxrdf::NamedOrBlankNode,
      ) -> Result<Self, ::linked_data_sparql::DeserializeError> {
        #(#attempts)*
        Err(::linked_data_sparql::DeserializeError::NoMatchingVariant)
      }
    }

    impl ::linked_data_sparql::FromRdfTerm for #ident {
      fn from_term(
        dataset: &::linked_data_sparql::reexport::oxrdf::Dataset,
        term: &::linked_data_sparql::reexport::oxrdf::Term,
      ) -> Result<Self, ::linked_data_sparql::DeserializeError> {
        let subject = ::linked_data_sparql::reexport::oxrdf::NamedOrBlankNode::try_from(term.clone())
          .map_err(|_| ::linked_data_sparql::DeserializeError::UnexpectedTerm { expected: "subject" })?;
        <Self as ::linked_data_sparql::FromRdfSubject>::deserialize_subject(dataset, &subject)
      }
    }
  }
}
