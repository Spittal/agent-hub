# Agent Hub

Manage MCP servers, skills, and plugins across all your AI tools from one place.

## Why we built this

We use agentic development tools heavily — Claude Code, Cursor, Windsurf, and others. As we added more MCP servers, skills, and plugins, the friction added up. Every tool has its own config format and file location. Adding a server means editing JSON in multiple places. Setting up persistent memory means wrangling Docker, Redis, embedding models, and environment variables. Skills are markdown files you copy into per-tool directories. Keeping everything in sync is tedious and error-prone.

Agent Hub is the tool we built to make our own workflow better. It sits between your MCP servers and your AI tools, handling config management, process lifecycle, and observability so you can focus on actual work. We think it'll improve your experience using agents for development too.

## How it works

Agent Hub runs a local HTTP proxy. Each MCP server you connect gets its own endpoint at `http://localhost:{port}/mcp/{server_id}`. When you connect or disconnect a server, Agent Hub rewrites the config files for all your enabled AI tools automatically.

When you quit the app, it restores the original configs so your tools still work standalone — no lock-in.

Supported tools: Claude Code, Cursor, Claude Desktop, Windsurf, Zed, OpenCode, Codex.

## Features

### MCP server management

Add servers via stdio or HTTP transport. Toggle them on and off. OAuth 2.1 is handled for you — token refresh, PKCE flow, the whole lifecycle. Servers that were connected when you last quit auto-reconnect on launch.

When you enable an AI tool integration, Agent Hub imports any servers already in that tool's config, so you don't lose existing setups.

### Auto-config sync

One server connected in Agent Hub shows up in every enabled AI tool. One server disconnected disappears from all of them. No more editing four different JSON files (and one TOML file) every time something changes.

### Discovery mode

When you have a lot of MCP servers, every tool from every server ends up in your agent's context. That's a lot of token overhead, and it makes tool selection harder for the model. Discovery mode replaces all those individual tool definitions with three meta-tools: `discover_tools` (search by keyword), `call_tool` (invoke on a specific server), and `list_servers` (see what's available). Tools are loaded on-demand instead of all at once, and a managed skill is auto-installed to teach your agent how to use them.

### Agent memory

Setting up [agent-memory-server](https://github.com/anthropics/agent-memory-server) manually means running Docker containers for Redis and the memory API, configuring embedding providers and environment variables, writing a skill file so your agent knows how to use it, and adding the server to each tool's config. Agent Hub reduces that to a toggle. Pick your embedding provider (OpenAI or Ollama), flip the switch, and it handles the container orchestration, health checks, MCP server registration, and skill installation. Memory is then shared across all your connected AI tools.

### Marketplace

Browse ~2,300 MCP servers from [MCPAnvil](https://mcpanvil.com) with GitHub star counts. One-click install — Agent Hub detects required environment variables (API keys, tokens) and prompts you for them before creating the server.

### Skills

Browse and install skills from [skills.sh](https://skills.sh). Skills are markdown instruction files that teach your agent domain-specific behaviors. Agent Hub syncs installed skills to all your enabled tools — no manual file copying.

### Plugins

Discover and manage Claude CLI plugins from a GUI. See what each plugin includes (skills, agents, commands, hooks, MCP servers) and install, uninstall, or toggle them without touching the terminal.

### Observability

Per-server tool usage stats: total calls, error rates, average latency. Per-tool breakdowns. Recent call history showing which AI tool made each call. Real-time log streaming from connected servers.

## Development

```bash
pnpm install
pnpm tauri dev        # full app with Rust backend
pnpm tauri build      # production build (.dmg/.app)
```

## Tech stack

Tauri v2 (Rust) + Vue 3 + TypeScript + Tailwind CSS v4. Axum for the proxy server. Pinia for frontend state.
