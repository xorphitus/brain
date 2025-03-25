use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::Config;

// Search result structure
#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub path: String,
    pub relevance: f64,
}

/// Searches files in the knowledge base for the given keywords
pub fn search_files(config: &Config, keywords: &[String]) -> Result<Vec<SearchResult>> {
    let root_path = Path::new(&config.knowledge.root_path);
    if !root_path.exists() {
        return Err(anyhow::anyhow!("Knowledge base path does not exist: {}", config.knowledge.root_path));
    }

    // Create regex patterns for each keyword
    let patterns: Vec<Regex> = keywords
        .iter()
        .map(|k| Regex::new(&format!(r"(?i){}", regex::escape(k))))
        .collect::<std::result::Result<Vec<_>, _>>()?;

    // Collect all .org files
    let files: Vec<PathBuf> = WalkDir::new(root_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| ext == "org")
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    // Search files in parallel
    let results: Vec<(PathBuf, f64)> = files
        .par_iter()
        .filter_map(|file_path| {
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    // Calculate relevance score based on keyword matches
                    let mut score = 0.0;
                    for pattern in &patterns {
                        let matches = pattern.find_iter(&content).count();
                        if matches > 0 {
                            score += matches as f64;
                        }
                    }
                    
                    if score > 0.0 {
                        Some((file_path.clone(), score))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            }
        })
        .collect();

    // Sort by relevance (descending) and limit to max_files
    let mut sorted_results = results;
    sorted_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    sorted_results.truncate(config.knowledge.max_files);

    // Convert to SearchResult format
    let search_results = sorted_results
        .into_iter()
        .map(|(path, relevance)| {
            SearchResult {
                path: path.to_string_lossy().to_string(),
                relevance,
            }
        })
        .collect();

    Ok(search_results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write as IoWrite;
    use tempfile::tempdir;
    use crate::config::{Config, OllamaConfig, KnowledgeConfig, McpConfig};

    fn create_test_environment() -> (tempfile::TempDir, Config) {
        let temp_dir = tempdir().unwrap();
        
        // Create a test org file
        let org_dir = temp_dir.path().join("notes");
        fs::create_dir_all(&org_dir).unwrap();
        
        let test_file = org_dir.join("test.org");
        let mut file = File::create(&test_file).unwrap();
        writeln!(file, "* Test Heading").unwrap();
        writeln!(file, "This is a test file with some keywords.").unwrap();
        writeln!(file, "It contains information about testing and examples.").unwrap();
        
        // Create test config
        let config = Config {
            ollama: OllamaConfig {
                endpoint: "http://localhost:11434".to_string(),
                model: "mistral".to_string(),
                max_context_length: 4096,
            },
            knowledge: KnowledgeConfig {
                root_path: temp_dir.path().to_string_lossy().to_string(),
                max_files: 5,
            },
            mcp: McpConfig {
                server_name: "brain-files".to_string(),
            },
        };
        
        (temp_dir, config)
    }

    #[test]
    fn test_search_files() {
        let (temp_dir, config) = create_test_environment();
        
        // Test with keywords that should match
        let keywords = vec!["test".to_string(), "keywords".to_string()];
        let results = search_files(&config, &keywords).unwrap();
        
        // Should find our test file
        assert!(!results.is_empty());
        assert!(results[0].path.contains("test.org"));
        assert!(results[0].relevance > 0.0);
        
        // Test with keywords that shouldn't match
        let keywords = vec!["nonexistent".to_string(), "notfound".to_string()];
        let results = search_files(&config, &keywords).unwrap();
        
        // Should not find any files
        assert!(results.is_empty());
        
        // Clean up
        drop(temp_dir);
    }
}
