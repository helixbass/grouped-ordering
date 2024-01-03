use inflector::Inflector;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, ExprArray, Ident, Token,
};

struct GroupedOrderingSpec {
    name: Ident,
    groups: Vec<Ident>,
}

impl Parse for GroupedOrderingSpec {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
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
            name,
            groups,
        })
    }
}

pub fn grouped_ordering_for_crate_name(input: TokenStream, crate_name: &str) -> TokenStream {
    let crate_name = format_ident!("{}", crate_name);
    let grouped_ordering_spec = parse_macro_input!(input as GroupedOrderingSpec);

    let group_enum_name = format_ident!("{}Group", grouped_ordering_spec.name);
    let group_enum_definition = get_group_enum_definition(&grouped_ordering_spec, &group_enum_name);
    let grouped_ordering_struct_definition = get_grouped_ordering_struct_definition(&grouped_ordering_spec, &group_enum_name);
    let impl_grouped_ordering = get_impl_grouped_ordering(&grouped_ordering_spec, &group_enum_name, &crate_name);
    let impl_try_from = get_impl_try_from(&grouped_ordering_spec, &group_enum_name);
    let impl_default = get_impl_default(&grouped_ordering_spec, &group_enum_name);
    let impl_deserialize = get_impl_deserialize(&grouped_ordering_spec, &group_enum_name);
    let instantiate_macro_definition = get_instantiate_macro_definition(&grouped_ordering_spec, &group_enum_name);

    quote! {
        #group_enum_definition

        #grouped_ordering_struct_definition

        #impl_grouped_ordering

        #impl_try_from

        #impl_default

        #impl_deserialize

        #instantiate_macro_definition
    }.into()
}

fn get_group_enum_definition(grouped_ordering_spec: &GroupedOrderingSpec, group_enum_name: &Ident) -> proc_macro2::TokenStream {
    let groups = &grouped_ordering_spec.groups;

    quote! {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Deserialize)]
        #[serde(rename_all = "kebab-case")]
        enum #group_enum_name {
            #(#groups),*
        }
    }
}

fn get_grouped_ordering_struct_definition(grouped_ordering_spec: &GroupedOrderingSpec, group_enum_name: &Ident) -> proc_macro2::TokenStream {
    let name = &grouped_ordering_spec.name;
    let num_groups = grouped_ordering_spec.groups.len();

    quote! {
        #[derive(Debug)]
        struct #name {
            pub groups: [#group_enum_name; #num_groups],
            index_lookup: std::collections::HashMap<#group_enum_name, usize>,
        }
    }
}

fn get_impl_grouped_ordering(grouped_ordering_spec: &GroupedOrderingSpec, group_enum_name: &Ident, crate_name: &Ident) -> proc_macro2::TokenStream {
    let name = &grouped_ordering_spec.name;

    quote! {
        impl #crate_name::GroupedOrdering for #name {
            type Group = #group_enum_name;

            fn compare(
                &self,
                a: &#group_enum_name,
                b: &#group_enum_name,
            ) -> std::cmp::Ordering {
                self.index_lookup[a].cmp(&self.index_lookup[b])
            }
        }
    }
}

fn get_impl_try_from(grouped_ordering_spec: &GroupedOrderingSpec, group_enum_name: &Ident) -> proc_macro2::TokenStream {
    let name = &grouped_ordering_spec.name;
    let num_groups = grouped_ordering_spec.groups.len();

    quote! {
        impl TryFrom<[#group_enum_name; #num_groups]> for #name {
            type Error = String;

            fn try_from(groups: [#group_enum_name; #num_groups]) -> Result<Self, Self::Error> {
                let index_lookup: std::collections::HashMap<#group_enum_name, usize> = groups
                    .into_iter()
                    .enumerate()
                    .map(|(index, group)| (group, index))
                    .collect();
                if index_lookup.len() < #num_groups {
                    return Err("Found duplicate groups".to_owned());
                }
                Ok(Self {
                    groups,
                    index_lookup,
                })
            }
        }
    }
}

fn get_impl_default(grouped_ordering_spec: &GroupedOrderingSpec, group_enum_name: &Ident) -> proc_macro2::TokenStream {
    let name = &grouped_ordering_spec.name;
    let qualified_groups = grouped_ordering_spec.groups.iter().map(|group| {
        quote! {
            #group_enum_name::#group
        }
    }).collect::<Vec<_>>();

    quote! {
        impl Default for #name {
            fn default() -> Self {
                Self::try_from([#(#qualified_groups),*]).unwrap()
            }
        }
    }
}

fn get_impl_deserialize(grouped_ordering_spec: &GroupedOrderingSpec, group_enum_name: &Ident) -> proc_macro2::TokenStream {
    let name = &grouped_ordering_spec.name;
    let num_groups = grouped_ordering_spec.groups.len();

    quote! {
        impl<'de> serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let array = <[#group_enum_name; #num_groups] as serde::Deserialize>::deserialize(deserializer)?;
                use serde::de::Error;
                Self::try_from(array).map_err(|_| D::Error::custom("Expected all variants"))
            }
        }
    }
}

fn get_instantiate_macro_definition(grouped_ordering_spec: &GroupedOrderingSpec, group_enum_name: &Ident) -> proc_macro2::TokenStream {
    let macro_name = format_ident!("{}", grouped_ordering_spec.name.to_string().to_snake_case());
    let name = &grouped_ordering_spec.name;

    quote! {
        #[allow(unused_macros)]
        macro_rules! #macro_name {
            ($($group:ident),* $(,)?) => {
                grouped_ordering_proc_macros::grouped_ordering_instance!(
                    #name,
                    #group_enum_name,
                    [$($group),*]
                )
            }
        }
    }
}
