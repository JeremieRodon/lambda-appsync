use proc_macro2::Span;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CaseType {
    Camel,  // Lowercase separated by Uppercase letters, first letter lower
    Pascal, // Lowercase separated by Uppercase letters, first letter uppercase
    Snake,  // Lowercase separated by _
    Upper,  // All uppercase separated by _
}
impl CaseType {
    fn case(s: &str) -> Self {
        let mut chars = s.chars();
        if let Some(c) = chars.next() {
            if c.is_uppercase() {
                // Can only be Pascal or all upper
                for c in chars {
                    if !(c.is_uppercase() || c == '_') {
                        return Self::Pascal;
                    }
                }
                return Self::Upper;
            } else {
                // Can only be Camel or all lower
                for c in chars {
                    if !(c.is_lowercase() || c == '_') {
                        return Self::Camel;
                    }
                }
                return Self::Snake;
            }
        }
        panic!("Empty string has no case");
    }
}

// Word is always stored as lowercase
#[derive(Debug)]
pub(crate) struct Word(String);
impl Word {
    pub(crate) fn capitalize(&self) -> String {
        let mut chars = self.0.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().chain(chars).collect(),
            None => String::new(),
        }
    }
    pub(crate) fn to_uppercase(&self) -> String {
        self.0.to_uppercase()
    }
}
impl std::fmt::Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::borrow::Borrow<str> for Word {
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug)]
pub(crate) struct Name {
    orig: String,
    span: Option<Span>,
    words: Vec<Word>,
}
impl From<String> for Name {
    fn from(value: String) -> Self {
        match CaseType::case(value.as_str()) {
            CaseType::Camel => {
                let mut words = vec![];
                let mut slice_start = 0;
                for (i, c) in value.chars().enumerate() {
                    if c.is_uppercase() {
                        words.push(Word(value[slice_start..i].to_lowercase()));
                        slice_start = i;
                    }
                }
                words.push(Word(value[slice_start..].to_lowercase()));
                Name {
                    orig: value,
                    span: None,
                    words,
                }
            }
            CaseType::Pascal => {
                let mut words = vec![];
                let mut slice_start = 0;
                for (i, c) in value.chars().enumerate() {
                    if c.is_uppercase() && i != 0 {
                        words.push(Word(value[slice_start..i].to_lowercase()));
                        slice_start = i;
                    }
                }
                words.push(Word(value[slice_start..].to_lowercase()));
                Name {
                    orig: value,
                    span: None,
                    words,
                }
            }
            CaseType::Snake => {
                let words = value.split('_').map(|w| Word(w.to_owned())).collect();
                Name {
                    orig: value,
                    span: None,
                    words,
                }
            }
            CaseType::Upper => {
                let words = value.split('_').map(|w| Word(w.to_lowercase())).collect();
                Name {
                    orig: value,
                    span: None,
                    words,
                }
            }
        }
    }
}
impl Name {
    pub(crate) fn orig(&self) -> &str {
        &self.orig
    }
    pub(crate) fn set_span(&mut self, span: Span) {
        self.span = Some(span);
    }
    pub(crate) fn to_case(&self, case: CaseType) -> String {
        match case {
            CaseType::Camel => {
                let mut word_iter = self.words.iter();
                let first = word_iter.next().expect("non empty name");
                format!(
                    "{first}{}",
                    word_iter
                        .map(|w| w.capitalize())
                        .collect::<Vec<_>>()
                        .join("")
                )
            }
            CaseType::Pascal => self
                .words
                .iter()
                .map(|w| w.capitalize())
                .collect::<Vec<_>>()
                .join(""),
            CaseType::Snake => self.words.join("_"),
            CaseType::Upper => self
                .words
                .iter()
                .map(|w| w.to_uppercase())
                .collect::<Vec<_>>()
                .join("_"),
        }
    }
    pub(crate) fn to_type_ident(&self) -> proc_macro2::Ident {
        proc_macro2::Ident::new(
            &self.to_case(CaseType::Pascal),
            self.span.unwrap_or_else(|| Span::call_site()),
        )
    }
    pub(crate) fn to_var_ident(&self) -> proc_macro2::Ident {
        proc_macro2::Ident::new(
            &self.to_case(CaseType::Snake),
            self.span.unwrap_or_else(|| Span::call_site()),
        )
    }
    pub(crate) fn to_unused_param_ident(&self) -> proc_macro2::Ident {
        proc_macro2::Ident::new(
            &format!("_{}", self.to_case(CaseType::Snake)),
            self.span.unwrap_or_else(|| Span::call_site()),
        )
    }
    pub(crate) fn to_prefixed_fct_ident(&self, prefix: &str) -> proc_macro2::Ident {
        proc_macro2::Ident::new(
            &format!("{prefix}_{}", self.to_case(CaseType::Snake)),
            self.span.unwrap_or_else(|| Span::call_site()),
        )
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum OperationKind {
    Query,
    Mutation,
    Subscription,
}
impl core::fmt::Display for OperationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationKind::Query => write!(f, "Query"),
            OperationKind::Mutation => write!(f, "Mutation"),
            OperationKind::Subscription => write!(f, "Subscription"),
        }
    }
}
impl OperationKind {
    pub(crate) fn fct_prefix(self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Mutation => "mutation",
            Self::Subscription => "subscription",
        }
    }
    pub(crate) fn operation_enum_name(self, span: Span) -> proc_macro2::Ident {
        match self {
            Self::Query => proc_macro2::Ident::new("QueryField", span),
            Self::Mutation => proc_macro2::Ident::new("MutationField", span),
            Self::Subscription => proc_macro2::Ident::new("SubscriptionField", span),
        }
    }
}
