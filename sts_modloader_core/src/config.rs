use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Config folder could not be resolved")]
    PathResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub enabled_mods: Vec<String>, // List of modids
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub sts_path: Option<PathBuf>,
    pub debug_mode: bool,
    pub profiles: Vec<Profile>,
    pub active_profile: Option<String>,
    pub theme: Option<String>,
}

/// Returns the platform-specific configuration directory path for this application.
pub fn get_config_dir() -> Result<PathBuf, ConfigError> {
    dirs::config_dir()
        .map(|p| p.join("sts_modloader"))
        .ok_or(ConfigError::PathResolution)
}

/// Loads the application configuration from disk. 
/// If the file does not exist, returns a default AppConfig.
pub fn load_config() -> Result<AppConfig, ConfigError> {
    load_config_from_dir(get_config_dir()?)
}

/// Helper function to load configuration from a specific directory.
pub fn load_config_from_dir(config_dir: PathBuf) -> Result<AppConfig, ConfigError> {
    let config_path = config_dir.join("modloader_config.json");
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }
    let file = std::fs::File::open(config_path)?;
    let config = serde_json::from_reader(file)?;
    Ok(config)
}

/// Saves the application configuration to disk. Creates necessary parent directories.
pub fn save_config(config: &AppConfig) -> Result<(), ConfigError> {
    save_config_to_dir(config, get_config_dir()?)
}

/// Helper function to save configuration to a specific directory.
pub fn save_config_to_dir(config: &AppConfig, config_dir: PathBuf) -> Result<(), ConfigError> {
    std::fs::create_dir_all(&config_dir)?;
    let config_path = config_dir.join("modloader_config.json");
    let file = std::fs::File::create(config_path)?;
    serde_json::to_writer_pretty(file, config)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn get_temp_dir() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let count = DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = temp_dir.join(format!("test_config_dir_{}_{}", std::process::id(), count));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn test_get_config_dir() {
        let dir_res = get_config_dir();
        assert!(dir_res.is_ok());
        let dir = dir_res.unwrap();
        assert!(dir.to_string_lossy().contains("sts_modloader"));
    }

    #[test]
    fn test_load_config_non_existent_returns_default() {
        let temp_dir = get_temp_dir();
        let res = load_config_from_dir(temp_dir.clone());
        let _ = fs::remove_dir_all(&temp_dir);

        let config = res.expect("Should load successfully");
        assert!(config.sts_path.is_none());
        assert!(!config.debug_mode);
        assert!(config.profiles.is_empty());
        assert!(config.active_profile.is_none());
    }

    #[test]
    fn test_save_and_load_config_success() {
        let temp_dir = get_temp_dir();
        
        let mut config = AppConfig::default();
        config.sts_path = Some(PathBuf::from("mock/sts/path"));
        config.debug_mode = true;
        config.profiles = vec![Profile {
            name: "TestProfile".to_string(),
            enabled_mods: vec!["mod-id-1".to_string()],
        }];
        config.active_profile = Some("TestProfile".to_string());
        config.theme = Some("Light".to_string());

        let save_res = save_config_to_dir(&config, temp_dir.clone());
        assert!(save_res.is_ok());

        let load_res = load_config_from_dir(temp_dir.clone());
        let _ = fs::remove_dir_all(&temp_dir);

        let loaded = load_res.expect("Should load successfully");
        assert_eq!(loaded.sts_path, Some(PathBuf::from("mock/sts/path")));
        assert!(loaded.debug_mode);
        assert_eq!(loaded.profiles.len(), 1);
        assert_eq!(loaded.profiles[0].name, "TestProfile");
        assert_eq!(loaded.profiles[0].enabled_mods, vec!["mod-id-1"]);
        assert_eq!(loaded.active_profile.as_deref(), Some("TestProfile"));
        assert_eq!(loaded.theme.as_deref(), Some("Light"));
    }

    #[test]
    fn test_load_config_invalid_json() {
        let temp_dir = get_temp_dir();
        let config_path = temp_dir.join("modloader_config.json");
        fs::write(config_path, "{ invalid json }").unwrap();

        let res = load_config_from_dir(temp_dir.clone());
        let _ = fs::remove_dir_all(&temp_dir);

        assert!(matches!(res, Err(ConfigError::Json(_))));
    }
}
