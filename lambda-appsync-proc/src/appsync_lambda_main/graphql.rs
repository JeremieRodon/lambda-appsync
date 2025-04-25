use std::cell::RefCell;

use graphql_parser::schema::{Definition, Document, TypeDefinition};
use proc_macro2::Span;
use quote::{quote_spanned, ToTokens};

use crate::common::{Name, OperationKind};

use super::overrides::FieldOverrides;

thread_local! {
    static CURRENT_SPAN: RefCell<Span> = RefCell::new(Span::call_site());
}
// Get the current span
fn current_span() -> Span {
    CURRENT_SPAN.with(|s| *s.borrow())
}

enum Scalar {
    String,
    ID,
    Int,
    Float,
    Boolean,
    AWSEmail,
    AWSPhone,
    AWSTimestamp,
    AWSDate,
    AWSTime,
    AWSDateTime,
    #[allow(clippy::upper_case_acronyms)]
    AWSJSON,
    #[allow(clippy::upper_case_acronyms)]
    AWSURL,
    AWSIPAddress,
}
impl TryFrom<&str> for Scalar {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "String" => Ok(Self::String),
            "ID" => Ok(Self::ID),
            "Int" => Ok(Self::Int),
            "Float" => Ok(Self::Float),
            "Boolean" => Ok(Self::Boolean),
            "AWSEmail" => Ok(Self::AWSEmail),
            "AWSPhone" => Ok(Self::AWSPhone),
            "AWSTimestamp" => Ok(Self::AWSTimestamp),
            "AWSDate" => Ok(Self::AWSDate),
            "AWSTime" => Ok(Self::AWSTime),
            "AWSDateTime" => Ok(Self::AWSDateTime),
            "AWSJSON" => Ok(Self::AWSJSON),
            "AWSURL" => Ok(Self::AWSURL),
            "AWSIPAddress" => Ok(Self::AWSIPAddress),
            _ => Err(()),
        }
    }
}
impl ToTokens for Scalar {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let span = current_span();
        tokens.extend(match self {
            Scalar::String => quote_spanned! {span=>String},
            Scalar::ID => quote_spanned! {span=>::lambda_appsync::ID},
            Scalar::Int => quote_spanned! {span=>i32},
            Scalar::Float => quote_spanned! {span=>f64},
            Scalar::Boolean => quote_spanned! {span=>bool},
            Scalar::AWSEmail => quote_spanned! {span=>::lambda_appsync::AWSEmail},
            Scalar::AWSPhone => quote_spanned! {span=>::lambda_appsync::AWSPhone},
            Scalar::AWSTimestamp => quote_spanned! {span=>::lambda_appsync::AWSTimestamp},
            Scalar::AWSDate => quote_spanned! {span=>::lambda_appsync::AWSDate},
            Scalar::AWSTime => quote_spanned! {span=>::lambda_appsync::AWSTime},
            Scalar::AWSDateTime => quote_spanned! {span=>::lambda_appsync::AWSDateTime},
            Scalar::AWSJSON => quote_spanned! {span=>::lambda_appsync::serde_json::Value},
            Scalar::AWSURL => quote_spanned! {span=>::lambda_appsync::AWSUrl},
            Scalar::AWSIPAddress => quote_spanned! {span=>::core::net::IpAddr},
        })
    }
}

