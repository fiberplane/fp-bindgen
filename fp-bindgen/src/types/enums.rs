use super::{
    structs::{Field, Struct, StructOptions},
    Type, TypeIdent,
};
use crate::types::format_bounds;
use crate::{casing::Casing, docs::get_doc_lines, primitives::Primitive, types::FieldAttrs};
use quote::ToTokens;
use std::{convert::TryFrom, str::FromStr};
use syn::{
    ext::IdentExt, parenthesized, parse::Parse, parse::ParseStream, Attribute, Error, GenericParam,
    Ident, ItemEnum, LitStr, Result, Token, TypePath,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Enum {
    pub ident: TypeIdent,
    pub variants: Vec<Variant>,
    pub doc_lines: Vec<String>,
    pub options: EnumOptions,
}

pub(crate) fn parse_enum_item(item: ItemEnum) -> Enum {
    let ident = TypeIdent {
        name: item.ident.to_string(),
        generic_args: item
            .generics
            .params
            .iter()
            .filter_map(|param| match param {
                GenericParam::Type(ty) => {
                    Some((TypeIdent::from(ty.ident.to_string()), format_bounds(ty)))
                }
                _ => None,
            })
            .collect(),
        ..Default::default()
    };
    let options = EnumOptions::from_attrs(&item.attrs);
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

            // Variants with inline tags may result in unserializable types.
            let has_inline_tag =
                options.tag_prop_name.is_some() && options.content_prop_name.is_none();

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
                            .unwrap_or_else(|| panic!("Unnamed field in variant of enum {}", ident))
                            .to_string();
                        if has_inline_tag && options.tag_prop_name.as_ref() == Some(&name) {
                            panic!(
                                "Enum {} cannot be serialized, because the variant `{}` has a \
                                    field with the same name as the enum's `tag` attribute",
                                ident, variant.ident
                            );
                        }

                        Field {
                            name: Some(name),
                            ty: TypeIdent::try_from(&field.ty)
                                .unwrap_or_else(|_| panic!("Invalid field type in enum {}", ident)),
                            doc_lines: get_doc_lines(&field.attrs),
                            attrs: FieldAttrs::from_attrs(&field.attrs),
                        }
                    })
                    .collect();
                Type::Struct(Struct {
                    ident: TypeIdent::from(name.clone()),
                    fields,
                    doc_lines: Vec::new(),
                    options: StructOptions::default(),
                })
            } else {
                let item_types: Vec<_> = variant
                    .fields
                    .iter()
                    .map(|field| {
                        if has_inline_tag && is_path_to_primitive(&field.ty) {
                            panic!(
                                "Enum {} cannot be serialized, because the variant `{}` has a \
                                    primitive unnamed field ({}) and the enum has no `content` \
                                    attribute",
                                ident,
                                variant.ident,
                                field.ty.to_token_stream()
                            );
                        }

                        TypeIdent::try_from(&field.ty)
                            .unwrap_or_else(|_| panic!("Invalid field type in enum {}", ident))
                    })
                    .collect();

                if has_inline_tag && item_types.len() > 1 {
                    panic!(
                        "Enum {} cannot be serialized, because the variant `{}` contains multiple \
                            unnamed fields and the enum has no `content` attribute",
                        ident, variant.ident,
                    );
                }

                Type::Tuple(item_types)
            };
            let doc_lines = get_doc_lines(&variant.attrs);
            let attrs = VariantAttrs::from_attrs(&variant.attrs);

            Variant {
                name,
                ty,
                doc_lines,
                attrs,
            }
        })
        .collect();

    Enum {
        ident,
        variants,
        doc_lines: get_doc_lines(&item.attrs),
        options,
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct EnumOptions {
    pub variant_casing: Casing,
    pub content_prop_name: Option<String>,
    pub tag_prop_name: Option<String>,

    /// If `true`, serialized variants are not tagged, and a match is found
    /// by attempting to deserialize variants in order, where the first one to
    /// successfully deserialize is used as the result.
    pub untagged: bool,

    /// Rust module path where the type can be found for the given generator.
    /// If present, the generator can use this type instead of generating it.
    ///
    /// ## Example:
    ///
    /// ```rs
    /// #[fp(rust_module = "my_crate")]
    /// enum MyEnum { /* ... */ }
    /// ```
    ///
    /// This will set `"my_crate"` as the rust_module, to
    /// be used by the Rust plugin generator to generate a `use` statement such
    /// as:
    ///
    /// ```rs
    /// pub use my_crate::MyEnum;
    /// ```
    ///
    /// Instead of generating the enum definition itself.
    pub rust_module: Option<String>,
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
        if let Some(other_rust_module) = &other.rust_module {
            self.rust_module = Some(other_rust_module.clone());
        }
    }

    pub fn to_serde_attrs(&self) -> Vec<String> {
        let mut serde_attrs = vec![];
        if self.untagged {
            serde_attrs.push("untagged".to_owned());
        } else if let Some(prop_name) = &self.tag_prop_name {
            serde_attrs.push(format!("tag = \"{prop_name}\""));

            if let Some(prop_name) = &self.content_prop_name {
                serde_attrs.push(format!("content = \"{prop_name}\""));
            }
        }
        if let Some(casing) = &self.variant_casing.as_maybe_str() {
            serde_attrs.push(format!("rename_all = \"{casing}\""));
        }
        serde_attrs
    }
}

