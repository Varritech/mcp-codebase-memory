// Minimal MCP Server Skeleton for Codebase Memory
// Build with: cargo build --release
// Run with: ./target/release/mcp-server --index ./my-project

use std::io::{self, BufRead, Write};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<u64>,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<u64>,
    result: Option<serde_json::Value>,
    error: Option<Error>,
}

#[derive(Debug, Serialize)]
struct Error {
    code: i32,
    message: String,
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };
        
        let response = handle_request(request);
        
        let output = serde_json::to_string(&response).unwrap();
        writeln!(stdout, "{}", output).unwrap();
        stdout.flush().unwrap();
    }
}

fn handle_request(request: JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "codebase-memory-mcp",
                    "version": "0.1.0"
                }
            })),
            error: None,
        },
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(serde_json::json!({
                "tools": [
                    {
                        "name": "search_symbols",
                        "description": "Search for symbols (functions, classes, modules) in the codebase",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": {"type": "string", "description": "Search query"},
                                "kind": {"type": "string", "description": "Symbol kind filter (function, class, module)"}
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "find_references",
                        "description": "Find all references to a symbol",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "symbol_name": {"type": "string", "description": "Symbol name to find references for"}
                            },
                            "required": ["symbol_name"]
                        }
                    }
                ]
            })),
            error: None,
        },
        "tools/call" => {
            // TODO: Implement actual tool logic here
            // For now, return placeholder
            JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::json!({
                    "content": [
                        {
                            "type": "text",
                            "text": "Tool call received. Implement search logic here."
                        }
                    ]
                })),
                error: None,
            }
        },
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(Error {
                code: -32601,
                message: "Method not found".to_string(),
            }),
        },
    }
}