enum FieldType {
    Overriden(syn::Type),
    Custom { name: Name },
    Scalar(Scalar),
    List(Box<FieldType>),
    Optionnal(Box<FieldType>),
}
impl FieldType {
    fn from_string(name: String) -> Self {
        if let Ok(scalar) = Scalar::try_from(name.as_str()) {
            Self::Scalar(scalar)
        } else {
            let name = Name::from((name, current_span()));
            Self::Custom { name }
        }
    }
    fn is_optionnal(&self) -> bool {
        matches!(self, FieldType::Optionnal(_))
    }
    fn override_type(&mut self, ty: syn::Type) {
        match self {
            FieldType::Overriden(_) | FieldType::Custom { .. } | FieldType::Scalar(_) => {
                *self = FieldType::Overriden(ty)
            }
            FieldType::List(field_type) => field_type.override_type(ty),
            FieldType::Optionnal(field_type) => field_type.override_type(ty),
        }
    }
}
impl From<graphql_parser::schema::Type<'_, String>> for FieldType {
    fn from(value: graphql_parser::schema::Type<'_, String>) -> Self {
        match value {
            graphql_parser::query::Type::NamedType(name) => {
                Self::Optionnal(Box::new(FieldType::from_string(name)))
            }
            graphql_parser::query::Type::ListType(inner) => {
                Self::Optionnal(Box::new(Self::List(Box::new(FieldType::from(*inner)))))
            }
            graphql_parser::query::Type::NonNullType(inner) => {
                let inner = *inner;
                match inner {
                    graphql_parser::query::Type::NamedType(name) => FieldType::from_string(name),
                    graphql_parser::query::Type::ListType(inner) => {
                        Self::List(Box::new(FieldType::from(*inner)))
                    }
                    graphql_parser::query::Type::NonNullType(_) => {
                        unreachable!("Double NonNullType is not supported")
                    }
                }
            }
        }
    }
}
impl ToTokens for FieldType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let span = current_span();
        match self {
            FieldType::Custom { name } => {
                let name = name.to_type_ident();
                tokens.extend(quote_spanned! {span=>#name})
            }
            FieldType::Scalar(scalar) => tokens.extend(quote_spanned! {span=>#scalar}),
            FieldType::List(field_type) => tokens.extend(quote_spanned! {span=>Vec<#field_type>}),
            FieldType::Optionnal(field_type) => {
                tokens.extend(quote_spanned! {span=>::core::option::Option<#field_type>})
            }
            FieldType::Overriden(ty) => tokens.extend(quote_spanned! {span=>#ty}),
        }
    }
}

struct Field {
    name: Name,
    field_type: FieldType,
}
impl From<graphql_parser::schema::Field<'_, String>> for Field {
    fn from(value: graphql_parser::schema::Field<'_, String>) -> Self {
        let name = Name::from((value.name, current_span()));
        let field_type = FieldType::from(value.field_type);
        Self { name, field_type }
    }
}
impl From<graphql_parser::schema::InputValue<'_, String>> for Field {
    fn from(value: graphql_parser::schema::InputValue<'_, String>) -> Self {
        let name = Name::from((value.name, current_span()));
        let field_type = FieldType::from(value.value_type);
        Self { name, field_type }
    }
}

struct FieldContext<'a> {
    field: &'a Field,
    deserialize_only: bool,
}
impl<'a> FieldContext<'a> {
    fn new(field: &'a Field, deserialize_only: bool) -> Self {
        Self {
            field,
            deserialize_only,
        }
    }
}
impl ToTokens for FieldContext<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let field = self.field;
        let name = field.name.to_var_ident();
        // If the `name` identifier is different from the original name, we must serde_rename the type
        let orig_name = field.name.orig();

        let field_type = &field.field_type;
        let mut serde_options = vec![];
        let span = current_span();
        if name != orig_name {
            serde_options.push(quote_spanned! {span=>
                rename = #orig_name
            });
        }
        if field_type.is_optionnal() {
            serde_options.push(quote_spanned! {span=>
                default
            });
            if !self.deserialize_only {
                serde_options.push(quote_spanned! {span=>
                    skip_serializing_if = "Option::is_none"
                });
            }
        }
        if !serde_options.is_empty() {
            tokens.extend(quote_spanned! {span=>
                #[serde(#(#serde_options),*)]
            })
        }
        tokens.extend(quote_spanned! {span=>
            pub #name: #field_type
        });
    }
}

struct Structure {
    name: Name,
    fields: Vec<Field>,
    deserialize_only: bool,
}
impl Structure {
    fn apply_field_overrides(
        &mut self,
        mut field_overrides: FieldOverrides,
    ) -> Result<(), syn::Error> {
        let mut errors = vec![];
        for field in self.fields.iter_mut() {
            let field_name = field.name.orig();
            if let Some((field_override, args_override)) = field_overrides.remove(field_name) {
                if !args_override.is_empty() {
                    errors.extend(args_override.into_values().flat_map(|to| {
                        syn::Error::new(
                            to.arg_name.expect("always set in args_override").span(),
                            "Using args overrides is only supported on operations",
                        )
                    }));
                }
                if let Some(field_override) = field_override {
                    field.field_type.override_type(field_override.type_ident);
                }
            }
        }
        if !field_overrides.is_empty() {
            errors.extend(
                field_overrides
                    .into_values()
                    .flat_map(|fo| fo.0.into_iter().chain(fo.1.into_values()))
                    .map(|to| {
                        syn::Error::new(
                            to.field_name.span(),
                            format!("No field `{}` in `{}`", to.field_name, to.type_name),
                        )
                    }),
            );
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors
                .into_iter()
                .reduce(|mut acc, e| {
                    acc.combine(e);
                    acc
                })
                .expect("not empty"))
        }
    }
}
impl From<graphql_parser::schema::ObjectType<'_, String>> for Structure {
    fn from(value: graphql_parser::schema::ObjectType<'_, String>) -> Self {
        let name = Name::from((value.name, current_span()));
        let fields = value.fields.into_iter().map(Field::from).collect();
        Self {
            name,
            fields,
            deserialize_only: false,
        }
    }
}
impl From<graphql_parser::schema::InputObjectType<'_, String>> for Structure {
    fn from(value: graphql_parser::schema::InputObjectType<'_, String>) -> Self {
        let name = Name::from(value.name);
        let fields = value.fields.into_iter().map(Field::from).collect();
        Self {
            name,
            fields,
            deserialize_only: true,
        }
    }
}
impl ToTokens for Structure {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let span = current_span();
        let struct_name = self.name.to_type_ident();
        let fields = self
            .fields
            .iter()
            .map(|f| FieldContext::new(f, self.deserialize_only));
        let serde_derive = if self.deserialize_only {
            quote_spanned! {span=>::lambda_appsync::serde::Deserialize}
        } else {
            quote_spanned! {span=>::lambda_appsync::serde::Serialize, ::lambda_appsync::serde::Deserialize}
        };
        tokens.extend(quote_spanned! {span=>
            #[derive(Debug, Clone, #serde_derive)]
            pub struct #struct_name {
                #(#fields,)*
            }
        });
    }
}

