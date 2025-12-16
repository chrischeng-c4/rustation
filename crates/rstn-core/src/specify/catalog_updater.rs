//! Features catalog management
//!
//! Manages the `specs/features.json` file that tracks all features
//! in the project.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{NewFeature, SpecifyError};

/// The features.json catalog structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesCatalog {
    /// Project name
    pub project: String,

    /// Project description
    pub description: String,

    /// List of features
    pub features: Vec<FeatureEntry>,
}

impl FeaturesCatalog {
    /// Find the highest feature number in the catalog
    pub fn max_number(&self) -> u32 {
        self.features
            .iter()
            .filter_map(|f| f.id.parse::<u32>().ok())
            .max()
            .unwrap_or(0)
    }

    /// Check if a feature number already exists
    pub fn has_number(&self, num: &str) -> bool {
        self.features.iter().any(|f| f.id == num)
    }
}

/// A single feature entry in the catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureEntry {
    /// Feature ID (e.g., "052")
    pub id: String,

    /// Feature short name (e.g., "internalize-spec-generation")
    pub name: String,

    /// Feature description
    pub description: String,

    /// Status: "draft", "in-progress", "complete"
    pub status: String,

    /// Phase number (typically 1)
    pub phase: u32,
}

/// Read the features catalog from disk
///
/// Returns an empty catalog if the file doesn't exist.
pub fn read_features_catalog(workspace_root: &Path) -> Result<FeaturesCatalog, SpecifyError> {
    let catalog_path = workspace_root.join("specs").join("features.json");

    if !catalog_path.exists() {
        return Ok(FeaturesCatalog {
            project: "unknown".to_string(),
            description: "Project features".to_string(),
            features: vec![],
        });
    }

    let content = fs::read_to_string(&catalog_path).map_err(SpecifyError::CatalogRead)?;

    serde_json::from_str(&content).map_err(SpecifyError::CatalogParse)
}

/// Update the features catalog with a new feature entry
///
/// This function:
/// 1. Reads the existing catalog
/// 2. Adds the new feature entry
/// 3. Writes atomically using temp file + rename
pub fn update_features_catalog(
    workspace_root: &Path,
    feature: &NewFeature,
) -> Result<(), SpecifyError> {
    let catalog_path = workspace_root.join("specs").join("features.json");

    // Read existing catalog
    let mut catalog = read_features_catalog(workspace_root)?;

    // Add new entry
    catalog.features.push(FeatureEntry {
        id: feature.number.clone(),
        name: feature.name.clone(),
        description: feature.title.clone(),
        status: "draft".to_string(),
        phase: 1,
    });

    // Write atomically: temp file + rename
    let temp_path = catalog_path.with_extension("json.tmp");
    let content = serde_json::to_string_pretty(&catalog).map_err(|e| {
        SpecifyError::CatalogWrite(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        ))
    })?;

    fs::write(&temp_path, &content).map_err(SpecifyError::CatalogWrite)?;

    fs::rename(&temp_path, &catalog_path).map_err(SpecifyError::CatalogWrite)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_read_empty_catalog() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("specs")).unwrap();

        let catalog = read_features_catalog(temp.path()).unwrap();
        assert!(catalog.features.is_empty());
    }

    #[test]
    fn test_read_existing_catalog() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let catalog_content = r#"{
            "project": "test",
            "description": "Test project",
            "features": [
                {"id": "001", "name": "first", "description": "First", "status": "complete", "phase": 1}
            ]
        }"#;
        fs::write(specs_dir.join("features.json"), catalog_content).unwrap();

        let catalog = read_features_catalog(temp.path()).unwrap();
        assert_eq!(catalog.features.len(), 1);
        assert_eq!(catalog.features[0].id, "001");
    }

    #[test]
    fn test_catalog_max_number() {
        let catalog = FeaturesCatalog {
            project: "test".to_string(),
            description: "test".to_string(),
            features: vec![
                FeatureEntry {
                    id: "001".to_string(),
                    name: "first".to_string(),
                    description: "First".to_string(),
                    status: "complete".to_string(),
                    phase: 1,
                },
                FeatureEntry {
                    id: "010".to_string(),
                    name: "tenth".to_string(),
                    description: "Tenth".to_string(),
                    status: "draft".to_string(),
                    phase: 1,
                },
            ],
        };

        assert_eq!(catalog.max_number(), 10);
    }

    #[test]
    fn test_catalog_has_number() {
        let catalog = FeaturesCatalog {
            project: "test".to_string(),
            description: "test".to_string(),
            features: vec![FeatureEntry {
                id: "001".to_string(),
                name: "first".to_string(),
                description: "First".to_string(),
                status: "complete".to_string(),
                phase: 1,
            }],
        };

        assert!(catalog.has_number("001"));
        assert!(!catalog.has_number("002"));
    }

    #[test]
    fn test_update_catalog() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        // Create initial catalog
        let initial = r#"{
            "project": "test",
            "description": "Test project",
            "features": []
        }"#;
        fs::write(specs_dir.join("features.json"), initial).unwrap();

        let feature = NewFeature {
            number: "001".to_string(),
            name: "test-feature".to_string(),
            title: "Test Feature".to_string(),
            description: "A test".to_string(),
        };

        update_features_catalog(temp.path(), &feature).unwrap();

        let catalog = read_features_catalog(temp.path()).unwrap();
        assert_eq!(catalog.features.len(), 1);
        assert_eq!(catalog.features[0].id, "001");
        assert_eq!(catalog.features[0].name, "test-feature");
        assert_eq!(catalog.features[0].status, "draft");
    }

    #[test]
    fn test_update_preserves_existing() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let initial = r#"{
            "project": "test",
            "description": "Test project",
            "features": [
                {"id": "001", "name": "first", "description": "First", "status": "complete", "phase": 1}
            ]
        }"#;
        fs::write(specs_dir.join("features.json"), initial).unwrap();

        let feature = NewFeature {
            number: "002".to_string(),
            name: "second".to_string(),
            title: "Second".to_string(),
            description: "Second feature".to_string(),
        };

        update_features_catalog(temp.path(), &feature).unwrap();

        let catalog = read_features_catalog(temp.path()).unwrap();
        assert_eq!(catalog.features.len(), 2);
        assert_eq!(catalog.features[0].id, "001");
        assert_eq!(catalog.features[1].id, "002");
    }
}
