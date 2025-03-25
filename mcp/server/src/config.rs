use anyhow::{Context, Result};
use dirs::home_dir;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

// Configuration structures
#[derive(Debug, Deserialize)]
pub struct Config {
    pub knowledge: KnowledgeConfig,
    pub mcp: McpConfig,
}

#[derive(Debug, Deserialize)]
pub struct KnowledgeConfig {
    pub root_path: String,
    pub max_files: usize,
}

#[derive(Debug, Deserialize)]
pub struct McpConfig {
    pub server_name: String,
}

/// Loads the configuration from the default path (~/.config/brain/config.toml)
pub fn load_config() -> Result<Config> {
    let config_path = get_default_config_path()?;
    load_config_from_path(&config_path)
}

/// Loads the configuration from a specific path
pub fn load_config_from_path(config_path: &Path) -> Result<Config> {
    if !config_path.exists() {
        return Err(anyhow::anyhow!("Config file not found: {}", config_path.display()));
    }

    let config_str = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let config: Config = toml::from_str(&config_str)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

    Ok(config)
}

/// Returns the default configuration file path
pub fn get_default_config_path() -> Result<PathBuf> {
    let config_path = home_dir()
        .context("Could not determine home directory")?
        .join(".config")
        .join("brain")
        .join("config.toml");
    
    Ok(config_path)
}

/// Creates a test configuration for testing purposes
#[cfg(test)]
pub fn create_test_config_for_tests(root_path: &Path) -> Config {
    Config {
        knowledge: KnowledgeConfig {
            root_path: root_path.to_string_lossy().to_string(),
            max_files: 5,
        },
        mcp: McpConfig {
            server_name: "brain-files".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write as IoWrite;
    use tempfile::tempdir;

    fn create_test_config() -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().join(".config").join("brain");
        fs::create_dir_all(&config_dir).unwrap();
        
        let config_path = config_dir.join("config.toml");
        let mut file = File::create(&config_path).unwrap();
        
        writeln!(file, "[knowledge]").unwrap();
        writeln!(file, "root_path = \"{}\"", temp_dir.path().display()).unwrap();
        writeln!(file, "max_files = 5").unwrap();
        writeln!(file, "").unwrap();
        writeln!(file, "[mcp]").unwrap();
        writeln!(file, "server_name = \"brain-files\"").unwrap();
        
        (temp_dir, config_path)
    }

    #[test]
    fn test_load_config_from_path() {
        let (temp_dir, config_path) = create_test_config();
        
        let config = load_config_from_path(&config_path);
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.knowledge.max_files, 5);
        assert_eq!(config.mcp.server_name, "brain-files");
        
        // Clean up
        drop(temp_dir);
    }

    #[test]
    fn test_load_config() {
        let (temp_dir, _) = create_test_config();
        
        // Temporarily override HOME to use our test config
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());
        
        let config = load_config();
        assert!(config.is_ok());
        
        let config = config.unwrap();
        assert_eq!(config.knowledge.max_files, 5);
        assert_eq!(config.mcp.server_name, "brain-files");
        
        // Restore original HOME
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
        
        // Clean up
        drop(temp_dir);
    }

    #[test]
    fn test_get_default_config_path() {
        let (temp_dir, _) = create_test_config();
        
        // Temporarily override HOME to use our test config
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());
        
        let config_path = get_default_config_path().unwrap();
        assert!(config_path.to_string_lossy().contains(".config/brain/config.toml"));
        
        // Restore original HOME
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
        
        // Clean up
        drop(temp_dir);
    }
}
