use proc_macro::TokenStream;

mod grouped_ordering;
mod grouped_ordering_instance;

use grouped_ordering::grouped_ordering_for_crate_name;

#[proc_macro]
pub fn grouped_ordering(input: TokenStream) -> TokenStream {
    grouped_ordering_for_crate_name(input, "grouped_ordering")
}

#[proc_macro]
pub fn grouped_ordering_crate_internal(input: TokenStream) -> TokenStream {
    grouped_ordering_for_crate_name(input, "crate")
}

#[proc_macro]
pub fn grouped_ordering_instance(input: TokenStream) -> TokenStream {
    grouped_ordering_instance::grouped_ordering_instance(input)
}
