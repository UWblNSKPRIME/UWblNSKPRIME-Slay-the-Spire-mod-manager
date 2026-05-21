pub mod config;

pub use config::{AppConfig, Profile, ConfigError, get_config_dir, load_config, save_config};

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModSource {
    Local,     // Found in SlayTheSpire/mods/
    Workshop,  // Found in steamapps/workshop/content/646570/
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModInfo {
    pub id: String,                 // modid
    pub name: String,               // name
    pub authors: Vec<String>,       // Normalized list of authors
    pub version: String,            // version
    pub description: Option<String>,// description
    pub dependencies: Vec<String>,  // dependencies list
    pub sts_version: Option<String>,// target Slay the Spire version
    pub mts_version: Option<String>,// target ModTheSpire version
    pub jar_path: PathBuf,          // Absolute file path to the jar
    pub source: ModSource,          // Origin directory indicator
    pub enabled: bool,              // In-memory selection status
}
