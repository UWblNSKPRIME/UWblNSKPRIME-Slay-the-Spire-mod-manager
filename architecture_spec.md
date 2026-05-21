# Slay the Spire Mod Loader - Architecture Specification

This document provides a detailed architectural specification for the Slay the Spire Mod Loader, a native, lightweight Rust GUI alternative to the standard Java-based ModTheSpire launcher. It outlines the crate structure, data models, file structures, API interfaces, and state-machine flows.

---

## 1. Cargo Workspace Directory Structure

The project is structured as a Cargo workspace with a root-level orchestrator binary crate and five supporting library crates to ensure strict separation of concerns, easy testing, and clear interface contracts.

```text
sts_modloader_root/
├── Cargo.toml                      # Workspace Root Configuration
├── sts_modloader/                  # Main Binary Crate (Orchestrator)
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                 # Entry point, spins up Iced UI
├── sts_modloader_core/             # App Config & Common Event/Error types
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── config.rs               # JSON configuration structures & loading
├── sts_modloader_fs/               # Steam detection, walkdir file-scanning, Java launcher
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── steam.rs                # Registry checks, libraryfolders.vdf parsing
│       ├── scanner.rs              # Mod scanning (local & workshop)
│       └── runner.rs               # Java runner (ModTheSpire properties write & run)
├── sts_modloader_parser/           # Zip/Jar parser for ModTheSpire.json
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── metadata.rs             # Metadata deserialization & normalization
├── sts_modloader_profile/          # Profile CRUD operations, import/export
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── manager.rs              # Profiles operations logic
└── sts_modloader_ui/               # GUI state machine, widgets, dark styles
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── app.rs                  # Iced Application trait implementation
        ├── components/
        │   ├── left_panel.rs       # Mod list with checkboxes & search
        │   ├── right_panel.rs      # Mod details, dependencies & badges
        │   ├── control_block.rs    # Profiles PickList, CRUD buttons & refresh
        │   ├── bottom_bar.rs       # Launch button & debug checkbox
        │   └── setup_screen.rs     # Slay the Spire path auto-detect/manual chooser
        └── styles.rs               # Theme, styling constants & custom sheets
```

---

## 2. Cargo Dependency Specifications

### 2.1 Workspace Root `Cargo.toml`
```toml
[workspace]
members = [
    "sts_modloader",
    "sts_modloader_core",
    "sts_modloader_fs",
    "sts_modloader_parser",
    "sts_modloader_profile",
    "sts_modloader_ui"
]
resolver = "2"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### 2.2 `sts_modloader/Cargo.toml`
```toml
[package]
name = "sts_modloader"
version = "0.1.0"
edition = "2021"

[dependencies]
sts_modloader_ui = { path = "../sts_modloader_ui" }
sts_modloader_core = { path = "../sts_modloader_core" }
tokio = { version = "1.35", features = ["rt-multi-thread", "macros"] }
iced = { version = "0.12", features = ["tokio", "image"] }
```

### 2.3 `sts_modloader_core/Cargo.toml`
```toml
[package]
name = "sts_modloader_core"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dirs = "5.0"
thiserror = "1.0"
```

### 2.4 `sts_modloader_fs/Cargo.toml`
```toml
[package]
name = "sts_modloader_fs"
version = "0.1.0"
edition = "2021"

[dependencies]
sts_modloader_core = { path = "../sts_modloader_core" }
sts_modloader_parser = { path = "../sts_modloader_parser" }
walkdir = "2.4"
tokio = { version = "1.35", features = ["process", "fs", "rt"] }
rfd = "0.12"
thiserror = "1.0"

[target.'cfg(windows)'.dependencies]
winreg = "0.50"
```

### 2.5 `sts_modloader_parser/Cargo.toml`
```toml
[package]
name = "sts_modloader_parser"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
zip = { version = "0.6", default-features = false, features = ["deflate"] }
thiserror = "1.0"
```

### 2.6 `sts_modloader_profile/Cargo.toml`
```toml
[package]
name = "sts_modloader_profile"
version = "0.1.0"
edition = "2021"

