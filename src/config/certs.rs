use serde_derive::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertsConfig {
    pub search_paths: Vec<PathBuf>,
}
