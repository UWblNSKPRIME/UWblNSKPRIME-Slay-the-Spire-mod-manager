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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_library_folders_missing() {
        let temp_dir = std::env::temp_dir().join(format!("test_vdf_missing_{}", std::process::id()));
        let res = parse_library_folders(&temp_dir);
        assert!(res.is_empty());
    }

    #[test]
    fn test_parse_library_folders_success() {
        let temp_dir = std::env::temp_dir().join(format!("test_vdf_success_{}", std::process::id()));
        let steamapps_dir = temp_dir.join("steamapps");
        fs::create_dir_all(&steamapps_dir).unwrap();

        let vdf_content = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"label"		""
		"contentid"		"84318712398"
	}
	"1"
	{
		"path"		"D:\\Games\\SteamLibrary"
		"label"		""
		"contentid"		"21798371238"
	}
}
"#;
        fs::write(steamapps_dir.join("libraryfolders.vdf"), vdf_content).unwrap();

        let res = parse_library_folders(&temp_dir);
        let _ = fs::remove_dir_all(&temp_dir);

        assert_eq!(res.len(), 2);
        assert_eq!(res[0], PathBuf::from("C:\\Program Files (x86)\\Steam"));
        assert_eq!(res[1], PathBuf::from("D:\\Games\\SteamLibrary"));
    }
}