[dependencies]
sts_modloader_core = { path = "../sts_modloader_core" }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
```

### 2.7 `sts_modloader_ui/Cargo.toml`
```toml
[package]
name = "sts_modloader_ui"
version = "0.1.0"
edition = "2021"

[dependencies]
sts_modloader_core = { path = "../sts_modloader_core" }
sts_modloader_fs = { path = "../sts_modloader_fs" }
sts_modloader_parser = { path = "../sts_modloader_parser" }
sts_modloader_profile = { path = "../sts_modloader_profile" }
iced = { version = "0.12", features = ["tokio", "canvas", "image"] }
tokio = { version = "1.35", features = ["rt", "macros"] }
```

---

## 3. Data Models & Formats

### 3.1 ModTheSpire.json Metadata Format
Inside every mod `.jar` archive, a `ModTheSpire.json` file must reside in the root directory. Because mod creators have inconsistencies in declaring fields (e.g., singular vs. plural author fields, arrays vs. comma-delimited strings), the parser must deserialize into a raw structure and then normalize the data.

#### Example `ModTheSpire.json`
```json
{
  "modid": "basemod",
  "name": "BaseMod",
  "author_list": ["t-larson", "kiooeht", "daviscook"],
  "version": "5.44.0",
  "description": "An API for modding Slay the Spire.",
  "dependencies": ["ModTheSpire"],
  "sts_version": "2.0",
  "mts_version": "3.18.0"
}
```

#### Raw & Normalized Rust Structs
```rust
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModSource {
    Local,     // Found in SlayTheSpire/mods/
    Workshop,  // Found in steamapps/workshop/content/646570/
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
```

### 3.2 Application Configuration (`modloader_config.json`)
The application config will store user preferences, paths, and saved mod profiles. It is stored in the standard user config folder:
* Windows: `%APPDATA%/sts_modloader/modloader_config.json`
* macOS/Linux: `~/.config/sts_modloader/modloader_config.json`

#### Config Schema Example
```json
{
  "sts_path": "D:\\SteamLibrary\\steamapps\\common\\SlayTheSpire",
  "debug_mode": false,
  "profiles": [
    {
      "name": "Base Config",
      "enabled_mods": ["basemod", "stslib"]
    },
    {
      "name": "Downfall Mod Pack",
      "enabled_mods": ["basemod", "stslib", "downfall"]
    }
  ],
  "active_profile": "Base Config"
}
```

### 3.3 ModTheSpire Properties Configuration
To launch the game with the selected mods without opening the Java ModTheSpire UI, we write configuration directly into the properties file:
* Path: `%LOCALAPPDATA%/ModTheSpire/ModTheSpire.properties`
* Format: Java properties format (key-value text separated by `=` or `:`).

#### Properties Output Schema
```properties
# Written by Slay the Spire Mod Loader
mods=basemod,stslib,downfall
debug=true
debug_mode=true
```
> [!NOTE]
> ModTheSpire uses mod IDs (e.g. `basemod`) rather than JAR names. The IDs must be comma-separated without spaces. We write both `debug` and `debug_mode` to ensure compatibility across various ModTheSpire releases.

---

## 4. Crate Interfaces & API Specifications

### 4.1 `sts_modloader_core`
Manages loading, updating, and saving `AppConfig`.

#### File: `src/config.rs`
```rust
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    let config_dir = get_config_dir()?;
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
    let config_dir = get_config_dir()?;
    std::fs::create_dir_all(&config_dir)?;
    let config_path = config_dir.join("modloader_config.json");
    let file = std::fs::File::create(config_path)?;
    serde_json::to_writer_pretty(file, config)?;
    Ok(())
}
```

### 4.2 `sts_modloader_parser`
Extracts and parses mod metadata from JAR files in-memory without disk extraction.

#### File: `src/metadata.rs`
```rust
use std::path::Path;
use serde::Deserialize;
use thiserror::Error;
use crate::metadata::{ModInfo, ModSource};

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Failed to read zip archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Failed to open jar file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse ModTheSpire.json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("ModTheSpire.json was not found in the jar root")]
    MetadataNotFound,
}

#[derive(Debug, Deserialize)]
struct RawModMetadata {
    pub modid: String,
    pub name: String,
    
