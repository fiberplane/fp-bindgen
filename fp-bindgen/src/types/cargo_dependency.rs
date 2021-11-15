use std::{collections::BTreeSet, fmt};

/// Used for defining Cargo dependencies.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct CargoDependency {
    pub git: Option<&'static str>,
    pub branch: Option<&'static str>,
    pub path: Option<&'static str>,
    pub version: Option<&'static str>,
    pub features: BTreeSet<&'static str>,
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
            }
        } else if let Some(git) = &other.git {
            Self {
                git: Some(git),
                branch: other.branch,
                path: None,
                version: other.version.or(self.version),
                features: self.features.union(&other.features).copied().collect(),
            }
        } else {
            Self {
                git: self.git,
                branch: self.branch,
                path: self.path,
                version: other.version.or(self.version),
                features: self.features.union(&other.features).copied().collect(),
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
        }
    }
}

impl fmt::Display for CargoDependency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {}
}
