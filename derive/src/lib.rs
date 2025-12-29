use linked_data_core::{
  PredicatePath, RdfEnum, RdfField, RdfStruct, RdfType, RdfVariant, TokenGenerator,
};
use proc_macro_error::proc_macro_error;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{DeriveInput, GenericArgument, PathArguments, Type};

#[proc_macro_error]
#[proc_macro_derive(Sparql, attributes(ld))]
pub fn derive_serialize(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let raw_input = syn::parse_macro_input!(item as DeriveInput);
  let linked_data_type: RdfType<Sparql> = RdfType::from_derive(raw_input);

  let mut output = TokenStream::new();
  linked_data_type.to_tokens(&mut output);
  output.into()
}

struct Sparql;

impl TokenGenerator for Sparql {
  fn generate_type_tokens(linked_data_type: &RdfType<Self>, tokens: &mut TokenStream) {
    tokens.extend(quote::quote! {
        use ::linked_data_sparql::Join as _;
    });

    let implementations = match linked_data_type {
      RdfType::Enum(rdf_enum) => quote::quote! {#rdf_enum},
      RdfType::Struct(rdf_struct) => quote::quote! {#rdf_struct},
    };

    tokens.extend(implementations)
  }

  fn generate_struct_tokens(rdf_struct: &RdfStruct<Self>, tokens: &mut TokenStream) {
    let ident = &rdf_struct.ident;
    let fields = &rdf_struct.fields;

    let type_tokens =
      rdf_struct.type_iri().map(|iri| iri.clone().into_string())
        .map(|type_iri| {
          quote::quote! {
            let construct_query = construct_query.join_with(
              binding_variable.clone(),
              ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
              ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked(#type_iri),
            );
          }
        })
        .unwrap_or_default();

    tokens.extend(quote::quote! {
      impl ::linked_data_sparql::ToConstructQuery for #ident {
        fn to_query_with_binding(binding_variable: ::linked_data_sparql::reexport::spargebra::term::Variable) -> ::linked_data_sparql::ConstructQuery {
          let construct_query = ::linked_data_sparql::ConstructQuery::default();
          #type_tokens
          #(#fields)*
          construct_query
        }
      }
    });
  }

  fn generate_enum_tokens(r#enum: &RdfEnum<Self>, tokens: &mut TokenStream) {
    let variants = &r#enum.variants;
    let ident = &r#enum.ident;

    tokens.extend(quote::quote! {
      impl ::linked_data_sparql::ToConstructQuery for #ident {
        fn to_query_with_binding(binding_variable: ::linked_data_sparql::reexport::spargebra::term::Variable) -> ::linked_data_sparql::ConstructQuery {
          let construct_query = ::linked_data_sparql::ConstructQuery::default();
          #(#variants)*
          construct_query
        }
      }
    });
  }

  fn generate_variant_tokens(variant: &RdfVariant<Self>, tokens: &mut TokenStream) {
    let ty = &variant.ty;

    let (iri_str, chained_object_with_predicate) = match &variant.predicate_path() {
      PredicatePath::Predicate(iri) => (iri.as_str(), quote::quote! {}),
      PredicatePath::ChainedPath {
        to_blank,
        from_blank,
      } => {
        let to_blank_str = to_blank.as_str();

        (
          from_blank.as_str(),
          quote::quote! {
            let (construct_query, _) = construct_query.union_with_binding::<#ty>(
              object,
              ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked(#to_blank_str),
            );
          },
        )
      }
    };

    tokens.extend(quote::quote! {
      let (construct_query, object) = construct_query.union_with_binding::<#ty>(
        binding_variable.clone(),
        ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked(#iri_str),
      );
      #chained_object_with_predicate
    });
  }

  fn generate_field_tokens(field: &RdfField<Self>, tokens: &mut TokenStream) {
    if field.is_ignored() {
      return;
    }

    if field.is_flattened() {
      let ty = &field.ty;
      tokens.extend(quote::quote! {
        let construct_query = construct_query.join(#ty::to_query_with_binding(binding_variable.clone()));
      });
    }

    if let Some(predicate) = field.predicate() {
      let ty = type_for_to_query_with_binding(field);
      let predicate_iri = predicate.as_str();

      if left_join_required(field) {
        tokens.extend(quote::quote! {
          let (construct_query, object) = construct_query.left_join_with_binding::<#ty>(
            binding_variable.clone(),
            ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked(#predicate_iri),
          );
        });
      } else {
        tokens.extend(quote::quote! {
          let (construct_query, object) = construct_query.join_with_binding::<#ty>(
            binding_variable.clone(),
            ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked(#predicate_iri),
          );
        });
      }
    }
  }
}

fn type_for_to_query_with_binding(field: &RdfField<Sparql>) -> Type {
  composed_inner_type(field).unwrap_or(field.ty.clone())
}

fn left_join_required(field: &RdfField<Sparql>) -> bool {
  composed_inner_type(field).is_some()
}

fn composed_inner_type(field: &RdfField<Sparql>) -> Option<Type> {
  if let Type::Path(type_path) = &field.ty
    && let Some(path_segment) = type_path.path.segments.first()
    && (path_segment.ident == "Option"
      || path_segment.ident == "Vec"
      || path_segment.ident == "HashSet")
  {
    if let PathArguments::AngleBracketed(arguments) = &path_segment.arguments {
      if let Some(GenericArgument::Type(argument_type)) = arguments.args.first() {
        Some(argument_type.clone())
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  }
}
