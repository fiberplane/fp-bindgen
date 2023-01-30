use inflector::Inflector;
use std::convert::TryFrom;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Casing {
    Original,
    CamelCase,
    PascalCase,
    SnakeCase,
    ScreamingSnakeCase,
}

impl Casing {
    pub fn as_maybe_str(&self) -> Option<&'static str> {
        match self {
            Self::Original => None,
            Self::CamelCase => Some("camelCase"),
            Self::PascalCase => Some("PascalCase"),
            Self::SnakeCase => Some("snake_case"),
            Self::ScreamingSnakeCase => Some("SCREAMING_SNAKE_CASE"),
        }
    }

    pub fn format_string(&self, string: &str) -> String {
        match self {
            Self::Original => string.to_owned(),
            Self::CamelCase => string.to_camel_case(),
            Self::PascalCase => string.to_pascal_case(),
            Self::SnakeCase => string.to_snake_case(),
            Self::ScreamingSnakeCase => string.to_screaming_snake_case(),
        }
    }
}

impl Default for Casing {
    fn default() -> Self {
        Self::Original
    }
}

impl TryFrom<&str> for Casing {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "camelCase" => Ok(Self::CamelCase),
            "PascalCase" => Ok(Self::PascalCase),
            "snake_case" => Ok(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Ok(Self::ScreamingSnakeCase),
            other => Err(format!("Unrecognized case format: {other}")),
        }
    }
}
