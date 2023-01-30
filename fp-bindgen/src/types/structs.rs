use super::TypeIdent;
use crate::types::format_bounds;
use crate::{casing::Casing, docs::get_doc_lines};
use quote::ToTokens;
use std::convert::TryFrom;
use syn::{
    ext::IdentExt, parenthesized, parse::Parse, parse::ParseStream, Attribute, Error, GenericParam,
    Ident, ItemStruct, LitStr, Result, Token,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Struct {
    pub ident: TypeIdent,
    pub fields: Vec<Field>,
    pub doc_lines: Vec<String>,
    pub options: StructOptions,
}

pub(crate) fn parse_struct_item(item: ItemStruct) -> Struct {
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
    let fields = item
        .fields
        .iter()
        .map(|field| Field {
            name: field.ident.as_ref().map(Ident::to_string),
            ty: TypeIdent::try_from(&field.ty)
                .unwrap_or_else(|_| panic!("Invalid field type in struct {}", ident)),
            doc_lines: get_doc_lines(&field.attrs),
            attrs: FieldAttrs::from_attrs(&field.attrs),
        })
        .collect();

    Struct {
        ident,
        fields,
        doc_lines: get_doc_lines(&item.attrs),
        options: StructOptions::from_attrs(&item.attrs),
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct StructOptions {
    pub field_casing: Casing,

    /// Rust module path where the type can be found for the given generator.
    /// If present, the generator can use this type instead of generating it.
    ///
    /// ## Example:
    ///
    /// ```rs
    /// #[fp(rust_module = "my_crate")]
    /// struct MyStruct { /* ... */ }
    /// ```
    ///
    /// This will set `"my_crate"` as the rust_module, to
    /// be used by the Rust plugin generator to generate a `use` statement such
    /// as:
    ///
    /// ```rs
    /// pub use my_crate::MyStruct;
    /// ```
    ///
    /// Instead of generating the struct definition itself.
    pub rust_module: Option<String>,
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
        opts
    }

    fn merge_with(&mut self, other: &Self) {
        if other.field_casing != Casing::default() {
            self.field_casing = other.field_casing;
        }
        if let Some(other_rust_module) = &other.rust_module {
            self.rust_module = Some(other_rust_module.clone());
        }
    }

    pub fn to_serde_attrs(&self) -> Vec<String> {
        let mut serde_attrs = vec![];
        if let Some(casing) = &self.field_casing.as_maybe_str() {
            serde_attrs.push(format!("rename_all = \"{casing}\""));
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
                "rust_module" => {
                    result.rust_module = Some(parse_value()?);
                }
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
pub struct Field {
    pub name: Option<String>,
    pub ty: TypeIdent,
    pub doc_lines: Vec<String>,
    pub attrs: FieldAttrs,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct FieldAttrs {
    /// Optional path to a function that will produce the default value in case
    /// the field is omitted from the serialized representation.
    ///
    /// See also: <https://serde.rs/field-attrs.html#default--path>
    ///
    /// An empty string may be used as value, in which case `Default::default`
    /// is assumed. See also: <https://serde.rs/field-attrs.html#default>
    pub default: Option<String>,

    /// Optional Serde dependency used for deserialization.
    ///
    /// See: <https://serde.rs/field-attrs.html#deserialize_with>
    pub deserialize_with: Option<String>,

    /// Determines whether the field should be flattened into the parent struct.
    ///
    /// See: <https://serde.rs/attr-flatten.html>
    pub flatten: bool,

    /// Optional name to use in the serialized format
    /// (only used if different than the field name itself).
    ///
    /// See also: <https://serde.rs/field-attrs.html#rename>
    pub rename: Option<String>,

    /// Optional Serde dependency used for serialization.
    ///
    /// See also: <https://serde.rs/field-attrs.html#serialize_with>
    pub serialize_with: Option<String>,

    /// Optional path to a function to determine whether serialized should be
    /// skipped for a particular value.
    ///
    /// E.g.: can be used to omit `Option`s by specifying
    /// `skip_serializing_if = "Option::is_none"`.
    ///
    /// See also: <https://serde.rs/field-attrs.html#skip_serializing_if>
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
        if other.default.is_some() {
            self.default = other.default.clone();
        }
        if other.deserialize_with.is_some() {
            self.deserialize_with = other.deserialize_with.clone();
        }
        if other.flatten {
            self.flatten = other.flatten;
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
        if let Some(default) = self.default.as_ref() {
            if default.is_empty() {
                serde_attrs.push("default".to_owned());
            } else {
                serde_attrs.push(format!("default = \"{default}\""));
            }
        }
        match (self.deserialize_with.as_ref(), self.serialize_with.as_ref()) {
            (Some(deserialize_with), Some(serialize_with))
                if deserialize_with == serialize_with =>
            {
                serde_attrs.push(format!("with = \"{deserialize_with}\""));
            }
            (Some(deserialize_with), Some(serialize_with)) => {
                serde_attrs.push(format!("deserialize_with = \"{deserialize_with}\""));
                serde_attrs.push(format!("serialize_with = \"{serialize_with}\""));
            }
            (Some(deserialize_with), None) => {
                serde_attrs.push(format!("deserialize_with = \"{deserialize_with}\""));
            }
            (None, Some(serialize_with)) => {
                serde_attrs.push(format!("serialize_with = \"{serialize_with}\""));
            }
            (None, None) => {}
        }
        if self.flatten {
            serde_attrs.push("flatten".to_owned());
        }
        if let Some(rename) = self.rename.as_ref() {
            serde_attrs.push(format!("rename = \"{rename}\""));
        }
        if let Some(skip_serializing_if) = self.skip_serializing_if.as_ref() {
            serde_attrs.push(format!("skip_serializing_if = \"{skip_serializing_if}\""));
        }
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

        let parse_optional_value = || -> Result<String> {
            if content.peek(Token![=]) {
                parse_value()
            } else {
                Ok(String::new())
            }
        };

        let mut result = Self::default();
        loop {
            let key: Ident = content.call(IdentExt::parse_any)?;
            match key.to_string().as_ref() {
                "default" => result.default = Some(parse_optional_value()?),
                "deserialize_with" => result.deserialize_with = Some(parse_value()?),
                "flatten" => result.flatten = true,
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
                        format!("Unexpected field attribute: {other}"),
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
