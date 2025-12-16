//! Feature number allocation
//!
//! Allocates unique feature numbers by reading from both features.json
//! and scanning the specs/ directory to find the highest existing number.

use std::fs;
use std::path::Path;

use super::SpecifyError;
use crate::specify::catalog_updater::FeaturesCatalog;

/// Allocate the next available feature number
///
/// This function checks both:
/// 1. The features.json catalog
/// 2. The specs/ directory for existing feature directories
///
/// It returns the maximum of both sources + 1, formatted as a zero-padded 3-digit string.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root (where specs/ directory is located)
///
/// # Returns
///
/// A zero-padded 3-digit string (e.g., "053")
pub fn allocate_feature_number(workspace_root: &Path) -> Result<String, SpecifyError> {
    let specs_dir = workspace_root.join("specs");

    // Get max from catalog
    let catalog_max = get_max_from_catalog(&specs_dir)?;

    // Get max from directory scan
    let dir_max = get_max_from_directory(&specs_dir)?;

    // Take the maximum of both sources
    let next_number = catalog_max.max(dir_max) + 1;

    // Format as zero-padded 3-digit string
    Ok(format!("{:03}", next_number))
}

/// Get the highest feature number from features.json
fn get_max_from_catalog(specs_dir: &Path) -> Result<u32, SpecifyError> {
    let catalog_path = specs_dir.join("features.json");

    if !catalog_path.exists() {
        return Ok(0);
    }

    let content = fs::read_to_string(&catalog_path).map_err(SpecifyError::CatalogRead)?;

    let catalog: FeaturesCatalog =
        serde_json::from_str(&content).map_err(SpecifyError::CatalogParse)?;

    Ok(catalog.max_number())
}

/// Get the highest feature number from directory scan
fn get_max_from_directory(specs_dir: &Path) -> Result<u32, SpecifyError> {
    if !specs_dir.exists() {
        return Ok(0);
    }

    let mut max_number: u32 = 0;

    let entries = fs::read_dir(specs_dir).map_err(|e| {
        SpecifyError::NumberAllocation(format!("Cannot read specs directory: {}", e))
    })?;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Extract leading digits from directory name (e.g., "052-feature-name" -> 52)
        if let Some(num_str) = name_str.split('-').next() {
            if let Ok(num) = num_str.parse::<u32>() {
                max_number = max_number.max(num);
            }
        }
    }

    Ok(max_number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_allocate_empty_workspace() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let result = allocate_feature_number(temp.path()).unwrap();
        assert_eq!(result, "001");
    }

    #[test]
    fn test_allocate_with_existing_directories() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir_all(specs_dir.join("001-first")).unwrap();
        fs::create_dir_all(specs_dir.join("005-fifth")).unwrap();
        fs::create_dir_all(specs_dir.join("003-third")).unwrap();

        let result = allocate_feature_number(temp.path()).unwrap();
        assert_eq!(result, "006");
    }

    #[test]
    fn test_allocate_with_catalog() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir_all(&specs_dir).unwrap();

        let catalog = r#"{
            "project": "test",
            "description": "test project",
            "features": [
                {"id": "001", "name": "first", "description": "First", "status": "complete", "phase": 1},
                {"id": "010", "name": "tenth", "description": "Tenth", "status": "draft", "phase": 1}
            ]
        }"#;
        fs::write(specs_dir.join("features.json"), catalog).unwrap();

        let result = allocate_feature_number(temp.path()).unwrap();
        assert_eq!(result, "011");
    }

    #[test]
    fn test_allocate_catalog_and_directory_combined() {
        let temp = TempDir::new().unwrap();
        let specs_dir = temp.path().join("specs");
        fs::create_dir_all(specs_dir.join("015-orphan")).unwrap();

        let catalog = r#"{
            "project": "test",
            "description": "test project",
            "features": [
                {"id": "010", "name": "tenth", "description": "Tenth", "status": "draft", "phase": 1}
            ]
        }"#;
        fs::write(specs_dir.join("features.json"), catalog).unwrap();

        // Should use max of catalog (10) and directory (15) -> 16
        let result = allocate_feature_number(temp.path()).unwrap();
        assert_eq!(result, "016");
    }
}
