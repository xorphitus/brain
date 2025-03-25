mod config;
mod search;
mod content;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;

use config::Config;

// MCP protocol structures
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct McpRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    params: Value,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    id: Value,
    result: Value,
}

#[derive(Debug, Serialize)]
struct McpError {
    jsonrpc: String,
    id: Value,
    error: McpErrorDetail,
}

#[derive(Debug, Serialize)]
struct McpErrorDetail {
    code: i32,
    message: String,
}

// Server implementation
struct BrainServer {
    tools: Vec<Tool>,
    config: Arc<Config>,
}

struct Tool {
    name: String,
    description: String,
    input_schema: Value,
}

impl BrainServer {
    fn new() -> Result<Self> {
        let config = load_config()?;
        let config = Arc::new(config);
        
        let mut server = Self { 
            tools: Vec::new(),
            config,
        };
        server.register_tools();
        Ok(server)
    }

    fn register_tools(&mut self) {
        // search_files tool
        self.tools.push(Tool {
            name: "search_files".to_string(),
            description: "Search for relevant files based on keywords".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "keywords": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Keywords to search for in files"
                    }
                },
                "required": ["keywords"]
            }),
        });

        // get_contents tool
        self.tools.push(Tool {
            name: "get_contents".to_string(),
            description: "Get contents of specified files".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "file_paths": {
                        "type": "array",
                        "items": {
                            "type": "string"
                        },
                        "description": "Paths of files to retrieve contents from"
                    }
                },
                "required": ["file_paths"]
            }),
        });
    }

    fn handle_request(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        match request.method.as_str() {
            "list_tools" => Ok(McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: json!({
                    "tools": self.tools.iter().map(|tool| json!({
                        "name": tool.name,
                        "description": tool.description,
                        "input_schema": tool.input_schema,
                    })).collect::<Vec<_>>()
                }),
            }),
            "call_tool" => {
                if let Some(tool_name) = request.params.get("name").and_then(|v| v.as_str()) {
                    match tool_name {
                        "search_files" => {
                            if let Some(args) = request.params.get("arguments") {
                                if let Some(keywords) = args.get("keywords").and_then(|k| k.as_array()) {
                                    let keywords: Vec<String> = keywords
                                        .iter()
                                        .filter_map(|k| k.as_str().map(|s| s.to_string()))
                                        .collect();
                                    
                                    match search::search_files(&self.config, &keywords) {
                                        Ok(results) => {
                                            return Ok(McpResponse {
                                                jsonrpc: "2.0".to_string(),
                                                id: request.id,
                                                result: json!({
                                                    "content": [{
                                                        "type": "text",
                                                        "text": serde_json::to_string_pretty(&results).unwrap()
                                                    }]
                                                }),
                                            });
                                        }
                                        Err(e) => {
                                            return Err(McpError {
                                                jsonrpc: "2.0".to_string(),
                                                id: request.id,
                                                error: McpErrorDetail {
                                                    code: -32603,
                                                    message: format!("Internal error: {}", e),
                                                },
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        "get_contents" => {
                            if let Some(args) = request.params.get("arguments") {
                                if let Some(file_paths) = args.get("file_paths").and_then(|p| p.as_array()) {
                                    let file_paths: Vec<String> = file_paths
                                        .iter()
                                        .filter_map(|p| p.as_str().map(|s| s.to_string()))
                                        .collect();
                                    
                                    match content::get_contents(&file_paths) {
                                        Ok(contents) => {
                                            return Ok(McpResponse {
                                                jsonrpc: "2.0".to_string(),
                                                id: request.id,
                                                result: json!({
                                                    "content": [{
                                                        "type": "text",
                                                        "text": contents
                                                    }]
                                                }),
                                            });
                                        }
                                        Err(e) => {
                                            return Err(McpError {
                                                jsonrpc: "2.0".to_string(),
                                                id: request.id,
                                                error: McpErrorDetail {
                                                    code: -32603,
                                                    message: format!("Internal error: {}", e),
                                                },
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                
                Err(McpError {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    error: McpErrorDetail {
                        code: -32602,
                        message: "Invalid parameters".to_string(),
                    },
                })
            }
            _ => Err(McpError {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                error: McpErrorDetail {
                    code: -32601,
                    message: "Method not found".to_string(),
                },
            }),
        }
    }
}

fn load_config() -> Result<Config> {
    config::load_config()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
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
        
        // Create a test org file
        let org_dir = temp_dir.path().join("notes");
        fs::create_dir_all(&org_dir).unwrap();
        
        let test_file = org_dir.join("test.org");
        let mut file = File::create(&test_file).unwrap();
        writeln!(file, "* Test Heading").unwrap();
        writeln!(file, "This is a test file with some keywords.").unwrap();
        writeln!(file, "It contains information about testing and examples.").unwrap();
        
        (temp_dir, config_path)
    }

    #[test]
    fn test_load_config() {
        // This test now uses the config module's load_config function
        let (temp_dir, _) = create_test_config();
        
        // Temporarily override HOME to use our test config
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());
        
        let config = config::load_config();
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
    fn test_list_tools_format() {
        let (temp_dir, _) = create_test_config();
        
        // Temporarily override HOME to use our test config
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());
        
        let server = BrainServer::new().unwrap();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(1),
            method: "list_tools".to_string(),
            params: json!({}),
        };

        let response = server.handle_request(request).unwrap();
        
        // Verify JSON-RPC 2.0 format
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, json!(1));
        
        // Verify tools list format
        if let Value::Object(result) = response.result {
            assert!(result.contains_key("tools"));
            if let Value::Array(tools) = &result["tools"] {
                assert_eq!(tools.len(), 2); // Should have search_files and get_contents
                
                // Check search_files tool
                let search_tool = tools.iter().find(|t| t["name"] == "search_files").unwrap();
                assert!(search_tool.get("description").is_some());
                assert!(search_tool.get("input_schema").is_some());
                
                // Check get_contents tool
                let contents_tool = tools.iter().find(|t| t["name"] == "get_contents").unwrap();
                assert!(contents_tool.get("description").is_some());
                assert!(contents_tool.get("input_schema").is_some());
            } else {
                panic!("tools should be an array");
            }
        } else {
            panic!("result should be an object");
        }
        
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
    fn test_search_files() {
        let (temp_dir, _) = create_test_config();
        
        // Temporarily override HOME to use our test config
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());
        
        let server = BrainServer::new().unwrap();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(2),
            method: "call_tool".to_string(),
            params: json!({
                "name": "search_files",
                "arguments": {
                    "keywords": ["test", "keywords"]
                }
            }),
        };

        let response = server.handle_request(request).unwrap();
        
        // Verify JSON-RPC 2.0 format
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, json!(2));
        
        // Verify response format
        if let Value::Object(result) = response.result {
            assert!(result.contains_key("content"));
            if let Value::Array(content) = &result["content"] {
                assert_eq!(content.len(), 1);
                let item = &content[0];
                assert_eq!(item["type"], "text");
                
                // The text should contain our test file
                let text = item["text"].as_str().unwrap();
                assert!(text.contains("test.org"));
            } else {
                panic!("content should be an array");
            }
        } else {
            panic!("result should be an object");
        }
        
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
    fn test_get_contents() {
        let (temp_dir, _) = create_test_config();
        
        // Temporarily override HOME to use our test config
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());
        
        // Get the path to our test file
        let test_file_path = temp_dir.path().join("notes").join("test.org").to_string_lossy().to_string();
        
        let server = BrainServer::new().unwrap();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(3),
            method: "call_tool".to_string(),
            params: json!({
                "name": "get_contents",
                "arguments": {
                    "file_paths": [test_file_path]
                }
            }),
        };

        let response = server.handle_request(request).unwrap();
        
        // Verify JSON-RPC 2.0 format
        assert_eq!(response.jsonrpc, "2.0");
        assert_eq!(response.id, json!(3));
        
        // Verify response format
        if let Value::Object(result) = response.result {
            assert!(result.contains_key("content"));
            if let Value::Array(content) = &result["content"] {
                assert_eq!(content.len(), 1);
                let item = &content[0];
                assert_eq!(item["type"], "text");
                
                // The text should contain our test file content
                let text = item["text"].as_str().unwrap();
                assert!(text.contains("Test Heading"));
                assert!(text.contains("keywords"));
            } else {
                panic!("content should be an array");
            }
        } else {
            panic!("result should be an object");
        }
        
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
    fn test_method_not_found() {
        let (temp_dir, _) = create_test_config();
        
        // Temporarily override HOME to use our test config
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());
        
        let server = BrainServer::new().unwrap();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(4),
            method: "invalid_method".to_string(),
            params: json!({}),
        };

        let error = server.handle_request(request).unwrap_err();
        
        // Verify JSON-RPC 2.0 error format
        assert_eq!(error.jsonrpc, "2.0");
        assert_eq!(error.id, json!(4));
        assert_eq!(error.error.code, -32601);
        assert_eq!(error.error.message, "Method not found");
        
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
    fn test_invalid_params() {
        let (temp_dir, _) = create_test_config();
        
        // Temporarily override HOME to use our test config
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", temp_dir.path());
        
        let server = BrainServer::new().unwrap();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(4),
            method: "call_tool".to_string(),
            params: json!({
                "name": "search_files",
                "arguments": {
                    "invalid_param": "test"
                }
            }),
        };

        let error = server.handle_request(request).unwrap_err();
        
        // Verify JSON-RPC 2.0 error format
        assert_eq!(error.jsonrpc, "2.0");
        assert_eq!(error.id, json!(4));
        assert_eq!(error.error.code, -32602);
        assert_eq!(error.error.message, "Invalid parameters");
        
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

#[tokio::main]
async fn main() -> Result<()> {
    let server = match BrainServer::new() {
        Ok(server) => server,
        Err(e) => {
            eprintln!("Failed to initialize server: {}", e);
            return Err(e);
        }
    };
    
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout();

    eprintln!("MCP server started. Waiting for input...");

    for line in stdin.lines() {
        let line = line?;
        eprintln!("Received request: {}", line);
        
        let request: McpRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let error = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    }
                });
                writeln!(stdout, "{}", error.to_string())?;
                stdout.flush()?;
                eprintln!("Parse error: {}", e);
                continue;
            }
        };

        let response = match server.handle_request(request) {
            Ok(response) => serde_json::to_string(&response)?,
            Err(error) => serde_json::to_string(&error)?,
        };

        eprintln!("Sending response: {}", response);
        writeln!(stdout, "{}", response)?;
        stdout.flush()?;
    }

    Ok(())
}
