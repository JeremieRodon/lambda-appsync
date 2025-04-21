//! Crate not intended for direct use.

mod appsync_lambda_main;
mod appsync_operation;
mod common;

use proc_macro::TokenStream;

#[proc_macro]
pub fn appsync_lambda_main(input: TokenStream) -> TokenStream {
    appsync_lambda_main::appsync_lambda_main_impl(input)
}

#[proc_macro_attribute]
pub fn appsync_operation(args: TokenStream, input: TokenStream) -> TokenStream {
    appsync_operation::appsync_operation_impl(args, input)
}
