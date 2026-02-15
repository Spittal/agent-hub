<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useServersStore } from '@/stores/servers';
import type { ProxyStatus } from '@/types/proxy';

const store = useServersStore();

const status = ref<ProxyStatus | null>(null);
const error = ref<string | null>(null);
const copied = ref(false);

async function fetchStatus() {
  try {
    status.value = await invoke<ProxyStatus>('get_proxy_status');
    error.value = null;
  } catch (e) {
    error.value = String(e);
  }
}

function manualSnippet(): string {
  const port = status.value?.port ?? 0;
  const servers = store.servers.filter(s => s.status === 'connected');
  const mcpServers: Record<string, { url: string }> = {};
  for (const s of servers) {
    mcpServers[s.name] = { url: `http://localhost:${port}/mcp/${s.id}` };
  }
  if (Object.keys(mcpServers).length === 0) {
    mcpServers['your-server'] = { url: `http://localhost:${port}/mcp/<server-id>` };
  }
  return JSON.stringify({ mcpServers }, null, 2);
}

async function copySnippet() {
  try {
    await navigator.clipboard.writeText(manualSnippet());
    copied.value = true;
    setTimeout(() => (copied.value = false), 2000);
  } catch {
    // Clipboard may not be available in some webview contexts
  }
}

onMounted(() => fetchStatus());
</script>

<template>
  <div>
    <h2 class="mb-1 text-xs font-medium text-text-primary">Proxy</h2>
    <p class="mb-4 text-xs text-text-secondary">
      Each connected server gets its own proxy endpoint, so AI tools see them individually.
      Use the Connected Apps toggles, or manually add the config snippet below.
    </p>

    <div v-if="error" class="rounded bg-status-error/10 px-3 py-2 text-xs text-status-error">
      Failed to get proxy status: {{ error }}
    </div>

    <div v-else-if="status" class="space-y-3">
      <div class="flex items-center gap-2">
        <span
          class="inline-block h-2 w-2 rounded-full"
          :class="status.running ? 'bg-status-connected' : 'bg-status-error'"
        />
        <span class="text-xs text-text-secondary">
          <template v-if="status.running">
            Running on port <span class="font-mono font-medium text-text-primary">{{ status.port }}</span>
          </template>
          <template v-else>Not running</template>
        </span>
      </div>

      <div v-if="status.running">
        <p class="mb-1.5 text-xs text-text-muted">Manual config snippet:</p>
        <div class="relative">
          <pre class="overflow-x-auto rounded bg-surface-2 p-3 font-mono text-xs text-text-primary">{{ manualSnippet() }}</pre>
          <button
            class="absolute top-1.5 right-1.5 rounded bg-surface-3 px-2 py-0.5 text-[10px] text-text-muted transition hover:text-text-primary"
            @click="copySnippet"
          >
            {{ copied ? 'Copied' : 'Copy' }}
          </button>
        </div>
      </div>
    </div>

    <div v-else class="text-xs text-text-muted">Loading proxy status...</div>
  </div>
</template>
