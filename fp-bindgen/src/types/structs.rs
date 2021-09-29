use super::{resolve_type, GenericArgument, Type};
use crate::{casing::Casing, docs::get_doc_lines};
use quote::ToTokens;
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
};
use syn::{
    ext::IdentExt, parenthesized, parse::Parse, parse::ParseStream, Attribute, Error, GenericParam,
    Ident, ItemStruct, LitStr, Result, Token,
};

pub(crate) fn parse_struct_item(item: ItemStruct, dependencies: &BTreeSet<Type>) -> Type {
    let name = item.ident.to_string();
    let generic_args = item
        .generics
        .params
        .into_iter()
        .filter_map(|param| match param {
            GenericParam::Type(ty) => Some(GenericArgument {
                name: ty.ident.to_string(),
                ty: None,
            }),
            _ => None,
        })
        .collect();
    let doc_lines = get_doc_lines(&item.attrs);
    let fields = item
        .fields
        .iter()
        .map(|field| {
            let name = field
                .ident
                .as_ref()
                .expect("Struct fields must be named")
                .to_string();
            let ty = resolve_type(&field.ty, dependencies)
                .unwrap_or_else(|| panic!("Unresolvable field type: {:?}", field.ty));
            let doc_lines = get_doc_lines(&field.attrs);
            Field {
                name,
                ty,
                doc_lines,
            }
        })
        .collect();
    let opts = StructOptions::from_attrs(&item.attrs);
    Type::Struct(name, generic_args, doc_lines, fields, opts)
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct StructOptions {
    pub field_casing: Casing,

    /// Rust module paths where the type can be found for the given generator.
    /// If present, the generator can use this type instead of generating it.
    ///
    /// ## Example:
    ///
    /// ```rs
    /// #[fp(rust_plugin_module = "my_crate")]
    /// struct MyStruct { /* ... */ }
    /// ```
    ///
    /// This will insert `"rust_plugin" => "my_crate"` into the map, which can
    /// be used by the Rust plugin generator to generate a `use` statement such
    /// as:
    ///
    /// ```rs
    /// pub use my_crate::MyStruct;
    /// ```
    ///
    /// Instead of generating the struct definition itself.
    pub native_modules: BTreeMap<String, String>,
}

impl StructOptions {
    pub fn from_attrs(attrs: &[Attribute]) -> Self {
        let mut opts = Self::default();
        for attr in attrs {
            if attr.path.is_ident("fp") || attr.path.is_ident("serde") {
                opts.merge_with(
                    &syn::parse2::<Self>(attr.tokens.clone()).expect("Could not parse attributes"),
                );
            }
        }
        if opts.native_modules.is_empty() && opts.field_casing == Casing::default() {
            opts.field_casing = Casing::CamelCase;
        }
        opts
    }

    fn merge_with(&mut self, other: &StructOptions) {
        if other.field_casing != Casing::default() {
            self.field_casing = other.field_casing;
        }
        for (key, value) in other.native_modules.iter() {
            self.native_modules.insert(key.clone(), value.clone());
        }
    }

    pub fn to_serde_attrs(&self) -> Vec<String> {
        let mut serde_attrs = vec![];
        if let Some(casing) = &self.field_casing.as_maybe_str() {
            serde_attrs.push(format!("rename_all = \"{}\"", casing));
        }
        serde_attrs
    }
}

impl Parse for StructOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        let mut result = Self::default();
        loop {
            let key: Ident = content.call(IdentExt::parse_any)?;
            match &*key.to_string() {
                "rename_all" => {
                    content.parse::<Token![=]>()?;
                    result.field_casing = Casing::try_from(
                        content
                            .parse::<LitStr>()?
                            .to_token_stream()
                            .to_string()
                            .trim_matches('"'),
                    )
                    .map_err(|err| Error::new(content.span(), err))?;
                }
                module if module.ends_with("_module") => {
                    content.parse::<Token![=]>()?;
                    let value = content
                        .parse::<LitStr>()?
                        .to_token_stream()
                        .to_string()
                        .trim_matches('"')
                        .to_owned();
                    result
                        .native_modules
                        .insert(module[0..module.len() - 7].to_owned(), value);
                }
                other => {
                    return Err(Error::new(
                        content.span(),
                        format!("Unexpected attribute: {}", other),
                    ))
                }
            }

            if content.is_empty() {
                break;
            }

            content.parse::<Token![,]>()?;
        }

        Ok(result)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub doc_lines: Vec<String>,
}
