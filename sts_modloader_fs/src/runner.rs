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

/// Attempts to locate ModTheSpire.jar, checking first in the game directory,
/// and then in the Steam Workshop directory under content id 646570/1605060445.
pub fn locate_mts_jar(sts_path: &Path) -> Option<PathBuf> {
    // 1. Check local game folder
    let local_mts = sts_path.join("ModTheSpire.jar");
    if local_mts.exists() {
        return Some(local_mts);
    }

    // 2. Check Steam workshop content directory for ModTheSpire (Workshop ID: 1605060445)
    // sts_path is typically .../steamapps/common/SlayTheSpire
    // steamapps root is 2 directories up
    if let Some(steamapps) = sts_path.parent().and_then(|p| p.parent()) {
        let workshop_mts = steamapps.join("workshop/content/646570/1605060445/ModTheSpire.jar");
        if workshop_mts.exists() {
            return Some(workshop_mts);
        }
    }

    None
}

/// Attempts to locate the Java executable on the host system.
/// Checks JAVA_HOME, Windows Registry keys (for Windows), common directories, and the PATH env var.
pub fn find_java_path() -> PathBuf {
    // 1. Check JAVA_HOME environment variable
    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let home_path = PathBuf::from(java_home);
        let exe = if cfg!(target_os = "windows") {
            home_path.join("bin").join("java.exe")
        } else {
            home_path.join("bin").join("java")
        };
        if exe.exists() {
            return exe;
        }
    }

    // 2. Windows registry check
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let keys_to_check = [
            r"SOFTWARE\JavaSoft\JDK",
            r"SOFTWARE\JavaSoft\Java Runtime Environment",
            r"SOFTWARE\JavaSoft\Java Development Kit",
        ];
        
        let check_keys = |predef| {
            let mut found = Vec::new();
            let base_key = RegKey::predef(predef);
            for parent_path in &keys_to_check {
                if let Ok(key) = base_key.open_subkey(parent_path) {
                    if let Ok(subkeys) = key.enum_keys().collect::<Result<Vec<_>, _>>() {
                        for name in subkeys {
                            if let Ok(sub) = key.open_subkey(&name) {
                                if let Ok(home) = sub.get_value::<String, _>("JavaHome") {
                                    let p = PathBuf::from(home).join("bin").join("java.exe");
                                    if p.exists() {
                                        found.push(p);
                                    }
                                }
                            }
                        }
                    }
                    if let Ok(version) = key.get_value::<String, _>("CurrentVersion") {
                        if let Ok(sub) = key.open_subkey(&version) {
                            if let Ok(home) = sub.get_value::<String, _>("JavaHome") {
                                let p = PathBuf::from(home).join("bin").join("java.exe");
                                if p.exists() {
                                    found.push(p);
                                }
                            }
                        }
                    }
                }
            }
            found
        };
        
        for predef in &[HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER] {
            let found = check_keys(*predef);
            if let Some(p) = found.into_iter().next() {
                return p;
            }
        }
    }

    // 3. Check standard search locations
    #[cfg(target_os = "windows")]
    {
        let common_paths = [
            r"C:\Program Files\Common Files\Oracle\Java\javapath\java.exe",
            r"C:\ProgramData\Oracle\Java\javapath\java.exe",
        ];
        for p in &common_paths {
            let path = PathBuf::from(p);
            if path.exists() {
                return path;
            }
        }

        // Check Program Files directory structures
        let java_dirs = [
            r"C:\Program Files\Java",
            r"C:\Program Files (x86)\Java",
        ];
        for java_dir in &java_dirs {
            if let Ok(entries) = std::fs::read_dir(java_dir) {
                for entry in entries.flatten() {
                    let p = entry.path().join("bin").join("java.exe");
                    if p.exists() {
                        return p;
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let common_paths = [
            "/usr/bin/java",
            "/usr/local/bin/java",
            "/usr/lib/jvm/default/bin/java",
        ];
        for p in &common_paths {
            let path = PathBuf::from(p);
            if path.exists() {
                return path;
            }
        }
    }

    // 4. Check PATH environment variable manually
    if let Ok(path_var) = std::env::var("PATH") {
        let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
        let exe_name = if cfg!(target_os = "windows") { "java.exe" } else { "java" };
        for dir in path_var.split(separator) {
            let path = PathBuf::from(dir).join(exe_name);
            if path.exists() {
                return path;
            }
        }
    }

    // Default fallback
    PathBuf::from(if cfg!(target_os = "windows") { "java.exe" } else { "java" })
}

/// Writes enabled mods to the system config and launches the Java process.
pub async fn launch_game(
    sts_path: &Path,
    enabled_mods: &[String], // List of enabled mod IDs
    debug_mode: bool,
) -> Result<(), LaunchError> {
    let mts_jar = locate_mts_jar(sts_path)
        .ok_or_else(|| LaunchError::MtsJarNotFound(sts_path.join("ModTheSpire.jar")))?;

    // Write selection properties to ModTheSpire.properties
    write_mts_properties(enabled_mods, debug_mode).map_err(|e| LaunchError::Io(e.to_string()))?;

    // Prepare java execution command using the auto-located Java executable path
    let java_path = find_java_path();
    let mut cmd = Command::new(java_path);
    cmd.arg("-jar")
       .arg(&mts_jar)
       .arg("--skip-launcher");
       
    if debug_mode {
        cmd.arg("--debug");
    }
    
    // Set the game path as Working Directory (CWD)
    cmd.current_dir(sts_path);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let child = cmd.spawn().map_err(|e| {
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
    let base_dir = if cfg!(target_os = "windows") {
        dirs::data_local_dir()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Local AppData path could not be resolved",
                )
            })?
    } else {
        dirs::config_dir()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Config path could not be resolved",
                )
            })?
    };
    let mts_pref_dir = base_dir.join("ModTheSpire");
    write_mts_properties_to_dir(&mts_pref_dir, enabled_mods, debug_mode)
}

