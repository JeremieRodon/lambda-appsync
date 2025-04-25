use std::collections::HashMap;

use syn::ext::IdentExt;

// Captures type_override = Type.field: CustomType and Type.field.param: CustomType options
// using a HashMap hierarchy of TypeName -> FieldName -> (Optional field override, Map of arg overrides)

// Top level mapping from GraphQL type names to their field overrides
pub(super) type TypeOverrides = HashMap<TypeName, FieldOverrides>;
pub(super) type TypeName = String;

// For each type, maps field names to their overrides
pub(super) type FieldOverrides = HashMap<FieldName, FieldOverride>;
pub(super) type FieldName = String;

// A field can have both a direct type override and argument type overrides
// - First element: Optional field type override (Type.field: CustomType)
// - Second element: Map of argument overrides (Type.field.arg: CustomType)
pub(super) type FieldOverride = (FieldTypeOverride, ArgTypeOverrides);
pub(super) type FieldTypeOverride = Option<TypeOverride>;

// Maps argument names to their type overrides for a field
pub(super) type ArgTypeOverrides = HashMap<ArgName, TypeOverride>;
pub(super) type ArgName = String;

pub(super) struct TypeOverride {
    pub(super) type_name: syn::Ident,
    pub(super) field_name: syn::Ident,
    pub(super) arg_name: Option<syn::Ident>,
    pub(super) type_ident: syn::Type,
}
impl TypeOverride {
    pub(super) fn type_name(&self) -> String {
        self.type_name.to_string()
    }
    pub(super) fn field_name(&self) -> String {
        self.field_name.to_string()
    }
    pub(super) fn arg_name(&self) -> Option<String> {
        self.arg_name.as_ref().map(|arg_name| arg_name.to_string())
    }
}
impl syn::parse::Parse for TypeOverride {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let type_name = input.call(syn::Ident::parse_any)?;
        _ = input.parse::<syn::Token![.]>()?;
        let field_name = input.call(syn::Ident::parse_any)?;
        let arg_name = if input.peek(syn::Token![.]) {
            _ = input.parse::<syn::Token![.]>()?;
            Some(input.call(syn::Ident::parse_any)?)
        } else {
            None
        };
        _ = input.parse::<syn::Token![:]>()?;
        let type_ident = input
            .parse()
            .map_err(|e| syn::Error::new(e.span(), "Expected a Type (struct, enum, etc...)"))?;
        Ok(Self {
            type_name,
            field_name,
            arg_name,
            type_ident,
        })
    }
}
