use super::{
    resolve_type,
    structs::{Field, StructOptions},
    GenericArgument, Type,
};
use crate::{casing::Casing, docs::get_doc_lines};
use quote::ToTokens;
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryFrom,
};
use syn::{
    ext::IdentExt, parenthesized, parse::Parse, parse::ParseStream, Attribute, Error, GenericParam,
    Ident, ItemEnum, LitStr, Result, Token,
};

pub(crate) fn parse_enum_item(item: ItemEnum, dependencies: &BTreeSet<Type>) -> Type {
    let name = item.ident.to_string();
    let generic_args = item
        .generics
        .params
        .iter()
        .filter_map(|param| match param {
            GenericParam::Type(ty) => Some(GenericArgument {
                name: ty.ident.to_string(),
                ty: None,
            }),
            _ => None,
        })
        .collect();
    let doc_lines = get_doc_lines(&item.attrs);
    let variants = item
        .variants
        .iter()
        .map(|variant| {
            if variant.discriminant.is_some() {
                panic!(
                    "Discriminants in enum variants are not supported. Found: {:?}",
                    item
                );
            }

            let name = variant.ident.to_string();
            let ty = if variant.fields.is_empty() {
                Type::Unit
            } else if variant.fields.iter().any(|field| field.ident.is_some()) {
                let fields = variant
                    .fields
                    .iter()
                    .map(|field| {
                        let name = field
                            .ident
                            .as_ref()
                            .expect("Expected all enum variant fields to be named")
                            .to_string();
                        let ty = resolve_type(&field.ty, dependencies).unwrap_or_else(|| {
                            panic!("Unresolvable variant field type: {:?}", field.ty)
                        });
                        let doc_lines = get_doc_lines(&field.attrs);
                        Field {
                            name,
                            ty,
                            doc_lines,
                        }
                    })
                    .collect();
                Type::Struct(
                    name.clone(),
                    vec![],
                    vec![],
                    fields,
                    StructOptions::default(),
                )
            } else {
                let item_types = variant
                    .fields
                    .iter()
                    .map(|field| {
                        resolve_type(&field.ty, dependencies).unwrap_or_else(|| {
                            panic!("Unresolvable variant item type: {:?}", field.ty)
                        })
                    })
                    .collect();
                Type::Tuple(item_types)
            };
            let doc_lines = get_doc_lines(&variant.attrs);

            Variant {
                name,
                ty,
                doc_lines,
            }
        })
        .collect();
    let opts = EnumOptions::from_attrs(&item.attrs);
    Type::Enum(name, generic_args, doc_lines, variants, opts)
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnumOptions {
    pub variant_casing: Casing,
    pub content_prop_name: Option<String>,
    pub tag_prop_name: Option<String>,

    /// If `true`, serialized variants are not tagged, and a match is found
    /// by attempting to deserialize variants in order, where the first one to
    /// successfully deserialize is used as the result.
    pub untagged: bool,

    /// Rust module paths where the type can be found for the given generator.
    /// If present, the generator can use this type instead of generating it.
    ///
    /// ## Example:
    ///
    /// ```rs
    /// #[fp(rust_plugin_module = "my_crate")]
    /// enum MyEnum { /* ... */ }
    /// ```
    ///
    /// This will insert `"rust_plugin" => "my_crate"` into the map, which can
    /// be used by the Rust plugin generator to generate a `use` statement such
    /// as:
    ///
    /// ```rs
    /// pub use my_crate::MyEnum;
    /// ```
    ///
    /// Instead of generating the enum definition itself.
    pub native_modules: BTreeMap<String, String>,
}

impl EnumOptions {
    pub fn from_attrs(attrs: &[Attribute]) -> Self {
        let mut opts = Self::default();
        for attr in attrs {
            if attr.path.is_ident("fp") || attr.path.is_ident("serde") {
                opts.merge_with(
                    &syn::parse2::<Self>(attr.tokens.clone()).expect("Could not parse attributes"),
                );
            }
        }
        opts
    }

    fn merge_with(&mut self, other: &EnumOptions) {
        if other.variant_casing != Casing::default() {
            self.variant_casing = other.variant_casing;
        }
        if other.content_prop_name.is_some() {
            self.content_prop_name = other.content_prop_name.clone();
        }
        if other.tag_prop_name.is_some() {
            self.tag_prop_name = other.tag_prop_name.clone();
        }
        if other.untagged {
            self.untagged = true;
        }
        for (key, value) in other.native_modules.iter() {
            self.native_modules.insert(key.clone(), value.clone());
        }
    }

    pub fn to_serde_attrs(&self) -> Vec<String> {
        let mut serde_attrs = vec![];
        if self.untagged {
            serde_attrs.push("untagged".to_owned());
        } else {
            if let Some(prop_name) = &self.tag_prop_name {
                serde_attrs.push(format!("tag = \"{}\"", prop_name));

                if let Some(prop_name) = &self.content_prop_name {
                    serde_attrs.push(format!("content = \"{}\"", prop_name));
                }
            }
            if let Some(casing) = &self.variant_casing.as_maybe_str() {
                serde_attrs.push(format!("rename_all = \"{}\"", casing));
            }
        }
        serde_attrs
    }
}

impl Parse for EnumOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        let mut result = Self::default();
        loop {
            let key: Ident = content.call(IdentExt::parse_any)?;
            match &*key.to_string() {
                "content" => {
                    content.parse::<Token![=]>()?;
                    result.content_prop_name = Some(
                        content
                            .parse::<LitStr>()?
                            .to_token_stream()
                            .to_string()
                            .trim_matches('"')
                            .to_owned(),
                    );
                }
                "tag" => {
                    content.parse::<Token![=]>()?;
                    result.tag_prop_name = Some(
                        content
                            .parse::<LitStr>()?
                            .to_token_stream()
                            .to_string()
                            .trim_matches('"')
                            .to_owned(),
                    );
                }
                "rename_all" => {
                    content.parse::<Token![=]>()?;
                    result.variant_casing = Casing::try_from(
                        content
                            .parse::<LitStr>()?
                            .to_token_stream()
                            .to_string()
                            .trim_matches('"'),
                    )
                    .map_err(|err| Error::new(content.span(), err))?;
                }
                "untagged" => {
                    result.untagged = true;
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
pub struct Variant {
    pub name: String,
    pub ty: Type,
    pub doc_lines: Vec<String>,
}