    // Alias to handle raw string, string arrays, or missing values gracefully
    #[serde(alias = "author_list", alias = "authors", alias = "author")]
    pub authors: Option<serde_json::Value>,
    
    pub version: String,
    pub description: Option<String>,
    
    #[serde(default)]
    pub dependencies: Vec<String>,
    
    #[serde(alias = "stsVersion")]
    pub sts_version: Option<String>,
    
    #[serde(alias = "mtsVersion")]
    pub mts_version: Option<String>,
}

/// Normalizes the custom polymorphic Serde Value authors into a flat vector.
fn normalize_authors(value: Option<serde_json::Value>) -> Vec<String> {
    match value {
        None => vec![],
        Some(val) => match val {
            serde_json::Value::String(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    vec![]
                } else if trimmed.contains(',') {
                    trimmed.split(',').map(|a| a.trim().to_string()).filter(|a| !a.is_empty()).collect()
                } else {
                    vec![trimmed.to_string()]
                }
            }
            serde_json::Value::Array(arr) => arr
                .into_iter()
                .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect(),
            _ => vec![],
        }
    }
}

/// Reads the ModTheSpire.json directly from the jar file and returns a normalized ModInfo.
pub fn parse_jar_metadata(jar_path: &Path, source: ModSource) -> Result<ModInfo, ParserError> {
    let file = std::fs::File::open(jar_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    let mut meta_file = archive
        .by_name("ModTheSpire.json")
        .map_err(|_| ParserError::MetadataNotFound)?;
        
    let raw: RawModMetadata = serde_json::from_reader(&mut meta_file)?;
    
    Ok(ModInfo {
        id: raw.modid,
        name: raw.name,
        authors: normalize_authors(raw.authors),
        version: raw.version,
        description: raw.description,
        dependencies: raw.dependencies,
        sts_version: raw.sts_version,
        mts_version: raw.mts_version,
        jar_path: jar_path.to_path_buf(),
        source,
        enabled: false,
    })
}
```

### 4.3 `sts_modloader_fs`
Locates Slay the Spire game files, scans folders asynchronously, and launches the Java process.

#### File: `src/steam.rs`
```rust
use std::path::PathBuf;

/// Scans standard Steam registry keys on Windows or returns standard install directories on Linux.
pub fn find_steam_install_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let steam_key = hkcu.open_subkey(r"Software\Valve\Steam").ok()?;
        let path: String = steam_key.get_value("SteamPath").ok()?;
        Some(PathBuf::from(path))
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Standard Linux Steam path
        let home = dirs::home_dir()?;
        let standard_path = home.join(".local/share/Steam");
        if standard_path.exists() {
            Some(standard_path)
        } else {
            None
        }
    }
}

/// Parses the Steam `libraryfolders.vdf` file to identify additional Steam Library folders.
pub fn parse_library_folders(steam_dir: &std::path::Path) -> Vec<PathBuf> {
    let vdf_path = steam_dir.join("steamapps/libraryfolders.vdf");
    if !vdf_path.exists() {
        return vec![];
    }
    
    let contents = match std::fs::read_to_string(&vdf_path) {
        Ok(s) => s,
        Err(_) => return vec![],
    };
    
    let mut libraries = vec![];
    // Simple line-by-line parsing looking for: "path" "Path/To/Library"
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("\"path\"") {
            let parts: Vec<&str> = trimmed.split('"').collect();
            if parts.len() >= 5 {
                let raw_path = parts[3];
                // Replace double backslashes in Windows paths
                let cleaned_path = raw_path.replace("\\\\", "\\");
                libraries.push(PathBuf::from(cleaned_path));
            }
        }
    }
    libraries
}

/// Automatically searches all Steam Library paths to find Slay the Spire.
pub fn auto_detect_sts_path() -> Option<PathBuf> {
    let steam_dir = find_steam_install_dir()?;
    let mut search_paths = vec![steam_dir.clone()];
    
    // Read additional library folders
    search_paths.extend(parse_library_folders(&steam_dir));
    
    for path in search_paths {
        let sts_candidate = path.join("steamapps/common/SlayTheSpire");
        if sts_candidate.exists() && sts_candidate.join("desktop-1.0.jar").exists() {
            return Some(sts_candidate);
        }
    }
    None
}
```

#### File: `src/scanner.rs`
```rust
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::steam::parse_library_folders;
use sts_modloader_core::ModInfo;
use sts_modloader_parser::metadata::{parse_jar_metadata, ModSource};

