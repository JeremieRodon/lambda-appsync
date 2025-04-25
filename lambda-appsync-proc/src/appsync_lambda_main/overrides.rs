use syn::ext::IdentExt;

pub(super) struct TypeOverride {
    type_name: syn::Ident,
    field_name: syn::Ident,
    arg_name: Option<syn::Ident>,
    type_ident: syn::Type,
}
impl TypeOverride {
    pub(super) fn type_name(&self) -> &syn::Ident {
        &self.type_name
    }
    pub(super) fn field_name(&self) -> &syn::Ident {
        &self.field_name
    }
    pub(super) fn arg_name(&self) -> Option<&syn::Ident> {
        self.arg_name.as_ref()
    }
    pub(super) fn type_ident(self) -> syn::Type {
        self.type_ident
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

pub(super) struct NameOverride {
    type_name: syn::Ident,
    field_name: Option<syn::Ident>,
    new_name: String,
}
impl NameOverride {
    pub(super) fn type_name(&self) -> &syn::Ident {
        &self.type_name
    }
    pub(super) fn field_name(&self) -> Option<&syn::Ident> {
        self.field_name.as_ref()
    }
    pub(super) fn new_name(self) -> String {
        self.new_name
    }
}
impl syn::parse::Parse for NameOverride {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let type_name = input.call(syn::Ident::parse_any)?;
        let field_name = if input.peek(syn::Token![.]) {
            _ = input.parse::<syn::Token![.]>()?;
            Some(input.call(syn::Ident::parse_any)?)
        } else {
            None
        };
        _ = input.parse::<syn::Token![:]>()?;
        let new_name = input.call(syn::Ident::parse_any)?;
        let new_name = new_name.to_string();
        Ok(Self {
            type_name,
            field_name,
            new_name,
        })
    }
}
