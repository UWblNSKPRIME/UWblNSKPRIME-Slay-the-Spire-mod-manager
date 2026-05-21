use std::path::Path;
use serde::Deserialize;
use thiserror::Error;
use sts_modloader_core::{ModInfo, ModSource};

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

fn normalize_authors(value: Option<serde_json::Value>) -> Vec<String> {
    match value {
        None => vec![],
        Some(val) => match val {
            serde_json::Value::String(s) => {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    vec![]
                } else if trimmed.contains(',') {
                    trimmed
                        .split(',')
                        .map(|a| a.trim().to_string())
                        .filter(|a| !a.is_empty())
                        .collect()
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
        },
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use zip::write::FileOptions;

    static JAR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn create_temp_jar(json_content: Option<&str>) -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let count = JAR_COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = temp_dir.join(format!("test_mod_{}_{}.jar", std::process::id(), count));

        let file = fs::File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);

        if let Some(content) = json_content {
            zip.start_file("ModTheSpire.json", FileOptions::default()).unwrap();
            std::io::Write::write_all(&mut zip, content.as_bytes()).unwrap();
        } else {
            zip.start_file("some_other_file.txt", FileOptions::default()).unwrap();
            std::io::Write::write_all(&mut zip, b"hello").unwrap();
        }

        zip.finish().unwrap();
        path
    }

    #[test]
    fn test_parse_jar_metadata_success_normal() {
        let json = r#"{
            "modid": "testmod",
            "name": "Test Mod",
            "author": "Alice",
            "version": "1.2.3",
            "description": "A test mod",
            "dependencies": ["dependency1", "dependency2"],
            "stsVersion": "2.0",
            "mtsVersion": "3.8.0"
        }"#;

        let jar_path = create_temp_jar(Some(json));
        let res = parse_jar_metadata(&jar_path, ModSource::Local);
        let _ = fs::remove_file(&jar_path);

        let info = res.expect("Should parse successfully");
        assert_eq!(info.id, "testmod");
        assert_eq!(info.name, "Test Mod");
        assert_eq!(info.authors, vec!["Alice"]);
        assert_eq!(info.version, "1.2.3");
        assert_eq!(info.description, Some("A test mod".to_string()));
        assert_eq!(info.dependencies, vec!["dependency1", "dependency2"]);
        assert_eq!(info.sts_version, Some("2.0".to_string()));
        assert_eq!(info.mts_version, Some("3.8.0".to_string()));
        assert_eq!(info.source, ModSource::Local);
        assert!(!info.enabled);
    }

    #[test]
    fn test_parse_jar_metadata_weird_authors() {
        // authors as comma-separated string
        let json_comma = r#"{
            "modid": "testmod",
            "name": "Test Mod",
            "authors": "Alice, Bob, Charlie",
            "version": "1.0.0"
        }"#;
        let jar_path = create_temp_jar(Some(json_comma));
        let info = parse_jar_metadata(&jar_path, ModSource::Workshop).unwrap();
        let _ = fs::remove_file(&jar_path);
        assert_eq!(info.authors, vec!["Alice", "Bob", "Charlie"]);

        // authors as json array
        let json_array = r#"{
            "modid": "testmod",
            "name": "Test Mod",
            "author_list": ["Alice", "Bob"],
            "version": "1.0.0"
        }"#;
        let jar_path2 = create_temp_jar(Some(json_array));
        let info2 = parse_jar_metadata(&jar_path2, ModSource::Workshop).unwrap();
        let _ = fs::remove_file(&jar_path2);
        assert_eq!(info2.authors, vec!["Alice", "Bob"]);

        // authors missing or invalid
        let json_missing = r#"{
            "modid": "testmod",
            "name": "Test Mod",
            "version": "1.0.0"
        }"#;
        let jar_path3 = create_temp_jar(Some(json_missing));
        let info3 = parse_jar_metadata(&jar_path3, ModSource::Workshop).unwrap();
        let _ = fs::remove_file(&jar_path3);
        assert!(info3.authors.is_empty());
    }

    #[test]
    fn test_parse_jar_metadata_not_found() {
        let jar_path = create_temp_jar(None);
        let res = parse_jar_metadata(&jar_path, ModSource::Local);
        let _ = fs::remove_file(&jar_path);

        assert!(matches!(res, Err(ParserError::MetadataNotFound)));
    }

    #[test]
    fn test_parse_jar_metadata_invalid_json() {
        let jar_path = create_temp_jar(Some("invalid json"));
        let res = parse_jar_metadata(&jar_path, ModSource::Local);
        let _ = fs::remove_file(&jar_path);

        assert!(matches!(res, Err(ParserError::Json(_))));
    }

    #[test]
    fn test_parse_jar_metadata_missing_file() {
        let jar_path = PathBuf::from("this_file_does_not_exist.jar");
        let res = parse_jar_metadata(&jar_path, ModSource::Local);
        assert!(matches!(res, Err(ParserError::Io(_))));
    }
}
