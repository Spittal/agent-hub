---
name: using-discovery
description: You have MCP tools from a server called "mcp-manager" (or
  "user-mcp-manager") that give you access to additional MCP servers
  through three tools — discover_tools, call_tool, and list_servers. USE THESE
  TOOLS whenever you need a capability you don't currently have, want to search
  for available tools, or need to call a tool on a remote server. These tools are
  already in your tool list — just call them.
---

# MCP Manager — Tool Discovery

You have three MCP tools from a server called **mcp-manager** (it may also appear as **user-mcp-manager**). These tools let you search for and call tools on connected MCP servers managed by MCP Manager.

**Note:** Discovery only covers servers managed by MCP Manager. Tools you already have through direct MCP connections (e.g., plugins, native integrations) won't appear in discovery results — you already have those and should call them directly.

## Your Three Tools

### `discover_tools` — Search for a tool by keyword

Your **go-to first step** when you need a capability. Searches across all connected servers and returns matching tools with full input schemas.

Search tips:
- Use broad terms: `"slack"` finds all Slack tools, `"database"` finds all DB tools
- Multiple words are AND-matched: `"slack message"` finds tools matching both terms
- If no results, try fewer or different keywords

### `call_tool` — Execute a tool on a specific server

Once you have a `server_id` and `tool_name` from `discover_tools`, call the tool with the appropriate `arguments` matching the tool's `inputSchema`.

### `list_servers` — See all connected servers

Call with no arguments to get an overview of every connected server and its tools. Useful when you want to browse rather than search.

## When to Use This

**Any time you think "I don't have a tool for X" — you probably do.** Call `discover_tools` before telling the user you can't do something.

## Workflow

1. **Discover**: Call `discover_tools` with keywords — note the `server_id`, `tool_name`, and `inputSchema` from the results
2. **Call**: Call `call_tool` with the server_id, tool_name, and arguments matching the schema

## Troubleshooting

- **`discover_tools` returns nothing?** Try broader keywords, or call `list_servers` to see what's connected. The user may not have that server set up.
- **`call_tool` fails with "not connected"?** The target server may have disconnected. Tell the user to reconnect it in MCP Manager.
- **`call_tool` fails with "managed externally"?** That server's tools are available through your direct MCP connection — call them directly instead of via discovery.
