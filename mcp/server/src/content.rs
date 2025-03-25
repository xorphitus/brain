use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Retrieves the contents of the specified files
pub fn get_contents(file_paths: &[String]) -> Result<String> {
    let mut contents = HashMap::new();

    for path in file_paths {
        let file_path = Path::new(path);
        if file_path.exists() {
            match fs::read_to_string(file_path) {
                Ok(content) => {
                    contents.insert(path.clone(), content);
                }
                Err(e) => {
                    eprintln!("Error reading file {}: {}", path, e);
                    contents.insert(path.clone(), format!("Error reading file: {}", e));
                }
            }
        } else {
            contents.insert(path.clone(), "File not found".to_string());
        }
    }

    Ok(serde_json::to_string_pretty(&contents)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write as IoWrite;
    use tempfile::tempdir;

    #[test]
    fn test_get_contents() {
        // Create a temporary test file
        let temp_dir = tempdir().unwrap();
        let test_file_path = temp_dir.path().join("test_content.txt");
        let test_content = "This is test content.";
        
        {
            let mut file = File::create(&test_file_path).unwrap();
            write!(file, "{}", test_content).unwrap();
        }
        
        // Test with existing file
        let file_paths = vec![test_file_path.to_string_lossy().to_string()];
        let result = get_contents(&file_paths).unwrap();
        
        // Result should be a JSON string containing our test content
        assert!(result.contains("test_content.txt"));
        assert!(result.contains(test_content));
        
        // Test with non-existent file
        let file_paths = vec!["nonexistent_file.txt".to_string()];
        let result = get_contents(&file_paths).unwrap();
        
        // Result should indicate file not found
        assert!(result.contains("nonexistent_file.txt"));
        assert!(result.contains("File not found"));
        
        // Clean up
        drop(temp_dir);
    }
}