#[derive(Debug)]
struct Enum {
    name: Name,
    variants: Vec<Name>,
}
impl From<graphql_parser::schema::EnumType<'_, String>> for Enum {
    fn from(value: graphql_parser::schema::EnumType<'_, String>) -> Self {
        let name = Name::from((value.name, current_span()));
        let variants = value
            .values
            .into_iter()
            .map(|v| Name::from((v.name, current_span())))
            .collect();
        Self { name, variants }
    }
}
impl ToTokens for Enum {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let enum_name = self.name.to_type_ident();
        let count = proc_macro2::Literal::usize_unsuffixed(self.variants.len());
        let variant_orig_iter = self.variants.iter().map(|n| n.orig()).collect::<Vec<_>>();
        let variants = self
            .variants
            .iter()
            .map(|n| n.to_type_ident())
            .collect::<Vec<_>>();
        let error_message = format!("`{{}}` is an invalid value for enum {}", enum_name);
        let span = current_span();
        tokens.extend(quote_spanned! {span=>
            #[derive(Debug, Clone, Copy, ::lambda_appsync::serde::Serialize, ::lambda_appsync::serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub enum #enum_name {
                #(#[serde(rename = #variant_orig_iter)]#variants,)*
            }
            impl #enum_name {
                pub const COUNT: usize = #count;
                pub fn all() -> [Self; Self::COUNT] {
                    [#(Self::#variants,)*]
                }
            }
            impl ::core::fmt::Display for #enum_name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #(Self::#variants => write!(f, #variant_orig_iter),)*
                    }
                }
            }
            impl ::core::str::FromStr for #enum_name {
                type Err = ::lambda_appsync::AppsyncError;

                fn from_str(s: &str) -> ::core::result::Result<Self, Self::Err> {
                    match s {
                        #(#variant_orig_iter => ::core::result::Result::Ok(Self::#variants),)*
                        _ => ::core::result::Result::Err(::lambda_appsync::AppsyncError::new(
                            "InvalidStr",
                            format!(#error_message, s),
                        ))
                    }
                }
            }
        });
    }
}

