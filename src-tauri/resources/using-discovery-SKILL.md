---
name: using-discovery
description: You have MCP tools from a server called "mcp-manager" (or
  "user-mcp-manager") that give you access to ALL other connected MCP servers
  through three tools — discover_tools, call_tool, and list_servers. USE THESE
  TOOLS whenever you need a capability you don't currently have (Slack, GitHub,
  databases, etc.), want to search for available tools, or need to call a tool on
  a remote server. These tools are already in your tool list — just call them.
---

# MCP Manager — Tool Discovery

You already have three MCP tools from a server called **mcp-manager** (it may also appear as **user-mcp-manager** depending on your client). These tools let you search for and call tools on ANY connected MCP server — Slack, GitHub, databases, file systems, anything the user has set up.

**You do not need separate MCP connections for each server.** The mcp-manager app acts as a proxy: your single connection to it gives you access to every server it manages.

## Your Three Tools

You have these tools right now in your MCP tool list. Call them directly.

### `discover_tools` — Search for a tool by keyword

This is your **go-to first step** when you need a capability. It searches across ALL connected servers and returns matching tools with their full input schemas.

```json
{
  "query": "slack message send"
}
```

Returns: tool name, server_id, server_name, description, and **full inputSchema** — everything you need to call it immediately via `call_tool`.

Search tips:
- Use broad terms: `"slack"` finds all Slack tools, `"database"` finds all DB tools
- Multiple words are AND-matched: `"slack message"` finds tools matching both terms
- If no results, try fewer or different keywords

### `call_tool` — Execute a tool on a specific server

Once you have a `server_id` and `tool_name` from `discover_tools`, call the tool:

```json
{
  "server_id": "the-server-id-from-discover",
  "tool_name": "send_message",
  "arguments": {
    "channel": "#general",
    "text": "Hello!"
  }
}
```

The `arguments` object must match the tool's `inputSchema` from the discover results.

### `list_servers` — See all connected servers

Call with no arguments to get an overview of every connected server and its tools:

```json
{}
```

Returns: server IDs, names, tool counts, and tool name lists. Useful when you want to browse rather than search.

## When to Use This

**Any time you think "I don't have a tool for X" — you probably do.** Call `discover_tools` before giving up or telling the user you can't do something.

Common situations:
- User asks you to send a Slack message, create a GitHub issue, query a database, etc.
- You need to interact with a service but don't see a direct tool for it
- You want to know what tools and servers are available

## Step-by-Step Workflow

1. **Call `discover_tools`** with keywords describing what you need
2. **Read the results** — pick the right tool, note its `server_id` and `tool_name`
3. **Read the `inputSchema`** — understand required vs optional arguments
4. **Call `call_tool`** with the server_id, tool_name, and arguments
5. **Use the result** — tool output is returned directly to you

## Example: Sending a Slack Message

```
Step 1: discover_tools(query="slack send message")
→ Found: send_message on server "slack-mcp" (server_id: "abc-123")
  inputSchema: { channel: string (required), text: string (required) }

Step 2: call_tool(server_id="abc-123", tool_name="send_message", arguments={"channel": "#general", "text": "Hello from the agent!"})
→ Message sent successfully
```

## Example: Searching for Database Tools

```
Step 1: discover_tools(query="database query")
→ Found: execute_query on server "postgres-mcp" (server_id: "def-456")
  inputSchema: { sql: string (required), params: array (optional) }

Step 2: call_tool(server_id="def-456", tool_name="execute_query", arguments={"sql": "SELECT * FROM users LIMIT 5"})
→ [query results]
```

## Troubleshooting

- **Can't find the tools?** Look for a server named "mcp-manager" or "user-mcp-manager" in your MCP connections. The three tools (`discover_tools`, `call_tool`, `list_servers`) come from that server.
- **`discover_tools` returns nothing?** Try broader keywords, or call `list_servers` to see what's actually connected. The user may not have the server you're looking for.
- **`call_tool` fails with "not connected"?** The target server may have disconnected. Tell the user the server needs to be reconnected in MCP Manager.
