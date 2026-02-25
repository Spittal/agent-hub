---
name: using-discovery
description: Find and use MCP tools through the discovery endpoint. When you need
  a tool you don't have, use discover_tools to search, list_servers to browse, and
  call_tool to execute. Use this skill whenever you're looking for capabilities,
  searching for tools, or need to call a tool on a specific server.
---

# Tool Discovery — Find and Use Any MCP Tool

You have access to a **tool discovery** endpoint that lets you search across all connected MCP servers and call any tool by name — without needing a separate MCP connection for each server.

## The Three Tools

### 1. `list_servers` — See what's connected

Call this first to get an overview of all available MCP servers and their tools.

```json
// No arguments needed
{}
```

Returns a list of servers with their IDs, names, and tool names. Use the `server_id` from results when calling `call_tool`.

### 2. `discover_tools` — Search for the right tool

When you need a capability but don't know which tool provides it, search by keyword.

```json
{
  "query": "send message slack"
}
```

Returns matching tools with their **full input schemas** so you can call them immediately. Results include the `server_id` and `tool_name` you need for `call_tool`.

### 3. `call_tool` — Execute a tool on a specific server

Once you know the server and tool name (from `discover_tools` or `list_servers`), call it:

```json
{
  "server_id": "abc-123",
  "tool_name": "send_message",
  "arguments": {
    "channel": "#general",
    "text": "Hello from the agent!"
  }
}
```

## Workflow

**When you need a tool you don't have:**

1. `discover_tools(query="what you need")` — find matching tools across all servers
2. Read the returned input schema to understand required arguments
3. `call_tool(server_id, tool_name, arguments)` — execute it

**When you want to explore what's available:**

1. `list_servers()` — see all connected servers and tool names
2. `discover_tools(query="specific capability")` — get full schemas for tools you're interested in
3. `call_tool(...)` — use what you find

## Examples

**"I need to search Slack messages"**
```
discover_tools(query="search slack messages")
→ Found: slack_search_messages on server "slack-mcp" (id: abc-123)
call_tool(server_id="abc-123", tool_name="slack_search_messages", arguments={"query": "project update"})
```

**"What databases can I query?"**
```
discover_tools(query="database query sql")
→ Found: execute_query on server "postgres-mcp" (id: def-456)
```

**"Create a GitHub issue"**
```
discover_tools(query="github issue create")
→ Found: create_issue on server "github-mcp" (id: ghi-789)
call_tool(server_id="ghi-789", tool_name="create_issue", arguments={"repo": "org/repo", "title": "Bug report", "body": "Details..."})
```

## Tips

- **Search broadly first**, then narrow down. `discover_tools(query="email")` is better than guessing a specific tool name.
- **Check the input schema** returned by `discover_tools` — it tells you exactly what arguments are required vs optional.
- **`list_servers` is cheap** — call it when you want a quick overview without searching for anything specific.
- If `discover_tools` returns no results, try **different keywords** or use `list_servers` to see what's actually connected.
