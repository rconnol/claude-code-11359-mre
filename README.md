# Claude Code MCP Nested Object Parameter Bug - MRE

This repository contains a minimal reproducible example (MRE) for a bug in **Claude Code** where nested object parameters in MCP tools are incorrectly stringified instead of being passed as proper JSON objects.

**Note:** This bug is specific to Claude Code. Claude Desktop does not have this issue.

## The Bug

When Claude Code calls an MCP tool that has a parameter with a nested struct/object type, it incorrectly passes the nested object as a JSON string instead of as a proper object. This causes deserialization errors on the server side.

### Expected Behavior

The `echo` tool expects parameters like:
```json
{
  "message": "hello",
  "nested_item": {
    "message": "world"
  }
}
```

### Actual Behavior

Claude Code sends:
```json
{
  "message": "hello",
  "nested_item": "{\"message\": \"world\"}"
}
```

Note that `nested_item` is a string containing JSON, not an actual object.

## Prerequisites

- Rust toolchain (install from https://rustup.rs/)
- Claude Code CLI (not Claude Desktop - the bug is specific to Claude Code)

## Setup Instructions

### 1. Clone and Build the MCP Server

```bash
# Clone or navigate to this repository
cd claude-code-11359-mre

# Build the server
cargo build

# Run the server
cargo run
```

The server will start on `http://127.0.0.1:4000`.

### 2. Configure Claude Code

Use the Claude CLI to add the MCP server:

```bash
claude mcp add --transport sse mre http://localhost:4000/mcp/sse
```

This will automatically configure the MCP server in your Claude Code settings.

## Reproducing the Bug

1. Open Claude Code
2. Send this prompt:

```
Can you echo hello - with a nested message of world using the echo mcp server
```

### Expected Result

The tool should successfully combine the messages and return: `hello world`

### Actual Result

You'll see an error:

```
Error: MCP error -32602: failed to deserialize parameters: invalid type: string "{\"message\": \"world\"}", expected struct NestedItem
```

This occurs because Claude Code is passing the `nested_item` parameter as a JSON string instead of a proper object.

## Technical Details

### Server Implementation

The server is a simple Rust MCP server using the `rmcp` crate. It defines:

- `EchoInput` struct with a `message: String` field and a `nested_item: NestedItem` field
- `NestedItem` struct with a `message: String` field
- An `echo` tool that combines both messages

See `src/main.rs` for the full implementation.

### Root Cause

The issue is in how Claude Code serializes nested object parameters when calling MCP tools. Instead of keeping nested objects as objects in the JSON-RPC call, it stringifies them, causing deserialization failures on the server side.

### Output of List Tools (w/ JSONRPC Schema)

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "echo",
        "description": "Takes the input and combines it into a single output message",
        "inputSchema": {
          "$schema": "http://json-schema.org/draft-07/schema#",
          "definitions": {
            "NestedItem": {
              "properties": {
                "message": {
                  "type": "string"
                }
              },
              "required": [
                "message"
              ],
              "type": "object"
            }
          },
          "properties": {
            "message": {
              "type": "string"
            },
            "nested_item": {
              "$ref": "#/definitions/NestedItem"
            }
          },
          "required": [
            "message",
            "nested_item"
          ],
          "title": "EchoInput",
          "type": "object"
        },
        "outputSchema": {
          "$schema": "http://json-schema.org/draft-07/schema#",
          "properties": {
            "message": {
              "type": "string"
            }
          },
          "required": [
            "message"
          ],
          "title": "EchoOutput",
          "type": "object"
        }
      }
    ]
  }
}
```