struct UnusedParam<'a>(&'a Field);
impl ToTokens for UnusedParam<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.0.name.to_unused_param_ident();
        let field_type = &self.0.field_type;
        let span = current_span();
        tokens.extend(quote_spanned! {span=>
            #name: #field_type
        });
    }
}

struct Operation {
    name: Name,
    args: Vec<Field>,
    return_type: FieldType,
}
impl Operation {
    fn variant(&self) -> proc_macro2::Ident {
        self.name.to_type_ident()
    }
    fn default_op(&self, kind: OperationKind) -> proc_macro2::TokenStream {
        let fct_name = self.name.to_prefixed_fct_ident(kind.fct_prefix());
        let params = self.args.iter().map(UnusedParam);
        let span = current_span();
        let return_type = match kind {
            OperationKind::Query | OperationKind::Mutation => {
                let return_type = &self.return_type;
                quote_spanned! {span=>#return_type}
            }
            OperationKind::Subscription => quote_spanned! {span=>
                ::core::option::Option<::lambda_appsync::subscription_filters::FilterGroup>
            },
        };
        let default_body = match kind {
            OperationKind::Query | OperationKind::Mutation => {
                let unimplemented_message =
                    format!("{kind} `{}` is unimplemented", self.name.orig());
                quote_spanned! {span=>
                    ::core::result::Result::Err(::lambda_appsync::AppsyncError::new(
                        "Unimplemented",
                        #unimplemented_message,
                    ))
                }
            }
            OperationKind::Subscription => quote_spanned! {span=>
                ::core::result::Result::Ok(None)
            },
        };
        quote_spanned! {span=>
            async fn #fct_name(#(#params,)* _event: &::lambda_appsync::AppsyncEvent<Operation>) -> ::core::result::Result<#return_type, ::lambda_appsync::AppsyncError> {
                #default_body
            }
        }
    }
    fn execute_match_arm(&self, kind: OperationKind) -> proc_macro2::TokenStream {
        let span = current_span();
        let operation_enum_name = kind.operation_enum_name(span);
        let variant = self.name.to_type_ident();
        let fct_name = self.name.to_prefixed_fct_ident(kind.fct_prefix());
        let param_strs = self.args.iter().map(|f| f.name.orig());
        quote_spanned! {span=>
            #operation_enum_name::#variant => Operation::#fct_name(
                #(::lambda_appsync::arg_from_json(&mut args, #param_strs)?,)*
                event
            )
            .await
            .map(::lambda_appsync::res_to_json)
        }
    }
    fn apply_field_override(
        &mut self,
        (field_type_override, mut arg_type_overrides): super::overrides::FieldOverride,
    ) -> Result<(), syn::Error> {
        let mut errors = vec![];
        if let Some(field_type_override) = field_type_override {
            self.return_type
                .override_type(field_type_override.type_ident);
        }
        for arg in self.args.iter_mut() {
            let arg_name = arg.name.orig();
            if let Some(arg_type_override) = arg_type_overrides.remove(arg_name) {
                arg.field_type.override_type(arg_type_override.type_ident);
            }
        }
        if !arg_type_overrides.is_empty() {
            errors.extend(arg_type_overrides.into_values().map(|to| {
                let arg_name = to.arg_name.expect("always set in arg_type_overrides");
                syn::Error::new(
                    arg_name.span(),
                    format!(
                        "No argument `{}` in operation `{}::{}`",
                        arg_name, to.type_name, to.field_name,
                    ),
                )
            }));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors
                .into_iter()
                .reduce(|mut acc, e| {
                    acc.combine(e);
                    acc
                })
                .expect("not empty"))
        }
    }
}
impl From<graphql_parser::schema::Field<'_, String>> for Operation {
    fn from(value: graphql_parser::schema::Field<'_, String>) -> Self {
        let name = Name::from(value.name);
        let args = value.arguments.into_iter().map(Field::from).collect();
        let return_type = FieldType::from(value.field_type);
        Self {
            name,
            args,
            return_type,
        }
    }
}