/// Scans both the local game mods directory and Steam Workshop directory.
pub async fn scan_mods(sts_path: &Path) -> Result<Vec<ModInfo>, String> {
    let local_mods_dir = sts_path.join("mods");
    let mut mods = vec![];

    // 1. Scan Local Mods Folder
    if local_mods_dir.exists() {
        let local_jar_paths = list_jars_in_dir(&local_mods_dir);
        for jar in local_jar_paths {
            if let Ok(mod_info) = parse_jar_metadata(&jar, ModSource::Local) {
                mods.push(mod_info);
            }
        }
    }

    // 2. Scan Workshop folder relative to STS directory structure
    // Path layout: <SteamLibrary>/steamapps/common/SlayTheSpire
    // Workshop layout: <SteamLibrary>/steamapps/workshop/content/646570/
    if let Some(steam_library_root) = sts_path.parent().and_then(|p| p.parent()) {
        let workshop_dir = steam_library_root.join("workshop/content/646570");
        if workshop_dir.exists() {
            let workshop_jar_paths = list_jars_in_dir(&workshop_dir);
            for jar in workshop_jar_paths {
                if let Ok(mod_info) = parse_jar_metadata(&jar, ModSource::Workshop) {
                    mods.push(mod_info);
                }
            }
        }
    }

    // De-duplicate: If a mod is present in both local and workshop, local takes precedence
    mods.sort_by(|a, b| a.id.cmp(&b.id));
    let mut deduped: Vec<ModInfo> = vec![];
    for m in mods {
        if let Some(existing) = deduped.iter_mut().find(|x| x.id == m.id) {
            // Local overrides workshop
            if m.source == ModSource::Local && existing.source == ModSource::Workshop {
                *existing = m;
            }
        } else {
            deduped.push(m);
        }
    }

    Ok(deduped)
}

fn list_jars_in_dir(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .max_depth(3) // Allow subdirectories for workshop contents
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file() && e.path().extension().map_or(false, |ext| ext == "jar"))
        .map(|e| e.into_path())
        .collect()
}
```

#### File: `src/runner.rs`
```rust
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum LaunchError {
    #[error("Java executable could not be found. Ensure Java is installed and added to the PATH system variable.")]
    JavaNotFound,
    #[error("ModTheSpire.jar was not found in the game folder: {0}")]
    MtsJarNotFound(PathBuf),
    #[error("Java process terminated with exit code {code}.\nStderr: {stderr}")]
    ProcessFailed { code: i32, stderr: String },
    #[error("IO Error occurred: {0}")]
    Io(String),
}

/// Writes enabled mods to the system config and launches the Java process.
pub async fn launch_game(
    sts_path: &Path,
    enabled_mods: &[String], // List of enabled mod IDs
    debug_mode: bool,
) -> Result<(), LaunchError> {
    let mts_jar = sts_path.join("ModTheSpire.jar");
    if !mts_jar.exists() {
        return Err(LaunchError::MtsJarNotFound(mts_jar));
    }

    // Write selection properties to %LOCALAPPDATA%/ModTheSpire/ModTheSpire.properties
    write_mts_properties(enabled_mods, debug_mode).map_err(|e| LaunchError::Io(e.to_string()))?;

    // Prepare java execution command
    let mut cmd = Command::new("java");
    cmd.arg("-jar")
       .arg("ModTheSpire.jar")
       .arg("--skip-launcher");
       
    if debug_mode {
        cmd.arg("--debug");
    }
    
    // Set the game path as Working Directory (CWD)
    cmd.current_dir(sts_path);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            LaunchError::JavaNotFound
        } else {
            LaunchError::Io(e.to_string())
        }
    })?;

    // Wait for the game process to exit
    let output = child.wait_with_output().await.map_err(|e| LaunchError::Io(e.to_string()))?;

    if !output.status.success() {
        let code = output.status.code().unwrap_or(-1);
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(LaunchError::ProcessFailed { code, stderr });
    }

    Ok(())
}