impl Parse for EnumOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        let parse_value = || -> Result<String> {
            content.parse::<Token![=]>()?;
            Ok(content
                .parse::<LitStr>()?
                .to_token_stream()
                .to_string()
                .trim_matches('"')
                .to_owned())
        };

        let mut result = Self::default();
        loop {
            let key: Ident = content.call(IdentExt::parse_any)?;
            match key.to_string().as_ref() {
                "content" => result.content_prop_name = Some(parse_value()?),
                "tag" => result.tag_prop_name = Some(parse_value()?),
                "rename_all" => {
                    result.variant_casing = Casing::try_from(parse_value()?.as_ref())
                        .map_err(|err| Error::new(content.span(), err))?
                }
                "rust_module" => {
                    result.rust_module = Some(parse_value()?);
                }
                "untagged" => result.untagged = true,
                other => {
                    return Err(Error::new(
                        content.span(),
                        format!("Unexpected attribute: {other}"),
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Variant {
    pub name: String,
    pub ty: Type,
    pub doc_lines: Vec<String>,
    pub attrs: VariantAttrs,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct VariantAttrs {
    pub field_casing: Casing,

    /// Optional name to use in the serialized format
    /// (only used if different than the variant name itself).
    ///
    /// See also: <https://serde.rs/variant-attrs.html#rename>
    pub rename: Option<String>,
}

impl VariantAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> Self {
        let mut opts = Self::default();
        for attr in attrs {
            if attr.path.is_ident("fp") || attr.path.is_ident("serde") {
                opts.merge_with(
                    &syn::parse2::<Self>(attr.tokens.clone())
                        .expect("Could not parse variant attributes"),
                );
            }
        }
        opts
    }

    fn merge_with(&mut self, other: &Self) {
        if other.field_casing != Casing::default() {
            self.field_casing = other.field_casing;
        }
        if other.rename.is_some() {
            self.rename = other.rename.clone();
        }
    }

    pub fn to_serde_attrs(&self) -> Vec<String> {
        let mut serde_attrs = vec![];
        if let Some(rename) = self.rename.as_ref() {
            serde_attrs.push(format!("rename = \"{rename}\""));
        }
        if let Some(casing) = &self.field_casing.as_maybe_str() {
            serde_attrs.push(format!("rename_all = \"{casing}\""));
        }
        serde_attrs
    }
}

impl Parse for VariantAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        let parse_value = || -> Result<String> {
            content.parse::<Token![=]>()?;
            Ok(content
                .parse::<LitStr>()?
                .to_token_stream()
                .to_string()
                .trim_matches('"')
                .to_owned())
        };

        let mut result = Self::default();
        loop {
            let key: Ident = content.call(IdentExt::parse_any)?;
            match key.to_string().as_ref() {
                "rename" => result.rename = Some(parse_value()?),
                "rename_all" => {
                    result.field_casing = Casing::try_from(parse_value()?.as_ref())
                        .map_err(|err| Error::new(content.span(), err))?
                }
                other => {
                    return Err(Error::new(
                        content.span(),
                        format!("Unexpected variant attribute: {other}"),
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

fn is_path_to_primitive(ty: &syn::Type) -> bool {
    matches!(
        ty,
        syn::Type::Path(TypePath { path, qself })
            if qself.is_none()
                && path
                    .get_ident()
                    .map(ToString::to_string)
                    .map(|ident| ident == "String" || Primitive::from_str(&ident).is_ok())
                    .unwrap_or(false)
    )
}
