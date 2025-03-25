use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

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

struct BrainServer {
    tools: Vec<Tool>,
}

struct Tool {
    name: String,
    description: String,
    input_schema: Value,
}

impl BrainServer {
    fn new() -> Self {
        let mut server = Self { tools: Vec::new() };
        server.register_tools();
        server
    }

    fn register_tools(&mut self) {
        self.tools.push(Tool {
            name: "query".to_string(),
            description: "Process a text query".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "The query text"
                    }
                },
                "required": ["text"]
            }),
        });
    }

    fn handle_request(&self, request: McpRequest) -> Result<McpResponse, McpError> {
        match request.method.as_str() {
            "list_tools" => Ok(McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: json!({
                    "tools": self.tools.iter().map(|tool| json!({
                        "name": tool.name,
                        "description": tool.description,
                        "input_schema": tool.input_schema,
                    })).collect::<Vec<_>>()
                }),
            }),
            "call_tool" => {
                if let Some(tool_name) = request.params.get("name") {
                    if tool_name == "query" {
                        if let Some(args) = request.params.get("arguments") {
                            if let Some(text) = args.get("text") {
                                return Ok(McpResponse {
                                    jsonrpc: "2.0".to_string(),
                                    id: request.id,
                                    result: json!({
                                        "content": [{
                                            "type": "text",
                                            "text": format!("Hello! Your query was: {}", text)
                                        }]
                                    }),
                                });
                            }
                        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_list_tools_format() {
        let server = BrainServer::new();
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
                assert!(!tools.is_empty());
                let tool = &tools[0];
                assert!(tool.get("name").is_some());
                assert!(tool.get("description").is_some());
                assert!(tool.get("input_schema").is_some());
            } else {
                panic!("tools should be an array");
            }
        } else {
            panic!("result should be an object");
        }
    }

    #[test]
    fn test_call_tool_query() {
        let server = BrainServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(2),
            method: "call_tool".to_string(),
            params: json!({
                "name": "query",
                "arguments": {
                    "text": "test query"
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
                assert!(item["text"].as_str().unwrap().contains("test query"));
            } else {
                panic!("content should be an array");
            }
        } else {
            panic!("result should be an object");
        }
    }

    #[test]
    fn test_method_not_found() {
        let server = BrainServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(3),
            method: "invalid_method".to_string(),
            params: json!({}),
        };

        let error = server.handle_request(request).unwrap_err();
        
        // Verify JSON-RPC 2.0 error format
        assert_eq!(error.jsonrpc, "2.0");
        assert_eq!(error.id, json!(3));
        assert_eq!(error.error.code, -32601);
        assert_eq!(error.error.message, "Method not found");
    }

    #[test]
    fn test_invalid_params() {
        let server = BrainServer::new();
        let request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: json!(4),
            method: "call_tool".to_string(),
            params: json!({
                "name": "query",
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
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let server = BrainServer::new();
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