fn write_mts_properties_to_dir(
    mts_pref_dir: &Path,
    enabled_mods: &[String],
    debug_mode: bool,
) -> std::io::Result<()> {
    std::fs::create_dir_all(mts_pref_dir)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test]
    async fn test_launch_game_missing_mts_jar() {
        let temp_dir = std::env::temp_dir().join(format!("test_launch_missing_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        let res = launch_game(&temp_dir, &[], false).await;
        fs::remove_dir_all(&temp_dir).unwrap();

        assert!(matches!(res, Err(LaunchError::MtsJarNotFound(_))));
    }

    #[test]
    fn test_locate_mts_jar() {
        let temp_dir = std::env::temp_dir().join(format!("test_locate_{}", std::process::id()));
        let sts_dir = temp_dir.join("steamapps/common/SlayTheSpire");
        fs::create_dir_all(&sts_dir).unwrap();

        // 1. None should be found initially
        assert!(locate_mts_jar(&sts_dir).is_none());

        // 2. Local jar found
        let local_mts_file = sts_dir.join("ModTheSpire.jar");
        fs::write(&local_mts_file, "mock jar content").unwrap();
        assert_eq!(locate_mts_jar(&sts_dir), Some(local_mts_file.clone()));

        // Remove local and test workshop fallback
        fs::remove_file(&local_mts_file).unwrap();

        // 3. Workshop jar found
        let workshop_dir = temp_dir.join("steamapps/workshop/content/646570/1605060445");
        fs::create_dir_all(&workshop_dir).unwrap();
        let workshop_mts_file = workshop_dir.join("ModTheSpire.jar");
        fs::write(&workshop_mts_file, "mock workshop content").unwrap();

        assert_eq!(locate_mts_jar(&sts_dir), Some(workshop_mts_file));

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_write_mts_properties_to_dir() {
        let temp_dir = std::env::temp_dir().join(format!("test_write_properties_{}", std::process::id()));
        fs::create_dir_all(&temp_dir).unwrap();

        let enabled_mods = vec!["ModA".to_string(), "ModB".to_string()];
        let res = write_mts_properties_to_dir(&temp_dir, &enabled_mods, true);
        assert!(res.is_ok());

        let pref_file = temp_dir.join("ModTheSpire.properties");
        assert!(pref_file.exists());

        let contents = fs::read_to_string(&pref_file).unwrap();
        fs::remove_dir_all(&temp_dir).unwrap();

        assert!(contents.contains("mods=ModA,ModB"));
        assert!(contents.contains("debug=true"));
        assert!(contents.contains("debug_mode=true"));
    }

    #[test]
    fn test_find_java() {
        let java = find_java_path();
        assert!(!java.to_string_lossy().is_empty());
    }
}