#[derive(Default)]
struct Operations(Vec<Operation>);
impl From<graphql_parser::schema::ObjectType<'_, String>> for Operations {
    fn from(value: graphql_parser::schema::ObjectType<'_, String>) -> Self {
        Self(value.fields.into_iter().map(Operation::from).collect())
    }
}
impl Operations {
    fn variants_iter(&self) -> impl Iterator<Item = proc_macro2::Ident> + '_ {
        self.0.iter().map(Operation::variant)
    }
    fn default_op_iter(
        &self,
        kind: OperationKind,
    ) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
        self.0.iter().map(move |op| op.default_op(kind))
    }
    fn execute_match_arm_iter(
        &self,
        kind: OperationKind,
    ) -> impl Iterator<Item = proc_macro2::TokenStream> + '_ {
        self.0.iter().map(move |op| op.execute_match_arm(kind))
    }
    fn apply_field_overrides(
        &mut self,
        mut field_overrides: super::overrides::FieldOverrides,
    ) -> Result<(), syn::Error> {
        let mut errors = vec![];
        for op in self.0.iter_mut() {
            let op_name = op.name.orig();
            if let Some(field_override) = field_overrides.remove(op_name) {
                match op.apply_field_override(field_override) {
                    Ok(_) => (),
                    Err(e) => errors.push(e),
                };
            }
        }
        if !field_overrides.is_empty() {
            errors.extend(
                field_overrides
                    .into_values()
                    .flat_map(|fo| fo.0.into_iter().chain(fo.1.into_values()))
                    .map(|to| {
                        syn::Error::new(
                            to.field_name.span(),
                            format!("No operation `{}` in `{}`", to.field_name, to.type_name),
                        )
                    }),
            );
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors
                .into_iter()
                .reduce(|mut acc, e| {
                    acc.combine(e);
                    acc
                })
                .expect("not empty"))
        }
    }
}

#[derive(Debug)]
struct SchemaDefinition {
    query: String,
    mutation: String,
    subscription: String,
}
impl SchemaDefinition {
    fn schema_definition(&self, name: &str) -> Option<OperationKind> {
        if name == self.query {
            Some(OperationKind::Query)
        } else if name == self.mutation {
            Some(OperationKind::Mutation)
        } else if name == self.subscription {
            Some(OperationKind::Subscription)
        } else {
            None
        }
    }
}
impl Default for SchemaDefinition {
    fn default() -> Self {
        Self {
            query: "Query".to_owned(),
            mutation: "Mutation".to_owned(),
            subscription: "Subscription".to_owned(),
        }
    }
}
impl From<graphql_parser::schema::SchemaDefinition<'_, String>> for SchemaDefinition {
    fn from(value: graphql_parser::schema::SchemaDefinition<'_, String>) -> Self {
        let mut sd = Self::default();
        if let Some(query) = value.query {
            sd.query = query
        }
        if let Some(mutation) = value.mutation {
            sd.mutation = mutation
        }
        if let Some(subscription) = value.subscription {
            sd.subscription = subscription
        }
        sd
    }
}

pub(crate) struct GraphQLSchema {
    queries: Operations,
    mutations: Operations,
    subscriptions: Operations,
    structures: Vec<Structure>,
    enums: Vec<Enum>,
}

