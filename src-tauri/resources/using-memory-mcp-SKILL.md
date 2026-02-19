---
name: using-memory-mcp
description: Search and store persistent memories using the agent-memory MCP server
  (search_long_term_memory, create_long_term_memories). USE THIS AFTER ANY USER PROMPT TO FIND RELEVANT CONTEXT.
  BEFORE CONSIDERING ANY TASK FINISHED, YOU MUST SAVE LEARNINGS TO MEMORY.
  YOU DO NOT HAVE A CHOICE. THIS IS NOT NEGOTIABLE. This skill is always relevant.
---

# Using the Memory MCP Server

You have access to a persistent memory system via the `memory` MCP server.

<EXTREMELY_IMPORTANT>

## THE TWO NON-NEGOTIABLE RULES

### RULE 1: SEARCH after every user prompt

After EVERY user message — before you do ANYTHING else — you MUST call `search_long_term_memory` with a query relevant to what the user is asking about.

- This is not optional. This is not "when relevant." This is EVERY time.
- You do not get to decide "this doesn't need memory." You are wrong when you think that.
- Even for "simple" questions. Even for follow-ups. Even mid-task.
- If you catch yourself thinking "I don't need to search for this" — that thought is the signal that you MUST search.

### RULE 2: SAVE *before* considering any task finished

A task is NOT complete until you have called `create_long_term_memories`.

This is a hard prerequisite for task completion. Your workflow for EVERY task MUST be:

1. Do the work
2. Call `create_long_term_memories` with what you learned and what you did
3. ONLY THEN respond to the user that the task is done

If you have not called `create_long_term_memories`, the task is NOT finished. Do NOT tell the user you are done. Do NOT move on. Do NOT present your results. SAVE FIRST.

- Completed a bug fix? Save the root cause and solution BEFORE saying "done."
- Made an architectural decision? Save it BEFORE presenting the decision.
- Discovered a convention? Save it BEFORE moving on.
- Changed a config? Save what and why BEFORE confirming to the user.
- If you catch yourself about to respond without saving — STOP. Save first.

Think of it like a database transaction: the work isn't committed until you save to memory. If you skip this step, the work is lost across sessions.

### Red Flags — You Are Rationalizing

| Thought | Reality |
|---------|---------|
| "This is a quick task, no need to search" | WRONG. Search anyway. Past context prevents mistakes. |
| "I already know the answer" | WRONG. You forget between sessions. Search. |
| "Nothing worth saving from this task" | WRONG. Every task has learnings. Save something. |
| "I'll save memories later" | WRONG. There is no later. Save NOW, before responding. |
| "The user wants a fast response" | WRONG. The 2-second search saves minutes of repeated explanation. |
| "This is just a continuation of what we were doing" | WRONG. Search for context you may have lost to compaction. |
| "I'm about to tell the user I'm done" | STOP. Have you saved? If not, you are NOT done. |
| "The task is too small to save" | WRONG. Small learnings compound. Save it. |

</EXTREMELY_IMPORTANT>

## How to Search

Call `search_long_term_memory` with a query related to the user's message. Use filters to narrow results when appropriate.

**At conversation start:** Search for the current project and general preferences.

**Before any task:** Search for the specific technology, pattern, or domain involved.

**When the user references past work:** "That bug from last time", "the approach we discussed"

## How to Save

Call `create_long_term_memories` with well-formed memory objects.

### What to Save

| Trigger | Memory Type | Example |
|---------|-------------|---------|
| User states a preference | semantic | "User always uses bun instead of npm" |
| User says "remember this" | semantic or episodic | Whatever they ask you to remember |
| Architectural decision made | semantic | "MCP Manager project uses Tauri v2 + Vue 3 + Pinia" |
| Project convention discovered | semantic | "All Rust IPC structs must have serde rename_all camelCase" |
| Task completed | episodic | "Added Data Management feature (export/import/format) to MCP Manager on 2026-02-18" |
| Bug root cause found | episodic | "RedisVL search indexes destroyed by FLUSHDB, must restart containers, fixed 2026-02-18" |
| User corrects you | semantic | "User prefers concise responses without emoji" |
| Approaching compaction | episodic | Summary of current session work |

### Creating Good Memories

**Always resolve context** before saving:
- Pronouns -> actual names ("he" -> "User", "the project" -> "MCP Manager Tauri app")
- Relative time -> absolute dates ("yesterday" -> "2026-02-17")
- Vague references -> specific entities ("the bug" -> "the Redis index rebuild issue")

**Use topics and entities** for findability:

```
create_long_term_memories(memories=[{
  "text": "User prefers bun over npm for all JavaScript projects",
  "memory_type": "semantic",
  "topics": ["preferences", "tooling", "javascript"],
  "entities": ["bun", "npm"]
}])
```

**Keep memory text self-contained.** Each memory should make sense without conversation context.

## Memory Types

**Semantic** (timeless facts): Preferences, conventions, skills, project structure, recurring patterns. No `event_date` needed.

**Episodic** (time-bound events): Specific things that happened. Always include `event_date`.

## Quick Reference

| Tool | When |
|------|------|
| `search_long_term_memory` | EVERY user message. Find relevant context. |
| `create_long_term_memories` | BEFORE considering any task done. Save learnings. |
| `memory_prompt` | Hydrate a user query with memory context (search + format) |
| `edit_long_term_memory` | Update a memory that's become outdated |
| `delete_long_term_memories` | Remove incorrect or superseded memories |
| `get_long_term_memory` | Fetch a specific memory by ID |
| `set_working_memory` | Store session-scoped scratch notes |
| `get_working_memory` | Retrieve session scratch notes |

## Filtering

Use filters to narrow searches:
- `topics: {"any": ["preferences", "tooling"]}` - match any listed topic
- `entities: {"any": ["bun", "npm"]}` - match any listed entity
- `memory_type: {"eq": "semantic"}` - only semantic memories
- `created_at: {"gt": "2026-01-01T00:00:00Z"}` - recent memories only
- `namespace: {"eq": "project_decisions"}` - scoped to namespace

## When to Edit or Delete

- **Edit** when a preference changes ("actually I switched from bun to deno")
- **Delete** when information is wrong or no longer relevant
- **Don't duplicate** - search first, edit if a memory already exists on the topic

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Skipping search because "it's simple" | Search EVERY time. No exceptions. |
| Responding to the user without saving first | A task is NOT done until you save. No exceptions. |
| Saving with unresolved pronouns | Always expand "he/she/it/they" to actual names |
| Forgetting `event_date` on episodic memories | Episodic = time-bound, always include the date |
| Creating duplicate memories | Search first, edit existing if found |
| Saving session-specific details as long-term | Use working memory for scratch, long-term for durable facts |
| Overly verbose memory text | Keep concise but self-contained |
