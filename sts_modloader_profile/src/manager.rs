use sts_modloader_core::{AppConfig, Profile};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProfileError {
    #[error("Profile with name '{0}' already exists.")]
    AlreadyExists(String),
    #[error("Profile '{0}' not found.")]
    NotFound(String),
    #[error("Profile name cannot be empty.")]
    EmptyName,
    #[error("Default profile cannot be modified.")]
    DefaultRestricted,
}

/// Creates a new profile in AppConfig, containing the provided enabled mod IDs.
pub fn create_profile(
    config: &mut AppConfig,
    name: String,
    enabled_mods: Vec<String>,
) -> Result<(), ProfileError> {
    let cleaned_name = name.trim();
    if cleaned_name.is_empty() {
        return Err(ProfileError::EmptyName);
    }
    if config.profiles.iter().any(|p| p.name.eq_ignore_ascii_case(cleaned_name)) {
        return Err(ProfileError::AlreadyExists(cleaned_name.to_string()));
    }
    
    let new_profile = Profile {
        name: cleaned_name.to_string(),
        enabled_mods,
    };
    config.profiles.push(new_profile);
    config.active_profile = Some(cleaned_name.to_string());
    Ok(())
}

/// Deletes a profile from AppConfig by name.
pub fn delete_profile(config: &mut AppConfig, name: &str) -> Result<(), ProfileError> {
    if name.eq_ignore_ascii_case("Default") {
        return Err(ProfileError::DefaultRestricted);
    }
    let idx = config
        .profiles
        .iter()
        .position(|p| p.name == name)
        .ok_or_else(|| ProfileError::NotFound(name.to_string()))?;
        
    config.profiles.remove(idx);
    
    // Set fallback active profile
    if config.active_profile.as_deref() == Some(name) {
        config.active_profile = config.profiles.first().map(|p| p.name.clone());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sts_modloader_core::{AppConfig, Profile};

    fn setup_mock_config() -> AppConfig {
        AppConfig {
            sts_path: None,
            debug_mode: false,
            profiles: vec![
                Profile {
                    name: "Default".to_string(),
                    enabled_mods: vec![],
                },
                Profile {
                    name: "Profile1".to_string(),
                    enabled_mods: vec!["mod1".to_string(), "mod2".to_string()],
                },
            ],
            active_profile: Some("Profile1".to_string()),
            theme: None,
        }
    }

    #[test]
    fn test_create_profile_success() {
        let mut config = setup_mock_config();
        let res = create_profile(&mut config, "NewProfile  ".to_string(), vec!["mod3".to_string()]);
        assert!(res.is_ok());
        assert_eq!(config.profiles.len(), 3);
        assert_eq!(config.profiles[2].name, "NewProfile");
        assert_eq!(config.profiles[2].enabled_mods, vec!["mod3"]);
        assert_eq!(config.active_profile.as_deref(), Some("NewProfile"));
    }

    #[test]
    fn test_create_profile_empty_name() {
        let mut config = setup_mock_config();
        let res = create_profile(&mut config, "   ".to_string(), vec![]);
        assert!(matches!(res, Err(ProfileError::EmptyName)));
    }

    #[test]
    fn test_create_profile_already_exists() {
        let mut config = setup_mock_config();
        // Case insensitive match
        let res = create_profile(&mut config, "profile1".to_string(), vec![]);
        assert!(matches!(res, Err(ProfileError::AlreadyExists(name)) if name == "profile1"));
    }

    #[test]
    fn test_delete_profile_success() {
        let mut config = setup_mock_config();
        
        // Let's delete Profile1, which is also the active one.
        let res = delete_profile(&mut config, "Profile1");
        assert!(res.is_ok());
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].name, "Default");
        // It should fallback active_profile to the first remaining profile ("Default")
        assert_eq!(config.active_profile.as_deref(), Some("Default"));

        // Delete another when active_profile is already Default
        let res2 = delete_profile(&mut config, "Default");
        assert!(matches!(res2, Err(ProfileError::DefaultRestricted)));
    }

    #[test]
    fn test_delete_profile_not_found() {
        let mut config = setup_mock_config();
        let res = delete_profile(&mut config, "NonExistent");
        assert!(matches!(res, Err(ProfileError::NotFound(name)) if name == "NonExistent"));
    }

    #[test]
    fn test_delete_profile_default_restricted() {
        let mut config = setup_mock_config();
        let res = delete_profile(&mut config, "default"); // Case-insensitive test for default
        assert!(matches!(res, Err(ProfileError::DefaultRestricted)));
    }
}
