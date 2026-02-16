import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { McpTool } from '@/types/mcp';

export const useToolsStore = defineStore('tools', () => {
  const tools = ref<McpTool[]>([]);
  const searchQuery = ref('');

  const filteredTools = computed(() => {
    if (!searchQuery.value) return tools.value;
    const q = searchQuery.value.toLowerCase();
    return tools.value.filter(
      t =>
        t.name.toLowerCase().includes(q) ||
        t.description?.toLowerCase().includes(q) ||
        t.serverName.toLowerCase().includes(q)
    );
  });

  function setTools(serverId: string, newTools: McpTool[]) {
    // Remove old tools for this server, add new ones
    tools.value = [
      ...tools.value.filter(t => t.serverId !== serverId),
      ...newTools,
    ];
  }

  function clearToolsForServer(serverId: string) {
    tools.value = tools.value.filter(t => t.serverId !== serverId);
  }

  async function fetchTools(serverId: string) {
    try {
      const serverTools = await invoke<McpTool[]>('list_tools', { id: serverId });
      setTools(serverId, serverTools);
    } catch {
      // Server not connected or not found — leave store as-is
    }
  }

  return { tools, searchQuery, filteredTools, setTools, clearToolsForServer, fetchTools };
});
