use std::{collections::HashSet, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct Selection(pub HashSet<PathBuf>);

impl Selection {
    pub fn into_inner(self) -> HashSet<PathBuf> {
        self.0
    }
}