fn write_mts_properties(enabled_mods: &[String], debug_mode: bool) -> std::io::Result<()> {
    let local_appdata = dirs::cache_dir() // or resolve via local_data_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "AppData path could not be resolved"))?;
    
    // Windows path fallback for %LOCALAPPDATA%
    let mts_pref_dir = local_appdata.join("ModTheSpire");
    std::fs::create_dir_all(&mts_pref_dir)?;
    let pref_file = mts_pref_dir.join("ModTheSpire.properties");

    let mods_csv = enabled_mods.join(",");
    let content = format!(
        "# Written by Slay the Spire Mod Loader\n\
         mods={}\n\
         debug={}\n\
         debug_mode={}\n",
        mods_csv, debug_mode, debug_mode
    );

    std::fs::write(pref_file, content)?;
    Ok(())
}
```

### 4.4 `sts_modloader_profile`
Contains the CRUD business logic of mod selection profiles.

#### File: `src/manager.rs`
```rust
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
```

---

## 5. UI Message Types & State Machine

The interface module `sts_modloader_ui` manages Iced state and Elm-style updates.

### 5.1 Message Types
```rust
use std::path::PathBuf;
use sts_modloader_core::ModInfo;
use sts_modloader_fs::runner::LaunchError;

#[derive(Debug, Clone)]
pub enum Message {
    // Application Initialization
    Init,
    ConfigLoaded(Result<sts_modloader_core::AppConfig, String>),
    
    // Slay the Spire Path Management
    AutoDetectPath,
    PathDetected(Option<PathBuf>),
    BrowsePath,
    PathSelected(PathBuf),
    
    // Mod Scan Operations
    ScanMods,
    ScanFinished(Result<Vec<ModInfo>, String>),
    
    // Active Mod Management
    SelectMod(String),        // Highlights a mod for the Right Panel (ID)
    ToggleMod(String),        // Checks/Unchecks a mod in the List (ID)
    ToggleAllMods(bool),      // Enables/Disables all currently filtered mods
    SearchQueryChanged(String),
    
    // Profiles
    SelectProfile(String),
    OpenNewProfileDialog,     // Toggles text input field display
    ProfileNameInput(String),
    CreateNewProfile,
    DeleteActiveProfile,
    
    // Launch Operations
    LaunchGame,
    GameExited(Result<(), LaunchError>),
    ToggleDebug(bool),
    
    // Modal controls
    DismissError,
}
```

### 5.2 Application State Model
```rust
use sts_modloader_core::AppConfig;
use sts_modloader_core::ModInfo;

pub struct AppState {
    pub config: AppConfig,
    pub mods: Vec<ModInfo>,
    
    // UI Filters and Selectors
    pub selected_mod_id: Option<String>,
    pub search_query: String,
    
