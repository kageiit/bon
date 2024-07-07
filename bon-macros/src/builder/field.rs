use darling::util::{Flag, SpannedValue};
use darling::FromAttributes;
use prox::prelude::*;
use quote::quote;
use syn::spanned::Spanned;

pub(crate) struct Field {
    /// Original name of the field is used as the name of the builder field and
    /// in its setter methods. Field names conventionally use snake_case in Rust,
    /// but this isn't enforced, so this field isn't guaranteed to be in snake case,
    /// but 99% of the time it will be.
    pub(crate) ident: syn::Ident,

    /// Doc comments for the setter methods are copied from the doc comments placed
    /// on top of individual arguments in the original function. Yes, doc comments
    /// are not valid on function arguments in regular Rust, but they are valid if
    /// a proc macro like this one pre-processes them and removes them from the
    /// expanded code.
    pub(crate) docs: Vec<syn::Attribute>,

    /// Type of the function argument that corresponds to this field. This is the
    /// resulting type that the builder should generate setters for.
    pub(crate) ty: Box<syn::Type>,

    /// The name of the associated type in the builder state trait that corresponds
    /// to this field.
    pub(crate) state_assoc_type_ident: syn::Ident,

    /// Parameters configured by the user explicitly via attributes
    pub(crate) params: FieldParams,
}

#[derive(darling::FromAttributes)]
#[darling(attributes(builder))]
pub(crate) struct FieldParams {
    /// Overrides the decision to use `Into` for the setter method.
    pub(crate) into: Option<bool>,

    #[darling(with = "parse_optional_expression", map = "Some")]
    pub(crate) default: Option<SpannedValue<Option<syn::Expr>>>,

    /// Makes the field required no matter what default treatment for such field
    /// is applied.
    pub(crate) required: Option<Flag>,
}

fn parse_optional_expression(meta: &syn::Meta) -> Result<SpannedValue<Option<syn::Expr>>> {
    match meta {
        syn::Meta::Path(_) => Ok(SpannedValue::new(None, meta.span())),
        syn::Meta::List(_) => Err(Error::unsupported_format("list").with_span(meta)),
        syn::Meta::NameValue(nv) => Ok(SpannedValue::new(Some(nv.value.clone()), nv.span())),
    }
}

impl Field {
    pub(crate) fn from_typed_fn_arg(arg: &syn::PatType) -> Result<Self> {
        let syn::Pat::Ident(pat) = arg.pat.as_ref() else {
            // We may allow setting a name for the builder method in parameter
            // attributes and relax this requirement
            prox::bail!(
                &arg.pat,
                "Only simple identifiers in function arguments supported \
                to infer the name of builder methods"
            );
        };

        let docs = arg
            .attrs
            .iter()
            .filter(|attr| attr.is_doc())
            .cloned()
            .collect();

        let params = FieldParams::from_attributes(&arg.attrs)?;

        let me = Self {
            state_assoc_type_ident: pat.ident.to_pascal_case(),
            ident: pat.ident.clone(),
            ty: arg.ty.clone(),
            params,
            docs,
        };

        me.validate()?;

        Ok(me)
    }

    fn validate(&self) -> Result<()> {
        if let Some(default) = &self.params.default {
            if self.ty.is_option() {
                prox::bail!(
                    &default.span(),
                    "`Option` and #[builder(default)] attributes are mutually exclusive"
                );
            }
        }

        Ok(())
    }

    pub(crate) fn as_optional(&self) -> Option<&syn::Type> {
        self.ty
            .option_type_param()
            .or_else(|| (self.ty.is_bool() || self.params.default.is_some()).then_some(&self.ty))
    }

    pub(crate) fn unset_state_type(&self) -> TokenStream2 {
        let ty = &self.ty;

        if let Some(inner_type) = self.as_optional() {
            quote!(bon::private::Optional<#inner_type>)
        } else {
            quote!(bon::private::Required<#ty>)
        }
    }

    pub(crate) fn set_state_type(&self) -> TokenStream2 {
        let ty = &self.ty;

        let ty = self
            .as_optional()
            .map(|ty| quote!(Option<#ty>))
            .unwrap_or_else(|| quote!(#ty));

        quote!(bon::private::Set<#ty>)
    }
}
