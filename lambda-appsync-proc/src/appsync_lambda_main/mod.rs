mod graphql;

use std::collections::HashMap;

use graphql::{FieldTypeOverride, GraphQLSchema};
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parenthesized, parse::Parse, parse_macro_input, LitBool, LitStr, Token, Type};

struct AWSClient {
    fct_identifier: Ident,
    client_type: Type,
}
impl Parse for AWSClient {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Expected example:
        // dynamodb() -> aws_sdk_dynamodb::Client
        let fct_identifier = input.parse::<Ident>()?;
        let _empty;
        _ = parenthesized!(_empty in input);
        _ = input.parse::<Token![->]>()?;
        let client_type = input.parse::<syn::Type>()?;
        Ok(Self {
            fct_identifier,
            client_type,
        })
    }
}
impl AWSClient {
    fn is_next(input: syn::parse::ParseStream) -> bool {
        Self::parse(&input.fork()).is_ok()
    }
    fn aws_config_getter() -> TokenStream2 {
        quote! {
            static AWS_SDK_CONFIG: ::std::sync::OnceLock<::lambda_appsync::aws_config::SdkConfig> = ::std::sync::OnceLock::new();
            pub fn aws_sdk_config() -> &'static ::lambda_appsync::aws_config::SdkConfig {
                AWS_SDK_CONFIG.get().unwrap()
            }
        }
    }
    fn aws_config_init() -> TokenStream2 {
        quote! {
            AWS_SDK_CONFIG.set(::lambda_appsync::aws_config::load_from_env().await).unwrap();
        }
    }
    fn aws_client_getter(&self) -> impl ToTokens {
        let Self {
            fct_identifier,
            client_type,
        } = self;
        quote! {
            pub fn #fct_identifier() -> #client_type {
                static CLIENT: ::std::sync::OnceLock<#client_type> = ::std::sync::OnceLock::new();
                CLIENT.get_or_init(||<#client_type>::new(aws_sdk_config())).clone()
            }
        }
    }
}

enum OptionalParameter {
    Batch(bool),
    ExcludeLambdaHandler(bool),
    OnlyLambdaHandler(bool),
    ExcludeAppsyncTypes(bool),
    OnlyAppsyncTypes(bool),
    ExcludeAppsyncOperations(bool),
    OnlyAppsyncOperations(bool),
    Hook(Ident),
    FieldTypeOverride(FieldTypeOverride),
}
impl Parse for OptionalParameter {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>()?;
        _ = input.parse::<Token![=]>()?;
        match ident.to_string().as_str() {
            "batch" => Ok(Self::Batch(input.parse::<LitBool>()?.value())),
            "exclude_lambda_handler" => Ok(Self::ExcludeLambdaHandler(
                input.parse::<LitBool>()?.value(),
            )),
            "only_lambda_handler" => Ok(Self::OnlyLambdaHandler(input.parse::<LitBool>()?.value())),
            "exclude_appsync_types" => {
                Ok(Self::ExcludeAppsyncTypes(input.parse::<LitBool>()?.value()))
            }
            "only_appsync_types" => Ok(Self::OnlyAppsyncTypes(input.parse::<LitBool>()?.value())),
            "exclude_appsync_operations" => Ok(Self::ExcludeAppsyncOperations(
                input.parse::<LitBool>()?.value(),
            )),
            "only_appsync_operations" => Ok(Self::OnlyAppsyncOperations(
                input.parse::<LitBool>()?.value(),
            )),
            "hook" => Ok(Self::Hook(input.parse()?)),
            "field_type_override" => Ok(Self::FieldTypeOverride(input.parse()?)),
            _ => Err(syn::Error::new(
                ident.span(),
                format!("Unknown parameter `{ident}`",),
            )),
        }
    }
}

struct OptionalParameters {
    batch: bool,
    appsync_types: bool,
    appsync_operations: bool,
    lambda_handler: bool,
    hook: Option<Ident>,
    ftos: HashMap<String, Vec<FieldTypeOverride>>,
}
impl Default for OptionalParameters {
    fn default() -> Self {
        Self {
            batch: true,
            appsync_types: true,
            appsync_operations: true,
            lambda_handler: true,
            hook: None,
            ftos: HashMap::new(),
        }
    }
}
impl OptionalParameters {
    fn set(&mut self, p: OptionalParameter) {
        match p {
            OptionalParameter::Batch(batch) => self.batch = batch,
            OptionalParameter::ExcludeLambdaHandler(b) if b => self.lambda_handler = false,
            OptionalParameter::OnlyLambdaHandler(b) if b => {
                self.lambda_handler = true;
                self.appsync_types = false;
                self.appsync_operations = false;
            }
            OptionalParameter::ExcludeAppsyncTypes(b) if b => self.appsync_types = false,
            OptionalParameter::OnlyAppsyncTypes(b) if b => {
                self.lambda_handler = false;
                self.appsync_types = true;
                self.appsync_operations = false;
            }
            OptionalParameter::ExcludeAppsyncOperations(b) if b => self.appsync_operations = false,
            OptionalParameter::OnlyAppsyncOperations(b) if b => {
                self.lambda_handler = false;
                self.appsync_types = false;
                self.appsync_operations = true;
            }
            OptionalParameter::Hook(ident) => {
                self.hook.replace(ident);
            }
            OptionalParameter::FieldTypeOverride(fto) => {
                self.ftos.entry(fto.structure_name()).or_default().push(fto);
            }
            OptionalParameter::ExcludeLambdaHandler(_) => (),
            OptionalParameter::OnlyLambdaHandler(_) => (),
            OptionalParameter::ExcludeAppsyncTypes(_) => (),
            OptionalParameter::OnlyAppsyncTypes(_) => (),
            OptionalParameter::ExcludeAppsyncOperations(_) => (),
            OptionalParameter::OnlyAppsyncOperations(_) => (),
        }
    }
}

