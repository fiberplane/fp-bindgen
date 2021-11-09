use super::{resolve_type_or_panic, GenericArgument, Type};
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
            let ty = resolve_type_or_panic(&field.ty, dependencies, "Unresolvable field type");
            let doc_lines = get_doc_lines(&field.attrs);
            let attrs = FieldAttrs::from_attrs(&field.attrs);
            Field {
                name,
                ty,
                doc_lines,
                attrs,
            }
        })
        .collect();
    let opts = StructOptions::from_attrs(&item.attrs);
    Type::Struct(name, generic_args, doc_lines, fields, opts)
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
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

    fn merge_with(&mut self, other: &Self) {
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
                "rename_all" => {
                    result.field_casing = Casing::try_from(parse_value()?.as_ref())
                        .map_err(|err| Error::new(content.span(), err))?
                }
                module if module.ends_with("_module") => {
                    result
                        .native_modules
                        .insert(module[0..module.len() - 7].to_owned(), parse_value()?);
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub doc_lines: Vec<String>,
    pub attrs: FieldAttrs,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct FieldAttrs {
    /// Optional Serde dependency used for deserialization.
    pub deserialize_with: Option<String>,

    /// Optional name to use in the serialized format
    /// (only used if different than the field name itself).
    pub rename: Option<String>,

    /// Optional Serde dependency used for serialization.
    pub serialize_with: Option<String>,

    /// Optional condition for skipping serialization of the field.
    /// Mainly used for omitting `Option`s.
    pub skip_serializing_if: Option<String>,
}

impl FieldAttrs {
    pub fn from_attrs(attrs: &[Attribute]) -> Self {
        let mut opts = Self::default();
        for attr in attrs {
            if attr.path.is_ident("fp") || attr.path.is_ident("serde") {
                opts.merge_with(
                    &syn::parse2::<Self>(attr.tokens.clone())
                        .expect("Could not parse field attributes"),
                );
            }
        }
        opts
    }

    fn merge_with(&mut self, other: &Self) {
        if other.deserialize_with.is_some() {
            self.deserialize_with = other.deserialize_with.clone();
        }
        if other.rename.is_some() {
            self.rename = other.rename.clone();
        }
        if other.serialize_with.is_some() {
            self.serialize_with = other.serialize_with.clone();
        }
        if other.skip_serializing_if.is_some() {
            self.skip_serializing_if = other.skip_serializing_if.clone();
        }
    }

    pub fn to_serde_attrs(&self) -> Vec<String> {
        let mut serde_attrs = vec![];
        match (self.deserialize_with.as_ref(), self.serialize_with.as_ref()) {
            (Some(deserialize_with), Some(serialize_with))
                if deserialize_with == serialize_with =>
            {
                serde_attrs.push(format!("with = \"{}\"", deserialize_with));
            }
            (Some(deserialize_with), Some(serialize_with)) => {
                serde_attrs.push(format!("deserialize_with = \"{}\"", deserialize_with));
                serde_attrs.push(format!("serialize_with = \"{}\"", serialize_with));
            }
            (Some(deserialize_with), None) => {
                serde_attrs.push(format!("deserialize_with = \"{}\"", deserialize_with));
            }
            (None, Some(serialize_with)) => {
                serde_attrs.push(format!("serialize_with = \"{}\"", serialize_with));
            }
            (None, None) => {}
        }
        if let Some(rename) = self.rename.as_ref() {
            serde_attrs.push(format!("rename = \"{}\"", rename));
        }
        if let Some(skip_serializing_if) = self.skip_serializing_if.as_ref() {
            serde_attrs.push(format!("skip_serializing_if = \"{}\"", skip_serializing_if));
        }
        serde_attrs.sort();
        serde_attrs
    }
}

impl Parse for FieldAttrs {
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
                "deserialize_with" => result.deserialize_with = Some(parse_value()?),
                "rename" => result.rename = Some(parse_value()?),
                "serialize_with" => result.serialize_with = Some(parse_value()?),
                "skip_serializing_if" => result.skip_serializing_if = Some(parse_value()?),
                "with" => {
                    let value = parse_value()?;
                    result.deserialize_with = Some(value.clone());
                    result.serialize_with = Some(value);
                }
                other => {
                    return Err(Error::new(
                        content.span(),
                        format!("Unexpected field attribute: {}", other),
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