impl GraphQLSchema {
    pub(crate) fn new(
        mut doc: Document<'_, String>,
        span: proc_macro2::Span,
        mut tos: super::TypeOverrides,
    ) -> Result<Self, syn::Error> {
        let mut queries = None;
        let mut mutations = None;
        let mut subscriptions = None;
        let mut structures = vec![];
        let mut enums = vec![];

        CURRENT_SPAN.replace(span);

        let sd = if let Some(index) = doc
            .definitions
            .iter()
            .position(|def| matches!(def, Definition::SchemaDefinition(_)))
        {
            let Definition::SchemaDefinition(def) = doc.definitions.swap_remove(index) else {
                unreachable!("just verified it is a schema def")
            };
            SchemaDefinition::from(def)
        } else {
            SchemaDefinition::default()
        };

        let mut errors = vec![];
        for def in doc.definitions {
            match def {
                Definition::TypeDefinition(type_definition) => {
                    match type_definition {
                        TypeDefinition::Object(object_type) => {
                            if let Some(sdt) = sd.schema_definition(&object_type.name) {
                                let field_overrides = tos.remove(&object_type.name);
                                let mut ops = Operations::from(object_type);
                                if let Some(field_overrides) = field_overrides {
                                    match ops.apply_field_overrides(field_overrides) {
                                        Ok(_) => (),
                                        Err(e) => errors.push(e),
                                    };
                                }
                                match sdt {
                                    OperationKind::Query => {
                                        queries.replace(ops);
                                    }
                                    OperationKind::Mutation => {
                                        mutations.replace(ops);
                                    }
                                    OperationKind::Subscription => {
                                        subscriptions.replace(ops);
                                    }
                                }
                            } else {
                                let mut structure = Structure::from(object_type);
                                if let Some(field_overrides) = tos.remove(structure.name.orig()) {
                                    match structure.apply_field_overrides(field_overrides) {
                                        Ok(_) => (),
                                        Err(e) => errors.push(e),
                                    };
                                }
                                structures.push(structure);
                            }
                        }
                        TypeDefinition::Enum(enum_type) => {
                            enums.push(Enum::from(enum_type));
                        }
                        TypeDefinition::InputObject(input_object_type) => {
                            let mut structure = Structure::from(input_object_type);
                            if let Some(field_overrides) = tos.remove(structure.name.orig()) {
                                match structure.apply_field_overrides(field_overrides) {
                                    Ok(_) => (),
                                    Err(e) => errors.push(e),
                                };
                            }
                            structures.push(structure);
                        }
                        // Not yet implemented, ignored for now
                        TypeDefinition::Scalar(_) => {}
                        TypeDefinition::Interface(_) => (),
                        TypeDefinition::Union(_) => (),
                    }
                }
                // Already processed
                Definition::SchemaDefinition(_) => {
                    return Err(syn::Error::new(
                        span,
                        "GraphQL schema file has two `schema` definition",
                    ));
                }
                // Ignored for now
                Definition::TypeExtension(_) => (),
                Definition::DirectiveDefinition(_) => (),
            }
        }

        if !tos.is_empty() {
            errors.extend(
                tos.into_values()
                    .flat_map(|fos| fos.into_values())
                    .flat_map(|fo| fo.0.into_iter().chain(fo.1.into_values()))
                    .map(|to| {
                        syn::Error::new(
                            to.type_name.span(),
                            format!("No type or input named `{}`", to.type_name),
                        )
                    }),
            );
        }
        if errors.is_empty() {
            Ok(Self {
                queries: queries.unwrap_or_default(),
                mutations: mutations.unwrap_or_default(),
                subscriptions: subscriptions.unwrap_or_default(),
                structures,
                enums,
            })
        } else {
            Err(errors
                .into_iter()
                .reduce(|mut acc, e| {
                    acc.combine(e);
                    acc
                })
                .expect("not empty"))
        }
    }
    fn enums_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let enums = self.enums.iter();
        let span = current_span();
        tokens.extend(quote_spanned! {span=>
            #(#enums)*
        });
    }
    fn structs_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let structures = self.structures.iter();
        let span = current_span();
        tokens.extend(quote_spanned! {span=>
            #(#structures)*
        });
    }
    fn operation_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let span = current_span();
        let query_field_name = OperationKind::Query.operation_enum_name(span);
        let query_field_variants = self.queries.variants_iter();
        let mutation_field_name = OperationKind::Mutation.operation_enum_name(span);
        let mutation_field_variants = self.mutations.variants_iter();
        let subscription_field_name = OperationKind::Subscription.operation_enum_name(span);
        let subscription_field_variants = self.subscriptions.variants_iter();
        tokens.extend(quote_spanned! {span=>
            #[derive(Debug, Clone, Copy, ::lambda_appsync::serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub enum #query_field_name {
                #(#query_field_variants,)*
            }
            #[derive(Debug, Clone, Copy, ::lambda_appsync::serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub enum #mutation_field_name {
                #(#mutation_field_variants,)*
            }
            #[derive(Debug, Clone, Copy, ::lambda_appsync::serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub enum #subscription_field_name {
                #(#subscription_field_variants,)*
            }
            #[derive(Debug, Clone, Copy, ::lambda_appsync::serde::Deserialize)]
            #[serde(tag = "parentTypeName", content = "fieldName")]
            pub enum Operation {
                Query(#query_field_name),
                Mutation(#mutation_field_name),
                Subscription(#subscription_field_name),
            }
        });
    }
    fn default_operations_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let query_field_default_ops = self.queries.default_op_iter(OperationKind::Query);
        let mutation_field_default_ops = self.mutations.default_op_iter(OperationKind::Mutation);
        let subscription_field_default_ops = self
            .subscriptions
            .default_op_iter(OperationKind::Subscription);
        let span = current_span();
        tokens.extend(quote_spanned! {span=>
            #[allow(dead_code)]
            trait DefautOperations {
                #(#query_field_default_ops)*
                #(#mutation_field_default_ops)*
                #(#subscription_field_default_ops)*
            }
            impl DefautOperations for Operation {}
        });
    }
    fn impl_operation_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let query_field_execute_match_arm =
            self.queries.execute_match_arm_iter(OperationKind::Query);
        let mutation_field_execute_match_arm = self
            .mutations
            .execute_match_arm_iter(OperationKind::Mutation);
        let subscription_field_execute_match_arm = self
            .subscriptions
            .execute_match_arm_iter(OperationKind::Subscription);
        let span = current_span();
        tokens.extend(quote_spanned! {span=>
            impl Operation {
                pub async fn execute(self,
                    args: ::lambda_appsync::serde_json::Value,
                    event: &::lambda_appsync::AppsyncEvent<Operation>
                ) -> ::lambda_appsync::AppsyncResponse {
                    match self._execute(args, event).await {
                        ::core::result::Result::Ok(v) => v.into(),
                        ::core::result::Result::Err(e) => {
                            ::lambda_appsync::log::error!("{e}");
                            e.into()
                        }
                    }
                }
                async fn _execute(
                    self,
                    mut args: ::lambda_appsync::serde_json::Value,
                    event: &::lambda_appsync::AppsyncEvent<Operation>
                ) -> ::core::result::Result<::lambda_appsync::serde_json::Value, ::lambda_appsync::AppsyncError> {
                    match self {
                        Operation::Query(query_field) => match query_field {
                            #(#query_field_execute_match_arm,)*
                        },
                        Operation::Mutation(mutation_field) => match mutation_field {
                            #(#mutation_field_execute_match_arm,)*
                        },
                        Operation::Subscription(subscription_field) => match subscription_field {
                            #(#subscription_field_execute_match_arm,)*
                        },
                    }
                }
            }
        });
    }
    pub(crate) fn appsync_types_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.enums_to_tokens(tokens);
        self.structs_to_tokens(tokens);
    }
    pub(crate) fn appsync_operations_to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.operation_to_tokens(tokens);
        self.default_operations_to_tokens(tokens);
        self.impl_operation_to_tokens(tokens);
    }
}
