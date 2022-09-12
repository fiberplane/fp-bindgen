use std::{collections::BTreeSet, fmt};

/// Used for defining Cargo dependencies.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct CargoDependency {
    pub git: Option<&'static str>,
    pub branch: Option<&'static str>,
    pub path: Option<&'static str>,
    pub version: Option<&'static str>,
    pub features: BTreeSet<&'static str>,
    pub default_features: Option<bool>,
}

impl CargoDependency {
    /// Merges or replaces this dependency with another.
    ///
    /// The algorithm used attempts to reuse as much of the current
    /// dependency as possible, but treats the incoming dependency as leading
    /// in case of conflicts.
    pub fn merge_or_replace_with(&self, other: &Self) -> Self {
        if let Some(path) = &other.path {
            Self {
                git: None,
                branch: None,
                path: Some(path),
                version: other.version.or(self.version),
                features: self.features.union(&other.features).copied().collect(),
                default_features: other.default_features.or(self.default_features),
            }
        } else if let Some(git) = &other.git {
            Self {
                git: Some(git),
                branch: other.branch,
                path: None,
                version: other.version.or(self.version),
                features: self.features.union(&other.features).copied().collect(),
                default_features: other.default_features.or(self.default_features),
            }
        } else {
            Self {
                git: self.git,
                branch: self.branch,
                path: self.path,
                version: other.version.or(self.version),
                features: self.features.union(&other.features).copied().collect(),
                default_features: other.default_features.or(self.default_features),
            }
        }
    }

    pub fn with_version(version: &'static str) -> Self {
        Self::with_version_and_features(version, BTreeSet::new())
    }

    pub fn with_version_and_features(
        version: &'static str,
        features: BTreeSet<&'static str>,
    ) -> Self {
        Self {
            git: None,
            branch: None,
            path: None,
            version: Some(version),
            features,
            default_features: None,
        }
    }
}

impl fmt::Display for CargoDependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut attributes = Vec::new();

        if let Some(path) = self.path {
            attributes.push(format!("path = {}", quote_value(path)));
        } else if let Some(git) = self.git {
            attributes.push(format!("git = {}", quote_value(git)));
            if let Some(branch) = self.branch {
                attributes.push(format!("branch = {}", quote_value(branch)));
            }
        }

        if let Some(version) = self.version {
            attributes.push(format!("version = {}", quote_value(version)));
        }

        if let Some(default_features) = self.default_features {
            attributes.push(format!("default_features = {}", default_features));
        }

        if !self.features.is_empty() {
            attributes.push(format!(
                "features = [{}]",
                self.features
                    .iter()
                    .map(|f| quote_value(f))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        write!(f, "{{ {} }}", attributes.join(", "))
    }
}

fn quote_value(val: &str) -> String {
    format!("\"{}\"", val.replace('\\', "\\\\").replace('\"', "\\\""))
}