struct AppsyncLambdaMain {
    graphql_schema: GraphQLSchema,
    aws_clients: Vec<AWSClient>,
    options: OptionalParameters,
}

impl Parse for AppsyncLambdaMain {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let graphql_schema_path = input.parse::<LitStr>()?;
        let schema_str = std::fs::read_to_string(graphql_schema_path.value()).map_err(|e| {
            syn::Error::new(
                graphql_schema_path.span(),
                format!("Could not open GraphQL schema file ({e})",),
            )
        })?;
        let schema = graphql_parser::parse_schema(&schema_str)
            .map_err(|e| {
                syn::Error::new(
                    graphql_schema_path.span(),
                    format!("Could not parse GraphQL schema file ({e})",),
                )
            })?
            .into_static();

        let mut options = OptionalParameters::default();
        let mut aws_clients = vec![];

        while input.peek(Token![,]) {
            _ = input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }
            if input.peek(syn::Ident) && input.peek2(Token![=]) {
                // That's a parameter
                options.set(input.parse()?);
            } else if AWSClient::is_next(input) {
                aws_clients.push(input.parse::<AWSClient>()?);
            } else {
                return Err(syn::Error::new(input.span(), "Unknown argument"));
            }
        }

        let ftos = options.ftos;
        options.ftos = HashMap::new();

        let graphql_schema = GraphQLSchema::new(schema, graphql_schema_path.span(), ftos)?;

        Ok(Self {
            graphql_schema,
            aws_clients,
            options,
        })
    }
}

impl AppsyncLambdaMain {
    fn appsync_event_handler(&self, tokens: &mut TokenStream2) {
        let call_hook = if let Some(ref hook) = self.options.hook {
            quote_spanned! {hook.span()=>
                if let Some(resp) = #hook(&event).await {
                    return resp;
                }
            }
        } else {
            quote! {}
        };
        tokens.extend(quote! {
            async fn appsync_handler(mut event: ::lambda_appsync::AppsyncEvent<Operation>) -> ::lambda_appsync::AppsyncResponse {
                ::lambda_appsync::log::info!("event={event:?}");
                ::lambda_appsync::log::info!("operation={:?}", event.info.operation);

                #call_hook

                let args = event.args.take();
                event.info.operation.execute(args, &event).await
            }
        });
        if self.options.batch {
            tokens.extend(quote! {
                async fn appsync_batch_handler(
                    events: Vec<::lambda_appsync::AppsyncEvent<Operation>>,
                ) -> Vec<::lambda_appsync::AppsyncResponse> {
                    let handles = events
                        .into_iter()
                        .map(|e| ::lambda_appsync::tokio::spawn(appsync_handler(e)))
                        .collect::<Vec<_>>();

                    let mut results = vec![];
                    for h in handles {
                        results.push(h.await.unwrap())
                    }
                    results
                }

            });
        }
    }

    fn lambda_main(&self, tokens: &mut TokenStream2) {
        let (config_init, config_getter) = if !self.aws_clients.is_empty() {
            (AWSClient::aws_config_init(), AWSClient::aws_config_getter())
        } else {
            (TokenStream2::new(), TokenStream2::new())
        };
        let aws_client_getters = self.aws_clients.iter().map(|ac| ac.aws_client_getter());

        let (appsync_handler, ret_type) = if self.options.batch {
            (
                format_ident!("appsync_batch_handler"),
                quote! {Vec<::lambda_appsync::AppsyncResponse>},
            )
        } else {
            (
                format_ident!("appsync_handler"),
                quote! {::lambda_appsync::AppsyncResponse},
            )
        };

        tokens.extend(quote! {
            async fn function_handler(
                event: ::lambda_appsync::lambda_runtime::LambdaEvent<::lambda_appsync::serde_json::Value>,
            ) -> Result<#ret_type, ::lambda_appsync::lambda_runtime::Error> {
                ::lambda_appsync::log::debug!("{event:?}");
                ::lambda_appsync::log::info!("{}", ::lambda_appsync::serde_json::json!(event.payload));
                Ok(#appsync_handler(::lambda_appsync::serde_json::from_value(event.payload)?).await)
            }

            #config_getter

            #(#aws_client_getters)*

            use ::lambda_appsync::tokio;
            #[tokio::main]
            async fn main() -> Result<(), ::lambda_appsync::lambda_runtime::Error> {
                ::lambda_appsync::env_logger::Builder::from_env(
                    ::lambda_appsync::env_logger::Env::default()
                        .default_filter_or("info,tracing::span=warn")
                        .default_write_style_or("never"),
                )
                .format_timestamp_micros()
                .init();

                #config_init

                ::lambda_appsync::lambda_runtime::run(::lambda_appsync::lambda_runtime::service_fn(function_handler)).await
            }
        });
    }
}

impl ToTokens for AppsyncLambdaMain {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if self.options.appsync_types {
            self.graphql_schema.appsync_types_to_tokens(tokens);
        }
        if self.options.appsync_operations {
            self.graphql_schema.appsync_operations_to_tokens(tokens);
        }
        if self.options.lambda_handler {
            self.appsync_event_handler(tokens);
            self.lambda_main(tokens);
        }
    }
}

pub(crate) fn appsync_lambda_main_impl(input: TokenStream) -> TokenStream {
    let alm = parse_macro_input!(input as AppsyncLambdaMain);
    quote! {
        #alm
    }
    .into()
}
