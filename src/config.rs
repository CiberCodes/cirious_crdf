//! Manages the configuration for the `cirious_crd` library, allowing for
//! customization of parsing and processing behavior.
use crate::CrdError;
use serde::{Deserialize, Serialize};
use std::fs;

/// Represents the configuration for the CRDF library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The scale to apply to vertices when loading.
    pub vertex_scale: f32,
    /// Whether to triangulate faces during parsing.
    pub triangulate: bool,
}

impl Default for Config {
    /// Creates a default configuration with a vertex scale of `1.0` and
    /// triangulation disabled.
    fn default() -> Self {
        Self {
            vertex_scale: 1.0,
            triangulate: false,
        }
    }
}

/// Creates a new configuration file with default values.
///
/// # Arguments
///
/// * `path` - The path to the new configuration file.
pub fn create_config_file(path: &str) -> Result<(), CrdError> {
    let config = Config::default();
    let content = serde_json::to_string_pretty(&config)?;
    fs::write(path, content)?;
    Ok(())
}

/// Loads the configuration from a file.
///
/// # Arguments
///
/// * `path` - The path to the configuration file.
pub fn load_config(path: &str) -> Result<Config, CrdError> {
    let content = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_create_and_load_config() {
        let path = "test_config.json";

        // Create a config file with default values
        create_config_file(path).unwrap();

        // Load the config file and verify its contents
        let loaded_config = load_config(path).unwrap();
        assert_eq!(loaded_config, Config::default());

        // Clean up the test file
        fs::remove_file(path).unwrap();
    }
}
