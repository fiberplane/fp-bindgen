use std::{collections::BTreeSet, fmt};

/// Used for defining Cargo dependencies.
#[non_exhaustive]
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct CargoDependency {
    pub branch: Option<&'static str>,
    pub default_features: Option<bool>,
    pub features: BTreeSet<&'static str>,
    pub git: Option<&'static str>,
    pub path: Option<&'static str>,
    pub registry: Option<&'static str>,
    pub version: Option<&'static str>,
    pub workspace: Option<bool>,
}

impl CargoDependency {
    pub fn from_workspace() -> Self {
        Self::from_workspace_with_features(BTreeSet::new())
    }

    pub fn from_workspace_with_features(features: BTreeSet<&'static str>) -> Self {
        Self {
            features,
            workspace: Some(true),
            ..Default::default()
        }
    }

    /// Merges or replaces this dependency with another.
    ///
    /// The algorithm used attempts to reuse as much of the current
    /// dependency as possible, but treats the incoming dependency as leading
    /// in case of conflicts.
    pub fn merge_or_replace_with(&self, other: &Self) -> Self {
        if let Some(path) = &other.path {
            Self {
                branch: None,
                default_features: other.default_features.or(self.default_features),
                features: self.features.union(&other.features).copied().collect(),
                git: None,
                path: Some(path),
                registry: other.registry.or(self.registry),
                version: other.version.or(self.version),
                workspace: None,
            }
        } else if let Some(git) = &other.git {
            Self {
                branch: other.branch,
                default_features: other.default_features.or(self.default_features),
                features: self.features.union(&other.features).copied().collect(),
                git: Some(git),
                path: None,
                registry: other.registry.or(self.registry),
                version: other.version.or(self.version),
                workspace: None,
            }
        } else if let Some(workspace) = &other.workspace {
            Self {
                branch: other.branch,
                default_features: other.default_features.or(self.default_features),
                features: self.features.union(&other.features).copied().collect(),
                git: other.git,
                path: other.path,
                registry: other.registry,
                version: other.version,
                workspace: Some(*workspace),
            }
        } else {
            Self {
                branch: self.branch,
                default_features: other.default_features.or(self.default_features),
                features: self.features.union(&other.features).copied().collect(),
                git: self.git,
                path: self.path,
                registry: other.registry.or(self.registry),
                workspace: self.workspace,
                version: other.version.or(self.version),
            }
        }
    }

    pub fn with_path(path: &'static str) -> Self {
        Self::with_path_and_features(path, BTreeSet::new())
    }

    pub fn with_path_and_features(path: &'static str, features: BTreeSet<&'static str>) -> Self {
        Self {
            features,
            path: Some(path),
            ..Default::default()
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
            features,
            version: Some(version),
            ..Default::default()
        }
    }

    pub fn with_version_from_registry(version: &'static str, registry: &'static str) -> Self {
        Self::with_version_and_features_from_registry(version, registry, BTreeSet::new())
    }

    pub fn with_version_and_features_from_registry(
        version: &'static str,
        registry: &'static str,
        features: BTreeSet<&'static str>,
    ) -> Self {
        Self {
            features,
            registry: Some(registry),
            version: Some(version),
            ..Default::default()
        }
    }

    pub fn with_git(git: &'static str) -> Self {
        Self::with_git_and_features(git, BTreeSet::new())
    }

    pub fn with_git_and_features(git: &'static str, features: BTreeSet<&'static str>) -> Self {
        Self {
            features,
            git: Some(git),
            ..Default::default()
        }
    }

    pub fn with_git_and_branch(git: &'static str, branch: &'static str) -> Self {
        Self::with_git_and_branch_and_features(git, branch, BTreeSet::new())
    }

    pub fn with_git_and_branch_and_features(
        git: &'static str,
        branch: &'static str,
        features: BTreeSet<&'static str>,
    ) -> Self {
        Self {
            features,
            git: Some(git),
            branch: Some(branch),
            ..Default::default()
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
        } else if self.workspace == Some(true) {
            attributes.push("workspace = true".to_owned());
        }

        if let Some(version) = self.version {
            attributes.push(format!("version = {}", quote_value(version)));
            if let Some(registry) = self.registry {
                attributes.push(format!("registry = {}", quote_value(registry)));
            }
        }

        if let Some(default_features) = self.default_features {
            attributes.push(format!("default_features = {default_features}"));
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
