use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::{Parse, ParseStream}, Ident, Token, ExprArray, Expr, parse_macro_input};

struct GroupedOrderingInstanceSpec {
    grouped_ordering_name: Ident,
    grouped_ordering_group_name: Ident,
    groups: Vec<Ident>,
}

impl Parse for GroupedOrderingInstanceSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let grouped_ordering_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let grouped_ordering_group_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let groups: Vec<Ident> = input
            .parse::<ExprArray>()?
            .elems
            .into_iter()
            .map(|group| match group {
                Expr::Path(expr) if expr.path.get_ident().is_some() => {
                    expr.path.get_ident().unwrap().clone()
                }
                _ => panic!("Expected ident"),
            })
            .collect();
        Ok(Self {
            grouped_ordering_name,
            grouped_ordering_group_name,
            groups,
        })
    }
}

pub fn grouped_ordering_instance(input: TokenStream) -> TokenStream {
    let grouped_ordering_instance_spec: GroupedOrderingInstanceSpec = parse_macro_input!(input);
    let grouped_ordering_name = &grouped_ordering_instance_spec.grouped_ordering_name;
    let grouped_ordering_group_name = &grouped_ordering_instance_spec.grouped_ordering_group_name;
    let groups = grouped_ordering_instance_spec.groups.iter().map(|group| {
        quote! {
            #grouped_ordering_group_name::#group
        }
    }).collect::<Vec<_>>();

    quote! {
        #grouped_ordering_name::try_from([
            #(#groups),*
        ]).unwrap()
    }.into()
}
