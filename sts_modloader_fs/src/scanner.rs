use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use sts_modloader_core::{ModInfo, ModSource};
use sts_modloader_parser::metadata::parse_jar_metadata;

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
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().map_or(false, |ext| ext == "jar")
        })
        .map(|e| e.into_path())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use zip::write::FileOptions;

    fn create_mock_mod_jar(path: &Path, modid: &str, name: &str, author: &str, version: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let file = fs::File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("ModTheSpire.json", FileOptions::default()).unwrap();
        let json = format!(
            r#"{{"modid": "{}", "name": "{}", "author": "{}", "version": "{}"}}"#,
            modid, name, author, version
        );
        std::io::Write::write_all(&mut zip, json.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    #[tokio::test]
    async fn test_scan_mods_success_and_deduplication() {
        let temp_base = std::env::temp_dir().join(format!("test_scanner_{}", std::process::id()));
        
        let sts_path = temp_base.join("steamapps/common/SlayTheSpire");
        let local_mods_dir = sts_path.join("mods");
        let workshop_dir = temp_base.join("steamapps/workshop/content/646570");

        // 1. Create a mod that is in both local and workshop (local should override)
        let local_dup_jar = local_mods_dir.join("dup_mod.jar");
        create_mock_mod_jar(&local_dup_jar, "dup-id", "Duplicate Mod", "LocalAuthor", "2.0.0");

        let workshop_dup_jar = workshop_dir.join("dup_mod_folder/dup_mod.jar");
        create_mock_mod_jar(&workshop_dup_jar, "dup-id", "Duplicate Mod", "WorkshopAuthor", "1.0.0");

        // 2. Create local-only mod
        let local_only_jar = local_mods_dir.join("local_only.jar");
        create_mock_mod_jar(&local_only_jar, "local-only-id", "Local Only", "LocalOnlyAuthor", "1.0.0");

        // 3. Create workshop-only mod
        let workshop_only_jar = workshop_dir.join("workshop_only_folder/workshop_only.jar");
        create_mock_mod_jar(&workshop_only_jar, "workshop-only-id", "Workshop Only", "WorkshopAuthor", "1.5.0");

        // Run the scanner!
        let mods_res = scan_mods(&sts_path).await;
        
        // Clean up
        let _ = fs::remove_dir_all(&temp_base);

        let mods = mods_res.expect("Scan should succeed");
        assert_eq!(mods.len(), 3);

        // Deduplicated duplicate mod (local should override workshop)
        let dup = mods.iter().find(|m| m.id == "dup-id").expect("dup-id mod should exist");
        assert_eq!(dup.source, ModSource::Local);
        assert_eq!(dup.authors, vec!["LocalAuthor"]);
        assert_eq!(dup.version, "2.0.0");

        // Local only mod
        let local_only = mods.iter().find(|m| m.id == "local-only-id").expect("local-only-id mod should exist");
        assert_eq!(local_only.source, ModSource::Local);
        assert_eq!(local_only.authors, vec!["LocalOnlyAuthor"]);

        // Workshop only mod
        let workshop_only = mods.iter().find(|m| m.id == "workshop-only-id").expect("workshop-only-id mod should exist");
        assert_eq!(workshop_only.source, ModSource::Workshop);
        assert_eq!(workshop_only.authors, vec!["WorkshopAuthor"]);
    }
}
