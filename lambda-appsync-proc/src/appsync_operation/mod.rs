use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{braced, parenthesized, parse::Parse, parse_macro_input, Ident, Token, Type, Visibility};

use crate::common::{Name, OperationKind};

enum ArgsOption {
    KeepOriginalFunctionName,
}
impl Parse for ArgsOption {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        match ident.to_string().as_str() {
            "keep_original_function_name" => Ok(Self::KeepOriginalFunctionName),
            _ => Err(syn::Error::new(
                ident.span(),
                format!("Unknown option `{ident}`",),
            )),
        }
    }
}

struct Args {
    op_kind: OperationKind,
    op_name: Name,
    op_name_span: proc_macro2::Span,
    keep_original_function_name: bool,
}
impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op_kind = input.parse::<Ident>()?;
        let op_kind_s = op_kind.to_string();
        let op_kind = match op_kind_s.as_str() {
            "query" => OperationKind::Query,
            "mutation" => OperationKind::Mutation,
            "subscription" => OperationKind::Subscription,
            _ => {
                return Err(syn::Error::new(
                    op_kind.span(),
                    format!(
                        "Expected one of `query`, `mutation` or `subscription`, got `{op_kind_s}`."
                    ),
                ));
            }
        };
        let op_name;
        _ = parenthesized!(op_name in input);
        let op_name = op_name.parse::<Ident>()?;
        let op_name_span = op_name.span();
        let op_name = Name::from(op_name.to_string());

        let mut args = Self {
            op_kind,
            op_name,
            op_name_span,
            keep_original_function_name: false,
        };

        while input.peek(Token![,]) {
            _ = input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            // We got an option
            let option = input.parse::<ArgsOption>()?;
            match option {
                ArgsOption::KeepOriginalFunctionName => args.keep_original_function_name = true,
            }
        }
        Ok(args)
    }
}

struct FctArg {
    is_mut: bool,
    name: Ident,
    ty: Type,
}
impl Parse for FctArg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let is_mut = if input.peek(Token![mut]) {
            _ = input.parse::<Token![mut]>()?;
            true
        } else {
            false
        };
        let name = input.parse()?;
        _ = input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(Self { is_mut, name, ty })
    }
}
impl ToTokens for FctArg {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if self.is_mut {
            tokens.extend(quote! {mut});
        }
        let name = &self.name;
        let ty = &self.ty;
        tokens.extend(quote! {
            #name: #ty
        });
    }
}
struct Fct {
    vis: Option<Visibility>,
    fct_name: Ident,
    args: Vec<FctArg>,
    return_type: Type,
    body: TokenStream2,
}
impl Parse for Fct {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let vis = if input.peek(Token![pub]) {
            Some(input.parse()?)
        } else {
            None
        };

        if !(input.peek(Token![async]) && input.peek2(Token![fn])) {
            return Err(syn::Error::new(
                input.span(),
                "appsync_operation macro must be use on an async function",
            ));
        }
        _ = input.parse::<Token![async]>()?;
        _ = input.parse::<Token![fn]>()?;
        let fct_name = input.parse()?;

        let args_input;
        _ = parenthesized!(args_input in input);
        let mut args = vec![];
        while let Ok(arg) = args_input.parse::<FctArg>() {
            args.push(arg);
            if args_input.peek(Token![,]) {
                _ = args_input.parse::<Token![,]>()?;
            }
        }

        _ = input.parse::<Token![->]>()?;

        let return_type = input.parse()?;

        let body_input;
        _ = braced!(body_input in input);
        let body = body_input.parse()?;

        Ok(Self {
            vis,
            fct_name,
            args,
            return_type,
            body,
        })
    }
}

struct AppsyncOperation {
    args: Args,
    fct: Fct,
}
impl TryFrom<(Args, Fct)> for AppsyncOperation {
    type Error = syn::Error;

    fn try_from((args, fct): (Args, Fct)) -> Result<Self, Self::Error> {
        Ok(Self { args, fct })
    }
}
impl ToTokens for AppsyncOperation {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let vis = if let Some(ref vis) = self.fct.vis {
            vis.into_token_stream()
        } else {
            TokenStream2::new()
        };
        let mut op_fct_name = self
            .args
            .op_name
            .to_prefixed_fct_ident(&self.args.op_kind.fct_prefix());
        op_fct_name.set_span(self.args.op_name_span);

        let fct_name = &self.fct.fct_name;
        let args = &self.fct.args;
        let arg_names = self.fct.args.iter().map(|a| &a.name).collect::<Vec<_>>();
        let return_type = &self.fct.return_type;
        let body = &self.fct.body;

        tokens.extend(quote! {
            impl crate::Operation {
                #vis async fn #op_fct_name(
                    #(#args,)*
                ) -> #return_type {
                    // This is just a marker to ensure an error is thrown if the user did not chose
                    // the correct signature for the function. Should be optimized away by the compiler.
                    if false {
                        return <crate::Operation as crate::DefautOperations>::#op_fct_name(
                            #(#arg_names,)*
                        )
                        .await;
                    }
                    #body
                }
            }
        });
        if self.args.keep_original_function_name {
            tokens.extend(quote! {
                #vis async fn #fct_name(
                    #(#args,)*
                ) -> #return_type {
                    crate::Operation::#op_fct_name(#(#arg_names,)*).await
                }
            });
        }
    }
}

pub(crate) fn appsync_operation_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let fct = parse_macro_input!(input as Fct);
    let appsync_operation = match AppsyncOperation::try_from((args, fct)) {
        Ok(ao) => ao,
        Err(e) => return e.into_compile_error().into(),
    };

    appsync_operation.into_token_stream().into()
}
