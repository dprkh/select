use std::{collections::HashSet, path::PathBuf, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct SelectedPath {
    pub path: PathBuf,
    pub recursive: bool,
}

impl SelectedPath {
    pub fn new(path: PathBuf, recursive: bool) -> Self {
        Self { path, recursive }
    }
}

impl FromStr for SelectedPath {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let recursive = !s.starts_with('*');
        let path_str = if recursive { s } else { &s[1..] };

        Ok(Self {
            path: PathBuf::from(path_str),
            recursive,
        })
    }
}

impl ToString for SelectedPath {
    fn to_string(&self) -> String {
        if self.recursive {
            self.path.display().to_string()
        } else {
            format!("*{}", self.path.display())
        }
    }
}

impl Serialize for SelectedPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for SelectedPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(s.parse().unwrap())
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct Selection(pub HashSet<SelectedPath>);

impl Selection {
    pub fn into_inner(self) -> HashSet<SelectedPath> {
        self.0
    }
}
