// MIT License
//
// Copyright (c) 2025 Dmytro Prokhorov
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

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

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct Selection(pub HashSet<SelectedPath>);

impl Selection {
    pub fn into_inner(self) -> HashSet<SelectedPath> {
        self.0
    }
}
