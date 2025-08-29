mod graphql;
mod overrides;

use std::collections::HashMap;

use graphql::GraphQLSchema;
use overrides::{NameOverride, TypeOverride};
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

// I suppose this is acceptable for a proc-macro
#[allow(clippy::large_enum_variant)]
enum OptionalParameter {
    Batch(bool),
    ExcludeLambdaHandler(bool),
    OnlyLambdaHandler(bool),
    ExcludeAppsyncTypes(bool),
    OnlyAppsyncTypes(bool),
    ExcludeAppsyncOperations(bool),
    OnlyAppsyncOperations(bool),
    Hook(Ident),
    TypeOverride(TypeOverride),
    NameOverride(NameOverride),
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
            "type_override" => Ok(Self::TypeOverride(input.parse()?)),
            "name_override" => Ok(Self::NameOverride(input.parse()?)),
            // Deprecated options
            "field_type_override" => Ok(Self::TypeOverride(input.parse()?)),
            // Unknown option
            _ => Err(syn::Error::new(
                ident.span(),
                format!("Unknown parameter `{ident}`",),
            )),
        }
    }
}

// Captures type_override = Type.field: CustomType and Type.field.param: CustomType options
// using a HashMap hierarchy of TypeName -> FieldName -> (Optional field override, Map of arg overrides)
// Top level mapping from GraphQL type names to their field overrides
type TypeOverrides = HashMap<TypeName, FieldTypeOverrides>;

// For each type, maps field names to their overrides
type FieldTypeOverrides = HashMap<FieldName, FieldTypeOverride>;

// A field can have both a direct type override and argument type overrides
// - First element: Optional field type override (Type.field: CustomType)
// - Second element: Map of argument overrides (Type.field.arg: CustomType)
type FieldTypeOverride = (Option<TypeOverride>, ArgTypeOverrides);

// Maps argument names to their type overrides for a field
type ArgTypeOverrides = HashMap<ArgName, TypeOverride>;

// Captures name_override = Type: CustomName and Type.field: custom_name options
// using a HashMap hierarchy of TypeName -> (Optional type override, Map of field overrides)
// This works the same for name_override = Enum: CustomEnumName and Enum.VARIANT: CustomVariant
// Top level mapping from GraphQL type names to their field overrides
type NameOverrides = HashMap<TypeName, TypeNameOverride>;

// A type can have both a direct name override and field name overrides
// - First element: Optional type name override (Type: CustomName)
// - Second element: Map of field overrides (Type.field: custom_name)
type TypeNameOverride = (Option<NameOverride>, FieldNameOverrides);

// Maps field names to their name overrides for a field
type FieldNameOverrides = HashMap<FieldName, NameOverride>;

// [Type|Field|Arg]Name are just String
type TypeName = String;
type FieldName = String;
type ArgName = String;

struct OptionalParameters {
    batch: bool,
    appsync_types: bool,
    appsync_operations: bool,
    lambda_handler: bool,
    hook: Option<Ident>,
    tos: TypeOverrides,
    nos: NameOverrides,
}
impl Default for OptionalParameters {
    fn default() -> Self {
        Self {
            batch: true,
            appsync_types: true,
            appsync_operations: true,
            lambda_handler: true,
            hook: None,
            tos: TypeOverrides::new(),
            nos: NameOverrides::new(),
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
            OptionalParameter::TypeOverride(to) => {
                // Retrieve the entry corresponding to `Type.field`
                let to_field_entry = self
                    .tos
                    .entry(to.type_name().to_string())
                    .or_default()
                    .entry(to.field_name().to_string())
                    .or_default();
                if let Some(arg_name) = to.arg_name() {
                    // There is a `.param`
                    // This is a parameter override
                    to_field_entry.1.insert(arg_name.to_string(), to);
                } else {
                    // no `.param`
                    // This is just a field override
                    to_field_entry.0.replace(to);
                }
            }
            OptionalParameter::NameOverride(no) => {
                // Retrieve the entry corresponding to `Type`
                let no_type_entry = self.nos.entry(no.type_name().to_string()).or_default();
                if let Some(field_name) = no.field_name() {
                    // There is a `.field`
                    // This is a field override
                    no_type_entry.1.insert(field_name.to_string(), no);
                } else {
                    // no `.field`
                    // This is just a type override
                    no_type_entry.0.replace(no);
                }
            }
            OptionalParameter::ExcludeLambdaHandler(_)
            | OptionalParameter::OnlyLambdaHandler(_)
            | OptionalParameter::ExcludeAppsyncTypes(_)
            | OptionalParameter::OnlyAppsyncTypes(_)
            | OptionalParameter::ExcludeAppsyncOperations(_)
            | OptionalParameter::OnlyAppsyncOperations(_) => (),
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
        let path_value = graphql_schema_path.value();
        let full_path = if std::path::Path::new(&path_value).is_relative() {
            std::env::current_dir()
                .map_err(|e| {
                    syn::Error::new(
                        graphql_schema_path.span(),
                        format!("Could not get current directory: {e}"),
                    )
                })?
                .join(&path_value)
        } else {
            std::path::PathBuf::from(path_value)
        };
        let schema_str = std::fs::read_to_string(&full_path).map_err(|e| {
            syn::Error::new(
                graphql_schema_path.span(),
                format!(
                    "Could not open GraphQL schema file at '{}' ({e})",
                    full_path.display()
                ),
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

        let graphql_schema = GraphQLSchema::new(
            schema,
            graphql_schema_path.span(),
            std::mem::take(&mut options.tos),
            std::mem::take(&mut options.nos),
        )?;

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
            async fn appsync_handler(event: ::lambda_appsync::AppsyncEvent<Operation>) -> ::lambda_appsync::AppsyncResponse {
                ::lambda_appsync::log::info!("event={event:?}");
                ::lambda_appsync::log::info!("operation={:?}", event.info.operation);

                #call_hook

                event.info.operation.execute(event).await
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