    // Modals & Overlay States
    pub error_modal: Option<String>,
    pub show_profile_input: bool,
    pub new_profile_name: String,
    pub is_loading: bool,
    pub is_game_running: bool,
}
```

---

## 6. Interaction Flows & Thread Safety

The following diagrams and specifications detail the lifecycle and thread boundaries of the loader:

1. **Initial Boot-up Flow**:
   - `Init` is dispatched. UI initiates a Tokio task to load the JSON configuration from disk.
   - If `AppConfig.sts_path` is empty or invalid, the UI displays the Setup panel, prompting the user for directory detection.
   - Once a valid Slay the Spire directory is resolved, the configuration is saved, and a mod scan is triggered.
   - The scan returns all local and workshop jar metadata. The UI updates the active list, filtering by any saved profile enabled-mods list.

2. **Search and Toggle Flow**:
   - Searching performs character case-insensitive filtering on `name` or `id`.
   - Clicking "Toggle All" updates the active state of all *currently displayed* filtered mods, keeping non-filtered ones unchanged.
   
3. **Execution Flow**:
   - Clicking "Play" writes the enabled list to `ModTheSpire.properties` and spawns `java -jar ModTheSpire.jar --skip-launcher`.
   - The UI thread yields execution to the tokio worker thread to await the process completion. The UI goes into a locked "Running" state.
   - When the process completes, it sends the status back via `GameExited(Result)`. If a non-zero exit code or starting failure occurs, the UI displays a dark modal overlay showing the stderr dump to the user.

---

## 7. UI Mockup & Styles Specification

The loader features a compact dark theme design layout (`1024x600` px window).

### 7.1 Visual Layout Blueprint

```text
+---------------------------------------------------------------------------------------------------+
|  [Profile Selection: PickList]   [+] [-]                     [Refresh List Button]  [Status: OK]   |
+---------------------------------------------------------------------------------------------------+
|  SEARCH: [  Filter mods...      ] |  MOD DETAILS:                                                 |
|                                  |  ============================================================  |
|  [X] BaseMod              v5.44  |  BaseMod (v5.44.0)                                             |
|  [X] Stslib               v2.10  |  Authors: t-larson, kiooeht, daviscook                         |
|  [ ] Downfall             v4.20  |  Target Game Version: 2.0  | MTS version: 3.18                 |
|  [ ] RelicStats           v2.01  |  ------------------------------------------------------------  |
|                                  |  An API for modding Slay the Spire. Adds custom cards, relics, |
|                                  |  events, campaigns, and debug facilities.                      |
|                                  |                                                                |
|                                  |  Dependencies:                                                 |
|                                  |  - ModTheSpire  [Satisfied]                                    |
|                                  |  - Stslib       [Warning: Disabled]                            |
+---------------------------------------------------------------------------------------------------+
|  [X] Enable Debug Console Mode                        |               [ PLAY GAME BUTTON ]        |
+---------------------------------------------------------------------------------------------------+
```

### 7.2 Styling Variables (Dark Theme)

The UI must use a uniform color palette using custom Iced style containers:

| Component | Target Color (Hex) | Purpose |
|---|---|---|
| **Primary Background** | `#181824` | Main window background |
| **Panel Background** | `#20202F` | Left and right panel boundaries |
| **Text Foreground** | `#E2E8F0` | High contrast readable content text |
| **Muted Text** | `#94A3B8` | Minor information, version codes, authors |
| **Accent Active** | `#6366F1` | Primary action buttons, active profile focus, selected row borders |
| **Play Button (Normal)** | `#10B981` | Safe green CTA button |
| **Play Button (Running)**| `#EF4444` | Red launch cancel or status indication |
| **Borders** | `#334155` | Borders and dividers |

---

## 8. Mod Dependency & Conflict Resolver Logic

To guarantee a clean game launch, a mod validation step should be run inside the UI module whenever the active list of mods changes.

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyStatus {
    Satisfied,
    Disabled,   // Mod is present but checkbox is unchecked
    Missing,    // Mod is completely absent from local/workshop directory
}

/// Validates dependencies for all selected mods
pub fn validate_mods_dependencies(
    all_mods: &[ModInfo],
) -> HashMap<String, Vec<(String, DependencyStatus)>> {
    let mut status_map = HashMap::new();
    let enabled_ids: Vec<String> = all_mods
        .iter()
        .filter(|m| m.enabled)
        .map(|m| m.id.clone())
        .collect();

    for m in all_mods {
        let mut deps_status = vec![];
        for dep_id in &m.dependencies {
            // ModTheSpire is the loader itself, satisfied implicitly
            if dep_id == "ModTheSpire" {
                deps_status.push((dep_id.clone(), DependencyStatus::Satisfied));
                continue;
            }

            let found_mod = all_mods.iter().find(|x| x.id == *dep_id);
            match found_mod {
                None => {
                    deps_status.push((dep_id.clone(), DependencyStatus::Missing));
                }
                Some(dm) => {
                    if dm.enabled {
                        deps_status.push((dep_id.clone(), DependencyStatus::Satisfied));
                    } else {
                        deps_status.push((dep_id.clone(), DependencyStatus::Disabled));
                    }
                }
            }
        }
        status_map.insert(m.id.clone(), deps_status);
    }
    
    status_map
}
```

This validation logic must be used to show warning badges in the right details panel, helping users troubleshoot why their mod configurations might fail to load.
