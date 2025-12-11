use linked_data_core::{
  PredicatePath, RdfEnum, RdfField, RdfStruct, RdfType, RdfVariant, TokenGenerator,
};
use proc_macro_error::proc_macro_error;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::DeriveInput;

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
            .join_with(
              binding_variable.clone(),
              ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
              ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked(#type_iri),
            )
          }
        })
        .unwrap_or_default();

    tokens.extend(quote::quote! {
      impl ::linked_data_sparql::ToConstructQuery for #ident {
        fn to_query_with_binding(binding_variable: ::linked_data_sparql::reexport::spargebra::term::Variable) -> ::linked_data_sparql::ConstructQuery {
          ::linked_data_sparql::ConstructQuery::default()
          #(#fields)*
          #type_tokens
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
          ::linked_data_sparql::ConstructQuery::default()
          #(#variants)*
        }
      }
    });
  }

  fn generate_variant_tokens(variant: &RdfVariant<Self>, tokens: &mut TokenStream) {
    let ty = &variant.ty;
    let inner_generator = quote::quote! { #ty::to_query_with_binding };

    let (iri_str, predicate_generator) = match &variant.predicate_path() {
      PredicatePath::Predicate(iri) => (iri.as_str(), inner_generator),
      PredicatePath::ChainedPath {
        to_blank,
        from_blank,
      } => {
        let to_blank_str = to_blank.as_str();

        (
          from_blank.as_str(),
          quote::quote! {
            ::linked_data_sparql::with_predicate(
              ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked(#to_blank_str),
              #inner_generator
            )
          },
        )
      }
    };

    tokens.extend(quote::quote! {
      .union_with_binding(
        binding_variable.clone(),
        ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked(#iri_str),
        #predicate_generator
      )
    });
  }

  fn generate_field_tokens(field: &RdfField<Self>, tokens: &mut TokenStream) {
    if field.is_ignored() {
      return;
    }

    if field.is_flattened() {
      let ty = &field.ty;
      tokens.extend(quote::quote! {
        .join(#ty::to_query_with_binding(binding_variable.clone()))
      });
    }

    if let Some(predicate) = field.predicate() {
      let ty = &field.ty;
      let predicate_iri = predicate.as_str();
      tokens.extend(quote::quote! {
        .join_with_binding(
          binding_variable.clone(),
          ::linked_data_sparql::reexport::spargebra::term::NamedNode::new_unchecked(#predicate_iri),
          <#ty>::to_query_with_binding,
        )
      });
    }
  }
}
